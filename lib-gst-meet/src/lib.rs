pub mod colibri;
mod conference;
mod jingle;
mod pinger;
mod source;
mod stanza_filter;
mod tls;
mod util;
mod xmpp;

pub use xmpp_parsers;

pub use crate::{
  conference::{Feature, JitsiConference, JitsiConferenceConfig, Participant},
  source::MediaType,
  stanza_filter::StanzaFilter,
  xmpp::connection::{Authentication, Connection},
};

#[cfg(feature = "tracing-subscriber")]
pub fn init_tracing(level: tracing::Level) {
  tracing_subscriber::fmt()
    .with_max_level(level)
    .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
    .with_target(false)
    .init();
}
