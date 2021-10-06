// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::iq::IqSetPayload;
use crate::jingle_grouping::Group;
use crate::jingle_ibb::Transport as IbbTransport;
use crate::jingle_ice_udp::Transport as IceUdpTransport;
use crate::jingle_rtp::Description as RtpDescription;
use crate::jingle_s5b::Transport as Socks5Transport;
use crate::ns;
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

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

        /// --- Non-standard messages used by Jitsi Meet:

        /// Add a source to existing content.
        SourceAdd => "source-add",
    }
);

generate_attribute!(
    /// Which party originally generated the content type.
    Creator, "creator", {
        /// This content was created by the initiator of this session.
        Initiator => "initiator",

        /// This content was created by the responder of this session.
        Responder => "responder",
    }
);

generate_attribute!(
    /// Which parties in the session will be generating content.
    Senders, "senders", {
        /// Both parties can send for this content.
        Both => "both",

        /// Only the initiator can send for this content.
        Initiator => "initiator",

        /// No one can send for this content.
        None => "none",

        /// Only the responder can send for this content.
        Responder => "responder",
    }
);

generate_attribute!(
    /// How the content definition is to be interpreted by the recipient. The
    /// meaning of this attribute matches the "Content-Disposition" header as
    /// defined in RFC 2183 and applied to SIP by RFC 3261.
    ///
    /// Possible values are defined here:
    /// https://www.iana.org/assignments/cont-disp/cont-disp.xhtml
    Disposition, "disposition", {
        /// Displayed automatically.
        Inline => "inline",

        /// User controlled display.
        Attachment => "attachment",

        /// Process as form response.
        FormData => "form-data",

        /// Tunneled content to be processed silently.
        Signal => "signal",

        /// The body is a custom ring tone to alert the user.
        Alert => "alert",

        /// The body is displayed as an icon to the user.
        Icon => "icon",

        /// The body should be displayed to the user.
        Render => "render",

        /// The body contains a list of URIs that indicates the recipients of
        /// the request.
        RecipientListHistory => "recipient-list-history",

        /// The body describes a communications session, for example, an
        /// RFC2327 SDP body.
        Session => "session",

        /// Authenticated Identity Body.
        Aib => "aib",

        /// The body describes an early communications session, for example,
        /// and [RFC2327] SDP body.
        EarlySession => "early-session",

        /// The body includes a list of URIs to which URI-list services are to
        /// be applied.
        RecipientList => "recipient-list",

        /// The payload of the message carrying this Content-Disposition header
        /// field value is an Instant Message Disposition Notification as
        /// requested in the corresponding Instant Message.
        Notification => "notification",

        /// The body needs to be handled according to a reference to the body
        /// that is located in the same SIP message as the body.
        ByReference => "by-reference",

        /// The body contains information associated with an Info Package.
        InfoPackage => "info-package",

        /// The body describes either metadata about the RS or the reason for
        /// the metadata snapshot request as determined by the MIME value
        /// indicated in the Content-Type.
        RecordingSession => "recording-session",
    }, Default = Session
);

