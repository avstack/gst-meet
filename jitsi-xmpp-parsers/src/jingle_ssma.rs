use xmpp_parsers::{
  jingle_ssma::{Parameter, Semantics},
  ns::JINGLE_SSMA,
};

use crate::ns::JITSI_MEET;

generate_element!(
  /// Source element for the ssrc SDP attribute.
  Source, "source", JINGLE_SSMA,
  attributes: [
    /// Maps to the ssrc-id parameter.
    id: Required<u32> = "ssrc",
  ],
  children: [
    /// List of attributes for this source.
    parameters: Vec<Parameter> = ("parameter", JINGLE_SSMA) => Parameter,

    /// --- Non-standard attributes used by Jitsi Meet: ---

    /// ssrc-info for this source.
    info: Option<SsrcInfo> = ("ssrc-info", JITSI_MEET) => SsrcInfo
  ]
);

impl Source {
  /// Create a new SSMA Source element.
  pub fn new(id: u32) -> Source {
    Source {
      id,
      parameters: Vec::new(),
      info: None,
    }
  }
}

generate_element!(
  /// ssrc-info associated with a ssrc.
  SsrcInfo, "ssrc-info", JITSI_MEET,
  attributes: [
    /// The owner of the ssrc.
    owner: Required<String> = "owner"
  ]
);

generate_element!(
  /// Element grouping multiple ssrc.
  Group, "ssrc-group", JINGLE_SSMA,
  attributes: [
      /// The semantics of this group.
      semantics: Required<Semantics> = "semantics",
  ],
  children: [
      /// The various ssrc concerned by this group.
      sources: Vec<Source> = ("source", JINGLE_SSMA) => Source
  ]
);
