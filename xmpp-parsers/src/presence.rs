// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ns;
use crate::util::error::Error;
use jid::Jid;
use minidom::{Element, IntoAttributeValue, Node};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::str::FromStr;

/// Should be implemented on every known payload of a `<presence/>`.
pub trait PresencePayload: TryFrom<Element> + Into<Element> {}

/// Specifies the availability of an entity or resource.
#[derive(Debug, Clone, PartialEq)]
pub enum Show {
    /// The entity or resource is temporarily away.
    Away,

    /// The entity or resource is actively interested in chatting.
    Chat,

    /// The entity or resource is busy (dnd = "Do Not Disturb").
    Dnd,

    /// The entity or resource is away for an extended period (xa = "eXtended
    /// Away").
    Xa,
}

impl FromStr for Show {
    type Err = Error;

    fn from_str(s: &str) -> Result<Show, Error> {
        Ok(match s {
            "away" => Show::Away,
            "chat" => Show::Chat,
            "dnd" => Show::Dnd,
            "xa" => Show::Xa,

            _ => return Err(Error::ParseError("Invalid value for show.")),
        })
    }
}

impl Into<Node> for Show {
    fn into(self) -> Node {
        Element::builder("show", ns::DEFAULT_NS)
            .append(match self {
                Show::Away => "away",
                Show::Chat => "chat",
                Show::Dnd => "dnd",
                Show::Xa => "xa",
            })
            .build()
            .into()
    }
}

type Lang = String;
type Status = String;

type Priority = i8;

///
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// This value is not an acceptable 'type' attribute, it is only used
    /// internally to signal the absence of 'type'.
    None,

    /// An error has occurred regarding processing of a previously sent
    /// presence stanza; if the presence stanza is of type "error", it MUST
    /// include an <error/> child element (refer to [XMPP‑CORE]).
    Error,

    /// A request for an entity's current presence; SHOULD be generated only by
    /// a server on behalf of a user.
    Probe,

    /// The sender wishes to subscribe to the recipient's presence.
    Subscribe,

    /// The sender has allowed the recipient to receive their presence.
    Subscribed,

    /// The sender is no longer available for communication.
    Unavailable,

    /// The sender is unsubscribing from the receiver's presence.
    Unsubscribe,

    /// The subscription request has been denied or a previously granted
    /// subscription has been canceled.
    Unsubscribed,
}

impl Default for Type {
    fn default() -> Type {
        Type::None
    }
}

impl FromStr for Type {
    type Err = Error;

    fn from_str(s: &str) -> Result<Type, Error> {
        Ok(match s {
            "error" => Type::Error,
            "probe" => Type::Probe,
            "subscribe" => Type::Subscribe,
            "subscribed" => Type::Subscribed,
            "unavailable" => Type::Unavailable,
            "unsubscribe" => Type::Unsubscribe,
            "unsubscribed" => Type::Unsubscribed,

            _ => {
                return Err(Error::ParseError(
                    "Invalid 'type' attribute on presence element.",
                ));
            }
        })
    }
}

impl IntoAttributeValue for Type {
    fn into_attribute_value(self) -> Option<String> {
        Some(
            match self {
                Type::None => return None,

                Type::Error => "error",
                Type::Probe => "probe",
                Type::Subscribe => "subscribe",
                Type::Subscribed => "subscribed",
                Type::Unavailable => "unavailable",
                Type::Unsubscribe => "unsubscribe",
                Type::Unsubscribed => "unsubscribed",
            }
            .to_owned(),
        )
    }
}

/// The main structure representing the `<presence/>` stanza.
#[derive(Debug, Clone)]
pub struct Presence {
    /// The sender of this presence.
    pub from: Option<Jid>,

    /// The recipient of this presence.
    pub to: Option<Jid>,

    /// The identifier, unique on this stream, of this stanza.
    pub id: Option<String>,

    /// The type of this presence stanza.
    pub type_: Type,

    /// The availability of the sender of this presence.
    pub show: Option<Show>,

    /// A localised list of statuses defined in this presence.
    pub statuses: BTreeMap<Lang, Status>,

    /// The sender’s resource priority, if negative it won’t receive messages
    /// that haven’t been directed to it.
    pub priority: Priority,

    /// A list of payloads contained in this presence.
    pub payloads: Vec<Element>,
}

impl Presence {
    /// Create a new presence of this type.
    pub fn new(type_: Type) -> Presence {
        Presence {
            from: None,
            to: None,
            id: None,
            type_,
            show: None,
            statuses: BTreeMap::new(),
            priority: 0i8,
            payloads: vec![],
        }
    }

    /// Set the emitter of this presence, this should only be useful for
    /// servers and components, as clients can only send presences from their
    /// own resource (which is implicit).
    pub fn with_from<J: Into<Jid>>(mut self, from: J) -> Presence {
        self.from = Some(from.into());
        self
    }

