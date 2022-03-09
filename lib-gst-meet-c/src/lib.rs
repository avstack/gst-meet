use std::{
  ffi::{c_void, CStr, CString},
  os::raw::c_char,
  ptr,
  sync::{
    atomic::{AtomicPtr, Ordering},
    Arc,
  },
};

use anyhow::Result;
use glib::{
  ffi::GMainContext,
  translate::{from_glib, from_glib_full, ToGlibPtr},
};
use lib_gst_meet::JitsiConferenceConfig;
pub use lib_gst_meet::{init_tracing, Authentication, Connection, JitsiConference, MediaType};
use tokio::runtime::Runtime;

pub struct Context {
  runtime: Runtime,
}

#[repr(C)]
pub struct ConferenceConfig {
  pub muc: *const c_char,
  pub focus: *const c_char,
  pub nick: *const c_char,
  pub region: *const c_char,
  pub video_codec: *const c_char,
}

#[repr(C)]
pub struct Participant {
  pub jid: *const c_char,
  pub muc_jid: *const c_char,
  pub nick: *const c_char,
}

trait ResultExt<T> {
  fn ok_raw_or_log(self) -> *mut T;
}

impl<T> ResultExt<T> for Result<T> {
  fn ok_raw_or_log(self) -> *mut T {
    match self {
      Ok(o) => Box::into_raw(Box::new(o)),
      Err(e) => {
        eprintln!("lib-gst-meet: {:?}", e);
        ptr::null_mut()
      },
    }
  }
}

