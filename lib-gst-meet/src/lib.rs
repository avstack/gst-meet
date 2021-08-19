mod colibri;
mod conference;
mod connection;
mod jingle;
mod pinger;
mod source;
mod stanza_filter;
mod util;
mod xmpp;

pub use crate::{
  colibri::ColibriMessage,
  conference::{JitsiConference, JitsiConferenceConfig, Participant},
  connection::JitsiConnection,
  source::MediaType,
};

#[cfg(feature = "tracing-subscriber")]
pub fn init_tracing(level: tracing::Level) {
  tracing_subscriber::fmt()
    .with_max_level(level)
    .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
    .with_target(false)
    .init();
}