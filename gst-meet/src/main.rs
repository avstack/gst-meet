use std::time::Duration;

use anyhow::{bail, Context, Result};
#[cfg(target_os = "macos")]
use cocoa::appkit::NSApplication;
use colibri::{ColibriMessage, Constraints, VideoType};
use glib::ObjectExt;
use gstreamer::{
  prelude::{ElementExt, GstBinExt},
  GhostPad,
};
use lib_gst_meet::{
  init_tracing, Authentication, Connection, JitsiConference, JitsiConferenceConfig, MediaType,
};
use structopt::StructOpt;
use tokio::{signal::ctrl_c, task, time::timeout};
use tracing::{error, info, trace, warn};

#[derive(Debug, Clone, StructOpt)]
#[structopt(
  name = "gst-meet",
  about = "Connect a GStreamer pipeline to a Jitsi Meet conference."
)]
struct Opt {
  #[structopt(long)]
  web_socket_url: String,
  #[structopt(long)]
  xmpp_domain: String,
  #[structopt(long)]
  room_name: String,
  #[structopt(long)]
  muc_domain: Option<String>,
  #[structopt(long)]
  focus_jid: Option<String>,
  #[structopt(long, default_value = "vp8")]
  video_codec: String,
  #[structopt(long, default_value = "gst-meet")]
  nick: String,
  #[structopt(long)]
  region: Option<String>,
  #[structopt(long)]
  send_pipeline: Option<String>,
  #[structopt(long)]
  recv_pipeline_participant_template: Option<String>,
  #[structopt(long)]
  select_endpoints: Option<String>,
  #[structopt(long)]
  last_n: Option<i32>,
  #[structopt(long)]
  recv_video_height: Option<i32>,
  #[structopt(long)]
  video_type: Option<String>,
  #[structopt(short, long, parse(from_occurrences))]
  verbose: u8,
}

#[cfg(not(target_os = "macos"))]
#[tokio::main]
async fn main() -> Result<()> {
  main_inner().await
}

#[cfg(target_os = "macos")]
fn main() {
  // GStreamer requires an NSApp event loop in order for osxvideosink etc to work.
  let app = unsafe { cocoa::appkit::NSApp() };

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap();

  rt.spawn(async move {
    if let Err(e) = main_inner().await {
      error!("fatal: {:?}", e);
    }
    unsafe {
      cocoa::appkit::NSApp().stop_(cocoa::base::nil);
    }
    std::process::exit(0);
  });

  unsafe {
    app.run();
  }
}

fn init_gstreamer() -> Result<()> {
  trace!("starting gstreamer init");
  gstreamer::init()?;
  trace!("finished gstreamer init");
  Ok(())
}