#[no_mangle]
pub extern "C" fn gstmeet_init() -> *mut Context {
  Runtime::new()
    .map(|runtime| Context { runtime })
    .map_err(|e| e.into())
    .ok_raw_or_log()
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_init_tracing(level: *const c_char) {
  let level = CStr::from_ptr(level)
    .to_string_lossy()
    .parse()
    .expect("invalid tracing level");
  init_tracing(level);
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_deinit(context: *mut Context) {
  Box::from_raw(context);
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_connection_new(
  context: *mut Context,
  websocket_url: *const c_char,
  xmpp_domain: *const c_char,
  tls_insecure: bool,
) -> *mut Connection {
  let websocket_url = CStr::from_ptr(websocket_url);
  let xmpp_domain = CStr::from_ptr(xmpp_domain);
  (*context)
    .runtime
    .block_on(Connection::new(
      &websocket_url.to_string_lossy(),
      &xmpp_domain.to_string_lossy(),
      Authentication::Anonymous,
      tls_insecure,
    ))
    .map(|(connection, background)| {
      (*context).runtime.spawn(background);
      connection
    })
    .ok_raw_or_log()
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_connection_free(connection: *mut Connection) {
  Box::from_raw(connection);
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_connection_connect(
  context: *mut Context,
  connection: *mut Connection,
) -> bool {
  (*context)
    .runtime
    .block_on((*connection).connect())
    .map_err(|e| eprintln!("lib-gst-meet: {:?}", e))
    .is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_connection_join_conference(
  context: *mut Context,
  connection: *mut Connection,
  glib_main_context: *mut GMainContext,
  config: *const ConferenceConfig,
) -> *mut JitsiConference {
  let muc = match CStr::from_ptr((*config).muc).to_string_lossy().parse() {
    Ok(jid) => jid,
    Err(e) => {
      eprintln!("lib-gst-meet: invalid MUC JID: {:?}", e);
      return ptr::null_mut();
    },
  };
  let focus = match CStr::from_ptr((*config).focus).to_string_lossy().parse() {
    Ok(jid) => jid,
    Err(e) => {
      eprintln!("lib-gst-meet: invalid focus JID: {:?}", e);
      return ptr::null_mut();
    },
  };
  let region = if (*config).region.is_null() {
    None
  }
  else {
    Some(
      CStr::from_ptr((*config).region)
        .to_string_lossy()
        .to_string(),
    )
  };
  let config = JitsiConferenceConfig {
    muc,
    focus,
    nick: CStr::from_ptr((*config).nick).to_string_lossy().to_string(),
    region,
    video_codec: CStr::from_ptr((*config).video_codec)
      .to_string_lossy()
      .to_string(),
    extra_muc_features: vec![],

    // TODO
    start_bitrate: 800,
    stereo: false,

    recv_video_scale_width: 1280,
    recv_video_scale_height: 720,

    buffer_size: 200,

    #[cfg(feature = "log-rtp")]
    log_rtp: false,
    #[cfg(feature = "log-rtp")]
    log_rtcp: false,
  };
  (*context)
    .runtime
    .block_on(JitsiConference::join(
      (*connection).clone(),
      from_glib_full(glib_main_context),
      config,
    ))
    .ok_raw_or_log()
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_conference_leave(
  context: *mut Context,
  conference: *mut JitsiConference,
) -> bool {
  (*context)
    .runtime
    .block_on(Box::from_raw(conference).leave())
    .map_err(|e| eprintln!("lib-gst-meet: {:?}", e))
    .is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_conference_set_muted(
  context: *mut Context,
  conference: *mut JitsiConference,
  media_type: MediaType,
  muted: bool,
) -> bool {
  (*context)
    .runtime
    .block_on((*conference).set_muted(media_type, muted))
    .map_err(|e| eprintln!("lib-gst-meet: {:?}", e))
    .is_ok()
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_conference_pipeline(
  context: *mut Context,
  conference: *mut JitsiConference,
) -> *mut gstreamer::ffi::GstPipeline {
  (*context)
    .runtime
    .block_on((*conference).pipeline())
    .map(|pipeline| pipeline.to_glib_full())
    .map_err(|e| eprintln!("lib-gst-meet: {:?}", e))
    .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_conference_audio_sink_element(
  context: *mut Context,
  conference: *mut JitsiConference,
) -> *mut gstreamer::ffi::GstElement {
  (*context)
    .runtime
    .block_on((*conference).audio_sink_element())
    .map(|pipeline| pipeline.to_glib_full())
    .map_err(|e| eprintln!("lib-gst-meet: {:?}", e))
    .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_conference_video_sink_element(
  context: *mut Context,
  conference: *mut JitsiConference,
) -> *mut gstreamer::ffi::GstElement {
  (*context)
    .runtime
    .block_on((*conference).video_sink_element())
    .map(|pipeline| pipeline.to_glib_full())
    .map_err(|e| eprintln!("lib-gst-meet: {:?}", e))
    .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_conference_on_participant(
  context: *mut Context,
  conference: *mut JitsiConference,
  f: unsafe extern "C" fn(
    *mut JitsiConference,
    Participant,
    *mut c_void,
  ) -> *mut gstreamer::ffi::GstBin,
  ctx: *mut c_void,
) {
  let ctx = Arc::new(AtomicPtr::new(ctx));
  (*context).runtime.block_on(
    (*conference).on_participant(move |conference, participant| {
      let ctx = ctx.clone();
      Box::pin(async move {
        let participant = Participant {
          jid: participant
            .jid
            .map(
              |jid| Ok::<_, anyhow::Error>(CString::new(jid.to_string())?.into_raw() as *const _),
            )
            .transpose()?
            .unwrap_or_else(ptr::null),
          muc_jid: CString::new(participant.muc_jid.to_string())?.into_raw() as *const _,
          nick: participant
            .nick
            .map(|nick| Ok::<_, anyhow::Error>(CString::new(nick)?.into_raw() as *const _))
            .transpose()?
            .unwrap_or_else(ptr::null),
        };
        f(
          Box::into_raw(Box::new(conference)),
          participant,
          ctx.load(Ordering::Relaxed),
        );
        Ok(())
      })
    }),
  );
}

#[no_mangle]
pub unsafe extern "C" fn gstmeet_conference_set_pipeline_state(
  context: *mut Context,
  conference: *mut JitsiConference,
  state: gstreamer::ffi::GstState,
) -> bool {
  (*context)
    .runtime
    .block_on((*conference).set_pipeline_state(from_glib(state)))
    .map_err(|e| eprintln!("lib-gst-meet: {:?}", e))
    .is_ok()
}
