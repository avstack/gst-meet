use minidom::{Element, NSChoice::Any};
use xmpp_parsers::ns::JINGLE_SSMA;

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
    // The namespace should be JINGLE_SSMA, but we have to use Any because Jicofo produces
    // parameters with the wrong namespace.
    // https://github.com/jitsi/jitsi-xmpp-extensions/issues/81
    parameters: Vec<Parameter> = ("parameter", Any) => Parameter,

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

/// Parameter associated with a ssrc.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
  pub name: String,
  pub value: Option<String>,
}

impl TryFrom<Element> for Parameter {
  type Error = xmpp_parsers::Error;

  fn try_from(root: Element) -> Result<Parameter, xmpp_parsers::Error> {
    // The namespace should be JINGLE_SSMA, but we have to use Any because Jicofo produces
    // parameters with the wrong namespace.
    // https://github.com/jitsi/jitsi-xmpp-extensions/issues/81
    check_self!(root, "parameter", Any, "Parameter");
    Ok(Parameter {
      name: get_attr!(root, "name", Required),
      value: get_attr!(root, "value", Option),
    })
  }
}

impl From<Parameter> for Element {
  fn from(parameter: Parameter) -> Element {
    Element::builder("parameter", JINGLE_SSMA)
      .attr("name", parameter.name)
      .attr("value", parameter.value)
      .build()
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

generate_attribute!(
  /// From RFC5888, the list of allowed semantics.
  Semantics, "semantics", {
      /// Lip Synchronization, defined in RFC5888.
      Ls => "LS",

      /// Flow Identification, defined in RFC5888.
      Fid => "FID",

      /// Single Reservation Flow, defined in RFC3524.
      Srf => "SRF",

      /// Alternative Network Address Types, defined in RFC4091.
      Anat => "ANAT",

      /// Forward Error Correction, defined in RFC4756.
      Fec => "FEC",

      /// Decoding Dependency, defined in RFC5583.
      Ddp => "DDP",

      /// Simulcast.
      Sim => "SIM",
  }
);
