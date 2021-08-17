pub mod conference;
pub mod connection;
mod jingle;
mod pinger;
pub mod source;
mod stanza_filter;
mod util;
mod xmpp;

pub use crate::{
  conference::{JitsiConference, JitsiConferenceConfig, Participant},
  connection::JitsiConnection,
  source::{MediaType, Source},
};

#[cfg(feature = "tracing-subscriber")]
pub fn init_tracing(level: tracing::Level) {
  tracing_subscriber::fmt()
    .with_max_level(level)
    .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
    .with_target(false)
    .init();
}