    /// Set the recipient of this presence, this is only useful for directed
    /// presences.
    pub fn with_to<J: Into<Jid>>(mut self, to: J) -> Presence {
        self.to = Some(to.into());
        self
    }

    /// Set the identifier for this presence.
    pub fn with_id(mut self, id: String) -> Presence {
        self.id = Some(id);
        self
    }

    /// Set the availability information of this presence.
    pub fn with_show(mut self, show: Show) -> Presence {
        self.show = Some(show);
        self
    }

    /// Set the priority of this presence.
    pub fn with_priority(mut self, priority: i8) -> Presence {
        self.priority = priority;
        self
    }

    /// Set the payloads of this presence.
    pub fn with_payloads(mut self, payloads: Vec<Element>) -> Presence {
        self.payloads = payloads;
        self
    }

    /// Set the availability information of this presence.
    pub fn set_status<L, S>(&mut self, lang: L, status: S)
    where
        L: Into<Lang>,
        S: Into<Status>,
    {
        self.statuses.insert(lang.into(), status.into());
    }

    /// Add a payload to this presence.
    pub fn add_payload<P: PresencePayload>(&mut self, payload: P) {
        self.payloads.push(payload.into());
    }
}

impl TryFrom<Element> for Presence {
    type Error = Error;

    fn try_from(root: Element) -> Result<Presence, Error> {
        check_self!(root, "presence", DEFAULT_NS);
        let mut show = None;
        let mut priority = None;
        let mut presence = Presence {
            from: get_attr!(root, "from", Option),
            to: get_attr!(root, "to", Option),
            id: get_attr!(root, "id", Option),
            type_: get_attr!(root, "type", Default),
            show: None,
            statuses: BTreeMap::new(),
            priority: 0i8,
            payloads: vec![],
        };
        for elem in root.children() {
            if elem.is("show", ns::DEFAULT_NS) {
                if show.is_some() {
                    return Err(Error::ParseError(
                        "More than one show element in a presence.",
                    ));
                }
                check_no_attributes!(elem, "show");
                check_no_children!(elem, "show");
                show = Some(Show::from_str(elem.text().as_ref())?);
            } else if elem.is("status", ns::DEFAULT_NS) {
                check_no_unknown_attributes!(elem, "status", ["xml:lang"]);
                check_no_children!(elem, "status");
                let lang = get_attr!(elem, "xml:lang", Default);
                if presence.statuses.insert(lang, elem.text()).is_some() {
                    return Err(Error::ParseError(
                        "Status element present twice for the same xml:lang.",
                    ));
                }
            } else if elem.is("priority", ns::DEFAULT_NS) {
                if priority.is_some() {
                    return Err(Error::ParseError(
                        "More than one priority element in a presence.",
                    ));
                }
                check_no_attributes!(elem, "priority");
                check_no_children!(elem, "priority");
                priority = Some(Priority::from_str(elem.text().as_ref())?);
            } else {
                presence.payloads.push(elem.clone());
            }
        }
        presence.show = show;
        if let Some(priority) = priority {
            presence.priority = priority;
        }
        Ok(presence)
    }
}

