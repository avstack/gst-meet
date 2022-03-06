use xmpp_parsers::{
  jingle_ice_udp::Candidate,
  ns::{JINGLE_DTLS, JINGLE_ICE_UDP},
};

use crate::{jingle_dtls_srtp::Fingerprint, ns::JITSI_COLIBRI};

generate_element!(
  /// Wrapper element for an ICE-UDP transport.
  #[derive(Default)]
  Transport, "transport", JINGLE_ICE_UDP,
  attributes: [
    /// A Password as defined in ICE-CORE.
    pwd: Option<String> = "pwd",

    /// A User Fragment as defined in ICE-CORE.
    ufrag: Option<String> = "ufrag",
  ],
  children: [
    /// List of candidates for this ICE-UDP session.
    candidates: Vec<Candidate> = ("candidate", JINGLE_ICE_UDP) => Candidate,

    /// Fingerprint of the key used for the DTLS handshake.
    fingerprint: Option<Fingerprint> = ("fingerprint", JINGLE_DTLS) => Fingerprint,

    /// Details of the Colibri WebSocket
    web_socket: Option<WebSocket> = ("web-socket", JITSI_COLIBRI) => WebSocket
  ]
);

impl Transport {
  /// Create a new ICE-UDP transport.
  pub fn new() -> Transport {
    Default::default()
  }

  /// Add a candidate to this transport.
  pub fn add_candidate(mut self, candidate: Candidate) -> Self {
    self.candidates.push(candidate);
    self
  }

  /// Set the DTLS-SRTP fingerprint of this transport.
  pub fn with_fingerprint(mut self, fingerprint: Fingerprint) -> Self {
    self.fingerprint = Some(fingerprint);
    self
  }
}

generate_element!(
  /// Colibri WebSocket details
  WebSocket, "web-socket", JITSI_COLIBRI,
  attributes: [
      /// The WebSocket URL
      url: Required<String> = "url",
  ]
);
