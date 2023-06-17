use std::convert::TryFrom;

use jid::Jid;
use xmpp_parsers::{
  iq::IqSetPayload,
  jingle::{ContentId, Creator, Disposition, ReasonElement, Senders, SessionId},
  jingle_grouping::Group,
  jingle_ibb::Transport as IbbTransport,
  jingle_s5b::Transport as Socks5Transport,
  ns::{JINGLE, JINGLE_GROUPING, JINGLE_IBB, JINGLE_ICE_UDP, JINGLE_RTP, JINGLE_S5B},
  Element, Error,
};

use crate::{
  jingle_ice_udp::Transport as IceUdpTransport, jingle_rtp::Description as RtpDescription,
};

generate_attribute!(
  /// The action attribute.
  Action, "action", {
      /// Accept a content-add action received from another party.
      ContentAccept => "content-accept",

      /// Add one or more new content definitions to the session.
      ContentAdd => "content-add",

      /// Change the directionality of media sending.
      ContentModify => "content-modify",

      /// Reject a content-add action received from another party.
      ContentReject => "content-reject",

      /// Remove one or more content definitions from the session.
      ContentRemove => "content-remove",

      /// Exchange information about parameters for an application type.
      DescriptionInfo => "description-info",

      /// Exchange information about security preconditions.
      SecurityInfo => "security-info",

      /// Definitively accept a session negotiation.
      SessionAccept => "session-accept",

      /// Send session-level information, such as a ping or a ringing message.
      SessionInfo => "session-info",

      /// Request negotiation of a new Jingle session.
      SessionInitiate => "session-initiate",

      /// End an existing session.
      SessionTerminate => "session-terminate",

      /// Accept a transport-replace action received from another party.
      TransportAccept => "transport-accept",

      /// Exchange transport candidates.
      TransportInfo => "transport-info",

      /// Reject a transport-replace action received from another party.
      TransportReject => "transport-reject",

      /// Redefine a transport method or replace it with a different method.
      TransportReplace => "transport-replace",

      /// --- Non-standard values used by Jitsi Meet: ---

      /// Add a source to existing content.
      SourceAdd => "source-add",

      /// Remove a source from existing content.
      SourceRemove => "source-remove",
  }
);

/// The main Jingle container, to be included in an iq stanza.
#[derive(Debug, Clone, PartialEq)]
pub struct Jingle {
  /// The action to execute on both ends.
  pub action: Action,

  /// Who the initiator is.
  pub initiator: Option<Jid>,

  /// Who the responder is.
  pub responder: Option<Jid>,

  /// Unique session identifier between two entities.
  pub sid: SessionId,

  /// A list of contents to be negotiated in this session.
  pub contents: Vec<Content>,

  /// An optional reason.
  pub reason: Option<ReasonElement>,

  /// An optional grouping.
  pub group: Option<Group>,

  /// Payloads to be included.
  pub other: Vec<Element>,
}

impl IqSetPayload for Jingle {}

impl Jingle {
  /// Create a new Jingle element.
  pub fn new(action: Action, sid: SessionId) -> Jingle {
    Jingle {
      action,
      sid,
      initiator: None,
      responder: None,
      contents: Vec::new(),
      reason: None,
      group: None,
      other: Vec::new(),
    }
  }

  /// Set the initiator’s JID.
  pub fn with_initiator(mut self, initiator: Jid) -> Jingle {
    self.initiator = Some(initiator);
    self
  }

  /// Set the responder’s JID.
  pub fn with_responder(mut self, responder: Jid) -> Jingle {
    self.responder = Some(responder);
    self
  }

  /// Add a content to this Jingle container.
  pub fn add_content(mut self, content: Content) -> Jingle {
    self.contents.push(content);
    self
  }

  /// Set the reason in this Jingle container.
  pub fn set_reason(mut self, reason: ReasonElement) -> Jingle {
    self.reason = Some(reason);
    self
  }

  /// Set the grouping in this Jingle container.
  pub fn set_group(mut self, group: Group) -> Jingle {
    self.group = Some(group);
    self
  }
}

impl TryFrom<Element> for Jingle {
  type Error = Error;

  fn try_from(root: Element) -> Result<Jingle, Error> {
    check_self!(root, "jingle", JINGLE, "Jingle");

    let mut jingle = Jingle {
      action: get_attr!(root, "action", Required),
      initiator: get_attr!(root, "initiator", Option),
      responder: get_attr!(root, "responder", Option),
      sid: get_attr!(root, "sid", Required),
      contents: vec![],
      reason: None,
      group: None,
      other: vec![],
    };

    for child in root.children().cloned() {
      if child.is("content", JINGLE) {
        let content = Content::try_from(child)?;
        jingle.contents.push(content);
      }
      else if child.is("reason", JINGLE) {
        if jingle.reason.is_some() {
          return Err(Error::ParseError(
            "Jingle must not have more than one reason.",
          ));
        }
        let reason = ReasonElement::try_from(child)?;
        jingle.reason = Some(reason);
      }
      else if child.is("group", JINGLE_GROUPING) {
        if jingle.group.is_some() {
          return Err(Error::ParseError(
            "Jingle must not have more than one grouping.",
          ));
        }
        let group = Group::try_from(child)?;
        jingle.group = Some(group);
      }
      else {
        jingle.other.push(child);
      }
    }

    Ok(jingle)
  }
}