impl From<Presence> for Element {
    fn from(presence: Presence) -> Element {
        Element::builder("presence", ns::DEFAULT_NS)
            .attr("from", presence.from)
            .attr("to", presence.to)
            .attr("id", presence.id)
            .attr("type", presence.type_)
            .append_all(presence.show.into_iter())
            .append_all(presence.statuses.into_iter().map(|(lang, status)| {
                Element::builder("status", ns::DEFAULT_NS)
                    .attr(
                        "xml:lang",
                        match lang.as_ref() {
                            "" => None,
                            lang => Some(lang),
                        },
                    )
                    .append(status)
            }))
            .append_all(if presence.priority == 0 {
                None
            } else {
                Some(
                    Element::builder("priority", ns::DEFAULT_NS)
                        .append(format!("{}", presence.priority)),
                )
            })
            .append_all(presence.payloads.into_iter())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jid::{BareJid, FullJid};

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Show, 1);
        assert_size!(Type, 1);
        assert_size!(Presence, 120);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Show, 1);
        assert_size!(Type, 1);
        assert_size!(Presence, 240);
    }

    #[test]
    fn test_simple() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'/>"
            .parse()
            .unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.from, None);
        assert_eq!(presence.to, None);
        assert_eq!(presence.id, None);
        assert_eq!(presence.type_, Type::None);
        assert!(presence.payloads.is_empty());
    }

    #[test]
    fn test_serialise() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client' type='unavailable'/>/>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept' type='unavailable'/>/>"
            .parse()
            .unwrap();
        let presence = Presence::new(Type::Unavailable);
        let elem2 = presence.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_show() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><show>chat</show></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element =
            "<presence xmlns='jabber:component:accept'><show>chat</show></presence>"
                .parse()
                .unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.show, Some(Show::Chat));
    }

    #[test]
    fn test_empty_show_value() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'/>"
            .parse()
            .unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.show, None);
    }

    #[test]
    fn test_missing_show_value() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><show/></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><show/></presence>"
            .parse()
            .unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid value for show.");
    }

    #[test]
    fn test_invalid_show() {
        // "online" used to be a pretty common mistake.
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><show>online</show></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element =
            "<presence xmlns='jabber:component:accept'><show>online</show></presence>"
                .parse()
                .unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid value for show.");
    }

    #[test]
    fn test_empty_status() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status/></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><status/></presence>"
            .parse()
            .unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.statuses.len(), 1);
        assert_eq!(presence.statuses[""], "");
    }

    #[test]
    fn test_status() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status>Here!</status></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element =
            "<presence xmlns='jabber:component:accept'><status>Here!</status></presence>"
                .parse()
                .unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.statuses.len(), 1);
        assert_eq!(presence.statuses[""], "Here!");
    }

    #[test]
    fn test_multiple_statuses() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status>Here!</status><status xml:lang='fr'>Là!</status></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><status>Here!</status><status xml:lang='fr'>Là!</status></presence>".parse().unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.statuses.len(), 2);
        assert_eq!(presence.statuses[""], "Here!");
        assert_eq!(presence.statuses["fr"], "Là!");
    }

    #[test]
    fn test_invalid_multiple_statuses() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status xml:lang='fr'>Here!</status><status xml:lang='fr'>Là!</status></presence>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<presence xmlns='jabber:component:accept'><status xml:lang='fr'>Here!</status><status xml:lang='fr'>Là!</status></presence>".parse().unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(
            message,
            "Status element present twice for the same xml:lang."
        );
    }

    #[test]
    fn test_priority() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><priority>-1</priority></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element =
            "<presence xmlns='jabber:component:accept'><priority>-1</priority></presence>"
                .parse()
                .unwrap();
        let presence = Presence::try_from(elem).unwrap();
        assert_eq!(presence.payloads.len(), 0);
        assert_eq!(presence.priority, -1i8);
    }

    #[test]
    fn test_invalid_priority() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><priority>128</priority></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element =
            "<presence xmlns='jabber:component:accept'><priority>128</priority></presence>"
                .parse()
                .unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        match error {
            Error::ParseIntError(_) => (),
            _ => panic!(),
        };
    }

    #[test]
    fn test_unknown_child() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><test xmlns='invalid'/></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element =
            "<presence xmlns='jabber:component:accept'><test xmlns='invalid'/></presence>"
                .parse()
                .unwrap();
        let presence = Presence::try_from(elem).unwrap();
        let payload = &presence.payloads[0];
        assert!(payload.is("test", "invalid"));
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_status_child() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status><coucou/></status></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element =
            "<presence xmlns='jabber:component:accept'><status><coucou/></status></presence>"
                .parse()
                .unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in status element.");
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_attribute() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<presence xmlns='jabber:client'><status coucou=''/></presence>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element =
            "<presence xmlns='jabber:component:accept'><status coucou=''/></presence>"
                .parse()
                .unwrap();
        let error = Presence::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in status element.");
    }

    #[test]
    fn test_serialise_status() {
        let status = Status::from("Hello world!");
        let mut presence = Presence::new(Type::Unavailable);
        presence.statuses.insert(String::from(""), status);
        let elem: Element = presence.into();
        assert!(elem.is("presence", ns::DEFAULT_NS));
        assert!(elem.children().next().unwrap().is("status", ns::DEFAULT_NS));
    }

    #[test]
    fn test_serialise_priority() {
        let presence = Presence::new(Type::None).with_priority(42);
        let elem: Element = presence.into();
        assert!(elem.is("presence", ns::DEFAULT_NS));
        let priority = elem.children().next().unwrap();
        assert!(priority.is("priority", ns::DEFAULT_NS));
        assert_eq!(priority.text(), "42");
    }

    #[test]
    fn presence_with_to() {
        let presence = Presence::new(Type::None);
        let elem: Element = presence.into();
        assert_eq!(elem.attr("to"), None);

        let presence = Presence::new(Type::None).with_to(Jid::Bare(BareJid::domain("localhost")));
        let elem: Element = presence.into();
        assert_eq!(elem.attr("to"), Some("localhost"));

        let presence = Presence::new(Type::None).with_to(BareJid::domain("localhost"));
        let elem: Element = presence.into();
        assert_eq!(elem.attr("to"), Some("localhost"));

        let presence = Presence::new(Type::None).with_to(Jid::Full(FullJid::new(
            "test",
            "localhost",
            "coucou",
        )));
        let elem: Element = presence.into();
        assert_eq!(elem.attr("to"), Some("test@localhost/coucou"));

        let presence =
            Presence::new(Type::None).with_to(FullJid::new("test", "localhost", "coucou"));
        let elem: Element = presence.into();
        assert_eq!(elem.attr("to"), Some("test@localhost/coucou"));
    }
}