async fn main_inner() -> Result<()> {
  let opt = Opt::from_args();

  init_tracing(match opt.verbose {
    0 => tracing::Level::INFO,
    1 => tracing::Level::DEBUG,
    _ => tracing::Level::TRACE,
  });
  glib::log_set_default_handler(glib::rust_log_handler);

  init_gstreamer()?;

  // Parse pipeline early so that we don't bother connecting to the conference if it's invalid.

  let send_pipeline = opt
    .send_pipeline
    .as_ref()
    .map(|pipeline| gstreamer::parse_bin_from_description(pipeline, false))
    .transpose()
    .context("failed to parse send pipeline")?;

  let (connection, background) = Connection::new(
    &opt.web_socket_url,
    &opt.xmpp_domain,
    Authentication::Anonymous,
  )
  .await
  .context("failed to connect")?;

  tokio::spawn(background);

  connection.connect().await?;

  let room_jid = format!(
    "{}@{}",
    opt.room_name,
    opt
      .muc_domain
      .clone()
      .unwrap_or_else(|| { format!("conference.{}", opt.xmpp_domain) }),
  );

  let focus_jid = opt
    .focus_jid
    .clone()
    .unwrap_or_else(|| format!("focus@auth.{}/focus", opt.xmpp_domain,));

  let Opt {
    nick,
    region,
    video_codec,
    recv_pipeline_participant_template,
    ..
  } = opt;

  let config = JitsiConferenceConfig {
    muc: room_jid.parse()?,
    focus: focus_jid.parse()?,
    nick,
    region,
    video_codec,
    extra_muc_features: vec![],
  };

  let main_loop = glib::MainLoop::new(None, false);

  let conference = JitsiConference::join(connection, main_loop.context(), config)
    .await
    .context("failed to join conference")?;

  if opt.select_endpoints.is_some() || opt.last_n.is_some() || opt.recv_video_height.is_some() {
    conference
      .send_colibri_message(ColibriMessage::ReceiverVideoConstraints {
        last_n: opt.last_n,
        selected_endpoints: opt
          .select_endpoints
          .map(|endpoints| endpoints.split(',').map(ToOwned::to_owned).collect()),
        on_stage_endpoints: None,
        default_constraints: opt.recv_video_height.map(|height| Constraints {
          max_height: Some(height),
          ideal_height: None,
        }),
        constraints: None,
      })
      .await?;
  }

  if let Some(video_type) = opt.video_type {
    conference
      .send_colibri_message(ColibriMessage::VideoTypeMessage {
        video_type: match video_type.as_str() {
          "camera" => VideoType::Camera,
          "desktop" => VideoType::Desktop,
          other => bail!(format!("invalid video type: {}", other)),
        },
      })
      .await?;
  }

  if let Some(bin) = send_pipeline {
    conference.add_bin(&bin).await?;

    if let Some(audio) = bin.by_name("audio") {
      info!("Found audio element in pipeline, linking...");
      let audio_sink = conference.audio_sink_element().await?;
      audio.link(&audio_sink)?;
    }
    else {
      conference.set_muted(MediaType::Audio, true).await?;
    }

    if let Some(video) = bin.by_name("video") {
      info!("Found video element in pipeline, linking...");
      let video_sink = conference.video_sink_element().await?;
      video.link(&video_sink)?;
    }
    else {
      conference.set_muted(MediaType::Video, true).await?;
    }
  }
  else {
    conference.set_muted(MediaType::Audio, true).await?;
    conference.set_muted(MediaType::Video, true).await?;
  }

  conference
    .on_participant(move |conference, participant| {
      let recv_pipeline_participant_template = recv_pipeline_participant_template.clone();
      Box::pin(async move {
        info!("New participant: {:?}", participant);

        if let Some(template) = recv_pipeline_participant_template {
          let pipeline_description = template
            .replace(
              "{jid}",
              &participant
                .jid
                .as_ref()
                .map(|jid| jid.to_string())
                .unwrap_or_default(),
            )
            .replace(
              "{jid_user}",
              participant
                .jid
                .as_ref()
                .and_then(|jid| jid.node.as_deref())
                .unwrap_or_default(),
            )
            .replace("{participant_id}", &participant.muc_jid.resource)
            .replace("{nick}", &participant.nick.unwrap_or_default());

          let bin = gstreamer::parse_bin_from_description(&pipeline_description, false)
            .context("failed to parse recv pipeline participant template")?;

          if let Some(audio_sink_element) = bin.by_name("audio") {
            let sink_pad = audio_sink_element.static_pad("sink").context(
              "audio sink element in recv pipeline participant template has no sink pad",
            )?;
            bin.add_pad(&GhostPad::with_target(Some("audio"), &sink_pad)?)?;
          }
          else {
            info!("No audio sink element found in recv pipeline participant template");
          }

          if let Some(video_sink_element) = bin.by_name("video") {
            let sink_pad = video_sink_element.static_pad("sink").context(
              "video sink element in recv pipeline participant template has no sink pad",
            )?;
            bin.add_pad(&GhostPad::with_target(Some("video"), &sink_pad)?)?;
          }
          else {
            info!("No video sink element found in recv pipeline participant template");
          }

          bin.set_property(
            "name",
            format!("participant_{}", participant.muc_jid.resource),
          )?;
          conference.add_bin(&bin).await?;
        }
        else {
          info!("No template for handling new participant");
        }

        Ok(())
      })
    })
    .await;

  conference
    .on_participant_left(move |_conference, participant| {
      Box::pin(async move {
        info!("Participant left: {:?}", participant);
        Ok(())
      })
    })
    .await;

  conference
    .on_colibri_message(move |_conference, message| {
      Box::pin(async move {
        info!("Colibri message: {:?}", message);
        Ok(())
      })
    })
    .await;

  conference
    .set_pipeline_state(gstreamer::State::Playing)
    .await?;

  let conference_ = conference.clone();
  let main_loop_ = main_loop.clone();
  tokio::spawn(async move {
    ctrl_c().await.unwrap();

    info!("Exiting...");

    match timeout(Duration::from_secs(10), conference_.leave()).await {
      Ok(Ok(_)) => {},
      Ok(Err(e)) => warn!("Error leaving conference: {:?}", e),
      Err(_) => warn!("Timed out leaving conference"),
    }

    main_loop_.quit();
  });

  task::spawn_blocking(move || main_loop.run()).await?;

  Ok(())
}
