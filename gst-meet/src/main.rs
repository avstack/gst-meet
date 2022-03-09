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
use http::Uri;
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

  #[structopt(
    long,
    help = "If not specified, assumed to be the host part of <web-socket-url>"
  )]
  xmpp_domain: Option<String>,

  #[structopt(long)]
  room_name: String,

  #[structopt(
    long,
    help = "If not specified, assumed to be conference.<xmpp-domain>"
  )]
  muc_domain: Option<String>,

  #[structopt(
    long,
    help = "If not specified, assumed to be focus@auth.<xmpp-domain>/focus"
  )]
  focus_jid: Option<String>,

  #[structopt(
    long,
    help = "If not specified, anonymous auth is used."
  )]
  xmpp_username: Option<String>,

  #[structopt(long)]
  xmpp_password: Option<String>,

  #[structopt(
    long,
    default_value = "vp9",
    help = "The video codec to negotiate support for. One of: vp9, vp8, h264"
  )]
  video_codec: String,

  #[structopt(long, default_value = "gst-meet")]
  nick: String,

  #[structopt(long)]
  region: Option<String>,

  #[structopt(long)]
  send_pipeline: Option<String>,

  #[structopt(
    long,
    help = "A GStreamer pipeline which will be instantiated at startup. If an element named 'audio' is found, every remote participant's audio will be linked to it (and any 'audio' element in the recv-pipeline-participant-template will be ignored). If an element named 'video' is found, every remote participant's video will be linked to it (and any 'video' element in the recv-pipeline-participant-template will be ignored)."
  )]
  recv_pipeline: Option<String>,

  #[structopt(
    long,
    help = "A GStreamer pipeline which will be instantiated for each remote participant. If an element named 'audio' is found, the participant's audio will be linked to it. If an element named 'video' is found, the participant's video will be linked to it."
  )]
  recv_pipeline_participant_template: Option<String>,

  #[structopt(
    long,
    help = "Comma-separated endpoint IDs to select (prioritise receiving of)"
  )]
  select_endpoints: Option<String>,

  #[structopt(
    long,
    help = "The maximum number of video streams we would like to receive"
  )]
  last_n: Option<u16>,

  #[structopt(
    long,
    default_value = "720",
    help = "The maximum height we plan to send video at (used for stats only)."
  )]
  send_video_height: u16,

  #[structopt(
    long,
    help = "The video type to signal that we are sending. One of: camera, desktop"
  )]
  video_type: Option<String>,

  #[structopt(
    long,
    default_value = "1280",
    help = "The width to scale received video to before passing it to the recv-pipeline."
  )]
  recv_video_scale_width: u16,

  #[structopt(
    long,
    default_value = "720",
    help = "The height to scale received video to before passing it to the recv-pipeline. This will also be signalled as the maximum height that JVB should send video to us at."
  )]
  recv_video_scale_height: u16,

  #[structopt(
    long,
    default_value = "200",
    help = "The size of the jitter buffers in milliseconds. Larger values are more resilient to packet loss and jitter, smaller values give lower latency."
  )]
  buffer_size: u32,

  #[structopt(long)]
  start_bitrate: Option<u32>,

  #[structopt(long)]
  stereo: Option<bool>,

  #[structopt(short, long, parse(from_occurrences))]
  verbose: u8,

  #[cfg(feature = "tls-insecure")]
  #[structopt(
    long,
    help = "Disable TLS certificate verification (use with extreme caution)"
  )]
  tls_insecure: bool,

  #[cfg(feature = "log-rtp")]
  #[structopt(long, help = "Log all RTP packets at DEBUG level (extremely verbose)")]
  log_rtp: bool,

  #[cfg(feature = "log-rtp")]
  #[structopt(long, help = "Log all RTCP packets at DEBUG level")]
  log_rtcp: bool,
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

  // Parse pipelines early so that we don't bother connecting to the conference if it's invalid.

  let send_pipeline = opt
    .send_pipeline
    .as_ref()
    .map(|pipeline| gstreamer::parse_bin_from_description(pipeline, false))
    .transpose()
    .context("failed to parse send pipeline")?;

  let recv_pipeline = opt
    .recv_pipeline
    .as_ref()
    .map(|pipeline| gstreamer::parse_bin_from_description(pipeline, false))
    .transpose()
    .context("failed to parse recv pipeline")?;

  let web_socket_url: Uri = opt.web_socket_url.parse()?;

  let xmpp_domain = opt
    .xmpp_domain
    .as_deref()
    .or_else(|| web_socket_url.host())
    .context("invalid WebSocket URL")?;

  let (connection, background) = Connection::new(
    &opt.web_socket_url,
    xmpp_domain,
    match opt.xmpp_username {
      Some(username) => Authentication::Plain {
        username,
        password: opt.xmpp_password.context("if xmpp-username is provided, xmpp-password must also be provided")?,
      },
      None => Authentication::Anonymous,
    },
    #[cfg(feature = "tls-insecure")]
    opt.tls_insecure,
    #[cfg(not(feature = "tls-insecure"))]
    false,
  )
  .await
  .context("failed to build connection")?;

  tokio::spawn(background);

  connection.connect().await?;

  let room_jid = format!(
    "{}@{}",
    opt.room_name,
    opt
      .muc_domain
      .clone()
      .unwrap_or_else(|| { format!("conference.{}", xmpp_domain) }),
  );

  let focus_jid = opt
    .focus_jid
    .clone()
    .unwrap_or_else(|| format!("focus@auth.{}/focus", xmpp_domain));

  let Opt {
    nick,
    region,
    video_codec,
    recv_pipeline_participant_template,
    send_video_height,
    recv_video_scale_height,
    recv_video_scale_width,
    buffer_size,
    start_bitrate,
    stereo,
    #[cfg(feature = "log-rtp")]
    log_rtp,
    #[cfg(feature = "log-rtp")]
    log_rtcp,
    ..
  } = opt;

  let config = JitsiConferenceConfig {
    muc: room_jid.parse()?,
    focus: focus_jid.parse()?,
    nick,
    region,
    video_codec,
    extra_muc_features: vec![],
    start_bitrate: start_bitrate.unwrap_or(800),
    stereo: stereo.unwrap_or_default(),
    recv_video_scale_height,
    recv_video_scale_width,
    buffer_size,
    #[cfg(feature = "log-rtp")]
    log_rtp,
    #[cfg(feature = "log-rtp")]
    log_rtcp,
  };

  let main_loop = glib::MainLoop::new(None, false);

  let conference = JitsiConference::join(connection, main_loop.context(), config)
    .await
    .context("failed to join conference")?;

  conference
    .set_send_resolution(send_video_height.into())
    .await;

  conference
    .send_colibri_message(ColibriMessage::ReceiverVideoConstraints {
      last_n: Some(opt.last_n.map(i32::from).unwrap_or(-1)),
      selected_endpoints: opt
        .select_endpoints
        .map(|endpoints| endpoints.split(',').map(ToOwned::to_owned).collect()),
      on_stage_endpoints: None,
      default_constraints: Some(Constraints {
        max_height: Some(opt.recv_video_scale_height.into()),
        ideal_height: None,
      }),
      constraints: None,
    })
    .await?;

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

  if let Some(bin) = recv_pipeline {
    conference.add_bin(&bin).await?;

    if let Some(audio_element) = bin.by_name("audio") {
      info!(
        "recv pipeline has an audio element, a sink pad will be requested from it for each participant"
      );
      conference
        .set_remote_participant_audio_sink_element(Some(audio_element))
        .await;
    }

    if let Some(video_element) = bin.by_name("video") {
      info!(
        "recv pipeline has a video element, a sink pad will be requested from it for each participant"
      );
      conference
        .set_remote_participant_video_sink_element(Some(video_element))
        .await;
    }
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

          if let Some(video_sink_element) = bin.by_name("video") {
            let sink_pad = video_sink_element.static_pad("sink").context(
              "video sink element in recv pipeline participant template has no sink pad",
            )?;
            bin.add_pad(&GhostPad::with_target(Some("video"), &sink_pad)?)?;
          }

          bin.set_property(
            "name",
            format!("participant_{}", participant.muc_jid.resource),
          );
          conference.add_bin(&bin).await?;
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