generate_id!(
    /// An unique identifier in a session, referencing a
    /// [struct.Content.html](Content element).
    ContentId
);

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
        Ok(if elem.is("description", ns::JINGLE_RTP) {
            Description::Rtp(RtpDescription::try_from(elem)?)
        } else {
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
        Ok(if elem.is("transport", ns::JINGLE_ICE_UDP) {
            Transport::IceUdp(IceUdpTransport::try_from(elem)?)
        } else if elem.is("transport", ns::JINGLE_IBB) {
            Transport::Ibb(IbbTransport::try_from(elem)?)
        } else if elem.is("transport", ns::JINGLE_S5B) {
            Transport::Socks5(Socks5Transport::try_from(elem)?)
        } else {
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
        senders: Option<Senders> = "senders",
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
            senders: Some(Senders::Both),
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
        self.senders = Some(senders);
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

/// Lists the possible reasons to be included in a Jingle iq.
#[derive(Debug, Clone, PartialEq)]
pub enum Reason {
    /// The party prefers to use an existing session with the peer rather than
    /// initiate a new session; the Jingle session ID of the alternative
    /// session SHOULD be provided as the XML character data of the <sid/>
    /// child.
    AlternativeSession, //(String),

    /// The party is busy and cannot accept a session.
    Busy,

    /// The initiator wishes to formally cancel the session initiation request.
    Cancel,

    /// The action is related to connectivity problems.
    ConnectivityError,

    /// The party wishes to formally decline the session.
    Decline,

    /// The session length has exceeded a pre-defined time limit (e.g., a
    /// meeting hosted at a conference service).
    Expired,

    /// The party has been unable to initialize processing related to the
    /// application type.
    FailedApplication,

    /// The party has been unable to establish connectivity for the transport
    /// method.
    FailedTransport,

    /// The action is related to a non-specific application error.
    GeneralError,

    /// The entity is going offline or is no longer available.
    Gone,

    /// The party supports the offered application type but does not support
    /// the offered or negotiated parameters.
    IncompatibleParameters,

    /// The action is related to media processing problems.
    MediaError,

    /// The action is related to a violation of local security policies.
    SecurityError,

    /// The action is generated during the normal course of state management
    /// and does not reflect any error.
    Success,

    /// A request has not been answered so the sender is timing out the
    /// request.
    Timeout,

    /// The party supports none of the offered application types.
    UnsupportedApplications,

    /// The party supports none of the offered transport methods.
    UnsupportedTransports,
}

impl FromStr for Reason {
    type Err = Error;

    fn from_str(s: &str) -> Result<Reason, Error> {
        Ok(match s {
            "alternative-session" => Reason::AlternativeSession,
            "busy" => Reason::Busy,
            "cancel" => Reason::Cancel,
            "connectivity-error" => Reason::ConnectivityError,
            "decline" => Reason::Decline,
            "expired" => Reason::Expired,
            "failed-application" => Reason::FailedApplication,
            "failed-transport" => Reason::FailedTransport,
            "general-error" => Reason::GeneralError,
            "gone" => Reason::Gone,
            "incompatible-parameters" => Reason::IncompatibleParameters,
            "media-error" => Reason::MediaError,
            "security-error" => Reason::SecurityError,
            "success" => Reason::Success,
            "timeout" => Reason::Timeout,
            "unsupported-applications" => Reason::UnsupportedApplications,
            "unsupported-transports" => Reason::UnsupportedTransports,

            _ => return Err(Error::ParseError("Unknown reason.")),
        })
    }
}

impl From<Reason> for Element {
    fn from(reason: Reason) -> Element {
        Element::builder(
            match reason {
                Reason::AlternativeSession => "alternative-session",
                Reason::Busy => "busy",
                Reason::Cancel => "cancel",
                Reason::ConnectivityError => "connectivity-error",
                Reason::Decline => "decline",
                Reason::Expired => "expired",
                Reason::FailedApplication => "failed-application",
                Reason::FailedTransport => "failed-transport",
                Reason::GeneralError => "general-error",
                Reason::Gone => "gone",
                Reason::IncompatibleParameters => "incompatible-parameters",
                Reason::MediaError => "media-error",
                Reason::SecurityError => "security-error",
                Reason::Success => "success",
                Reason::Timeout => "timeout",
                Reason::UnsupportedApplications => "unsupported-applications",
                Reason::UnsupportedTransports => "unsupported-transports",
            },
            ns::JINGLE,
        )
        .build()
    }
}

type Lang = String;

/// Informs the recipient of something.
#[derive(Debug, Clone, PartialEq)]
pub struct ReasonElement {
    /// The list of possible reasons to be included in a Jingle iq.
    pub reason: Reason,

    /// A human-readable description of this reason.
    pub texts: BTreeMap<Lang, String>,
}

impl fmt::Display for ReasonElement {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", Element::from(self.reason.clone()).name())?;
        if let Some(text) = self.texts.get("en") {
            write!(fmt, ": {}", text)?;
        } else if let Some(text) = self.texts.get("") {
            write!(fmt, ": {}", text)?;
        }
        Ok(())
    }
}

impl TryFrom<Element> for ReasonElement {
    type Error = Error;

    fn try_from(elem: Element) -> Result<ReasonElement, Error> {
        check_self!(elem, "reason", JINGLE);
        check_no_attributes!(elem, "reason");
        let mut reason = None;
        let mut texts = BTreeMap::new();
        for child in elem.children() {
            if child.is("text", ns::JINGLE) {
                check_no_children!(child, "text");
                check_no_unknown_attributes!(child, "text", ["xml:lang"]);
                let lang = get_attr!(elem, "xml:lang", Default);
                if texts.insert(lang, child.text()).is_some() {
                    return Err(Error::ParseError(
                        "Text element present twice for the same xml:lang.",
                    ));
                }
            } else if child.has_ns(ns::JINGLE) {
                if reason.is_some() {
                    return Err(Error::ParseError(
                        "Reason must not have more than one reason.",
                    ));
                }
                check_no_children!(child, "reason");
                check_no_attributes!(child, "reason");
                reason = Some(child.name().parse()?);
            } else {
                return Err(Error::ParseError("Reason contains a foreign element."));
            }
        }
        let reason = reason.ok_or(Error::ParseError("Reason doesn’t contain a valid reason."))?;
        Ok(ReasonElement { reason, texts })
    }
}

impl From<ReasonElement> for Element {
    fn from(reason: ReasonElement) -> Element {
        Element::builder("reason", ns::JINGLE)
            .append(Element::from(reason.reason))
            .append_all(reason.texts.into_iter().map(|(lang, text)| {
                Element::builder("text", ns::JINGLE)
                    .attr("xml:lang", lang)
                    .append(text)
            }))
            .build()
    }
}

generate_id!(
    /// Unique identifier for a session between two JIDs.
    SessionId
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
        check_no_unknown_attributes!(root, "Jingle", ["action", "initiator", "responder", "sid"]);

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
            if child.is("content", ns::JINGLE) {
                let content = Content::try_from(child)?;
                jingle.contents.push(content);
            } else if child.is("reason", ns::JINGLE) {
                if jingle.reason.is_some() {
                    return Err(Error::ParseError(
                        "Jingle must not have more than one reason.",
                    ));
                }
                let reason = ReasonElement::try_from(child)?;
                jingle.reason = Some(reason);
            } else if child.is("group", ns::JINGLE_GROUPING) {
                if jingle.group.is_some() {
                    return Err(Error::ParseError(
                        "Jingle must not have more than one grouping.",
                    ));
                }
                let group = Group::try_from(child)?;
                jingle.group = Some(group);
            } else {
                jingle.other.push(child);
            }
        }

        Ok(jingle)
    }
}

impl From<Jingle> for Element {
    fn from(jingle: Jingle) -> Element {
        Element::builder("jingle", ns::JINGLE)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Action, 1);
        assert_size!(Creator, 1);
        assert_size!(Senders, 1);
        assert_size!(Disposition, 1);
        assert_size!(ContentId, 12);
        assert_size!(Content, 252);
        assert_size!(Reason, 1);
        assert_size!(ReasonElement, 16);
        assert_size!(SessionId, 12);
        assert_size!(Jingle, 152);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Action, 1);
        assert_size!(Creator, 1);
        assert_size!(Senders, 1);
        assert_size!(Disposition, 1);
        assert_size!(ContentId, 24);
        assert_size!(Content, 504);
        assert_size!(Reason, 1);
        assert_size!(ReasonElement, 32);
        assert_size!(SessionId, 24);
        assert_size!(Jingle, 304);
    }

    #[test]
    fn test_simple() {
        let elem: Element =
            "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'/>"
                .parse()
                .unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle.action, Action::SessionInitiate);
        assert_eq!(jingle.sid, SessionId(String::from("coucou")));
    }

    #[test]
    fn test_invalid_jingle() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1'/>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'action' missing.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-info'/>"
            .parse()
            .unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='coucou' sid='coucou'/>"
            .parse()
            .unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'action' attribute.");
    }

    #[test]
    fn test_content() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou'><description/><transport xmlns='urn:xmpp:jingle:transports:stub:0'/></content></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle.contents[0].creator, Creator::Initiator);
        assert_eq!(jingle.contents[0].name, ContentId(String::from("coucou")));
        assert_eq!(jingle.contents[0].senders, Senders::Both);
        assert_eq!(jingle.contents[0].disposition, Disposition::Session);

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' senders='both'><description/><transport xmlns='urn:xmpp:jingle:transports:stub:0'/></content></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle.contents[0].senders, Senders::Both);

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' disposition='early-session'><description/><transport xmlns='urn:xmpp:jingle:transports:stub:0'/></content></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        assert_eq!(jingle.contents[0].disposition, Disposition::EarlySession);
    }

    #[test]
    fn test_invalid_content() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'creator' missing.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator'/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'name' missing.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='coucou' name='coucou'/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'creator' attribute.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' senders='coucou'/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'senders' attribute.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><content creator='initiator' name='coucou' senders=''/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'senders' attribute.");
    }

    #[test]
    fn test_reason() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><success/></reason></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        let reason = jingle.reason.unwrap();
        assert_eq!(reason.reason, Reason::Success);
        assert_eq!(reason.texts, BTreeMap::new());

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><success/><text>coucou</text></reason></jingle>".parse().unwrap();
        let jingle = Jingle::try_from(elem).unwrap();
        let reason = jingle.reason.unwrap();
        assert_eq!(reason.reason, Reason::Success);
        assert_eq!(reason.texts.get(""), Some(&String::from("coucou")));
    }

    #[test]
    fn test_invalid_reason() {
        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Reason doesn’t contain a valid reason.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><a/></reason></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown reason.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><a xmlns='http://www.w3.org/1999/xhtml'/></reason></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Reason contains a foreign element.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><decline/></reason><reason/></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Jingle must not have more than one reason.");

        let elem: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='coucou'><reason><decline/><text/><text/></reason></jingle>".parse().unwrap();
        let error = Jingle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Text element present twice for the same xml:lang.");
    }

    #[test]
    fn test_serialize_jingle() {
        let reference: Element = "<jingle xmlns='urn:xmpp:jingle:1' action='session-initiate' sid='a73sjjvkla37jfea'><content xmlns='urn:xmpp:jingle:1' creator='initiator' name='this-is-a-stub'><description xmlns='urn:xmpp:jingle:apps:stub:0'/><transport xmlns='urn:xmpp:jingle:transports:stub:0'/></content></jingle>"
        .parse()
        .unwrap();

        let jingle = Jingle {
            action: Action::SessionInitiate,
            initiator: None,
            responder: None,
            sid: SessionId(String::from("a73sjjvkla37jfea")),
            contents: vec![Content {
                creator: Creator::Initiator,
                disposition: Disposition::default(),
                name: ContentId(String::from("this-is-a-stub")),
                senders: Senders::default(),
                description: Some(Description::Unknown(
                    Element::builder("description", "urn:xmpp:jingle:apps:stub:0").build(),
                )),
                transport: Some(Transport::Unknown(
                    Element::builder("transport", "urn:xmpp:jingle:transports:stub:0").build(),
                )),
                security: None,
            }],
            reason: None,
            group: None,
            other: vec![],
        };
        let serialized: Element = jingle.into();
        assert_eq!(serialized, reference);
    }
}