impl From<Jingle> for Element {
  fn from(jingle: Jingle) -> Element {
    Element::builder("jingle", JINGLE)
      .attr("action", jingle.action)
      .attr("initiator", jingle.initiator)
      .attr("responder", jingle.responder)
      .attr("sid", jingle.sid)
      .append_all(jingle.contents)
      .append_all(jingle.reason.map(Element::from))
      .append_all(jingle.group.map(Element::from))
      .build()
  }
}

/// Enum wrapping all of the various supported descriptions of a Content.
#[derive(Debug, Clone, PartialEq)]
pub enum Description {
  /// Jingle RTP Sessions (XEP-0167) description.
  Rtp(RtpDescription),

  /// To be used for any description that isn’t known at compile-time.
  Unknown(Element),
}

impl TryFrom<Element> for Description {
  type Error = Error;

  fn try_from(elem: Element) -> Result<Description, Error> {
    Ok(if elem.is("description", JINGLE_RTP) {
      Description::Rtp(RtpDescription::try_from(elem)?)
    }
    else {
      Description::Unknown(elem)
    })
  }
}

impl From<RtpDescription> for Description {
  fn from(desc: RtpDescription) -> Description {
    Description::Rtp(desc)
  }
}

impl From<Description> for Element {
  fn from(desc: Description) -> Element {
    match desc {
      Description::Rtp(desc) => desc.into(),
      Description::Unknown(elem) => elem,
    }
  }
}

/// Enum wrapping all of the various supported transports of a Content.
#[derive(Debug, Clone, PartialEq)]
pub enum Transport {
  /// Jingle ICE-UDP Bytestreams (XEP-0176) transport.
  IceUdp(IceUdpTransport),

  /// Jingle In-Band Bytestreams (XEP-0261) transport.
  Ibb(IbbTransport),

  /// Jingle SOCKS5 Bytestreams (XEP-0260) transport.
  Socks5(Socks5Transport),

  /// To be used for any transport that isn’t known at compile-time.
  Unknown(Element),
}

impl TryFrom<Element> for Transport {
  type Error = Error;

  fn try_from(elem: Element) -> Result<Transport, Error> {
    Ok(if elem.is("transport", JINGLE_ICE_UDP) {
      Transport::IceUdp(IceUdpTransport::try_from(elem)?)
    }
    else if elem.is("transport", JINGLE_IBB) {
      Transport::Ibb(IbbTransport::try_from(elem)?)
    }
    else if elem.is("transport", JINGLE_S5B) {
      Transport::Socks5(Socks5Transport::try_from(elem)?)
    }
    else {
      Transport::Unknown(elem)
    })
  }
}

impl From<IceUdpTransport> for Transport {
  fn from(transport: IceUdpTransport) -> Transport {
    Transport::IceUdp(transport)
  }
}

impl From<IbbTransport> for Transport {
  fn from(transport: IbbTransport) -> Transport {
    Transport::Ibb(transport)
  }
}

impl From<Socks5Transport> for Transport {
  fn from(transport: Socks5Transport) -> Transport {
    Transport::Socks5(transport)
  }
}

impl From<Transport> for Element {
  fn from(transport: Transport) -> Element {
    match transport {
      Transport::IceUdp(transport) => transport.into(),
      Transport::Ibb(transport) => transport.into(),
      Transport::Socks5(transport) => transport.into(),
      Transport::Unknown(elem) => elem,
    }
  }
}

generate_element!(
  /// Describes a session’s content, there can be multiple content in one
  /// session.
  Content, "content", JINGLE,
  attributes: [
      /// Who created this content.
      creator: Option<Creator> = "creator",

      /// How the content definition is to be interpreted by the recipient.
      disposition: Default<Disposition> = "disposition",

      /// A per-session unique identifier for this content.
      name: Required<ContentId> = "name",

      /// Who can send data for this content.
      senders: Default<Senders> = "senders",
  ],
  children: [
      /// What to send.
      description: Option<Description> = ("description", *) => Description,

      /// How to send it.
      transport: Option<Transport> = ("transport", *) => Transport,

      /// With which security.
      security: Option<Element> = ("security", JINGLE) => Element
  ]
);

impl Content {
  /// Create a new content.
  pub fn new(creator: Creator, name: ContentId) -> Content {
    Content {
      creator: Some(creator),
      name,
      disposition: Disposition::Session,
      senders: Senders::Both,
      description: None,
      transport: None,
      security: None,
    }
  }

  /// Set how the content is to be interpreted by the recipient.
  pub fn with_disposition(mut self, disposition: Disposition) -> Content {
    self.disposition = disposition;
    self
  }

  /// Specify who can send data for this content.
  pub fn with_senders(mut self, senders: Senders) -> Content {
    self.senders = senders;
    self
  }

  /// Set the description of this content.
  pub fn with_description<D: Into<Description>>(mut self, description: D) -> Content {
    self.description = Some(description.into());
    self
  }

  /// Set the transport of this content.
  pub fn with_transport<T: Into<Transport>>(mut self, transport: T) -> Content {
    self.transport = Some(transport.into());
    self
  }

  /// Set the security of this content.
  pub fn with_security(mut self, security: Element) -> Content {
    self.security = Some(security);
    self
  }
}
