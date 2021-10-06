// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ns;
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use std::collections::BTreeMap;
use std::convert::TryFrom;

/// Should be implemented on every known payload of a `<message/>`.
pub trait MessagePayload: TryFrom<Element> + Into<Element> {}

generate_attribute!(
    /// The type of a message.
    MessageType, "type", {
        /// Standard instant messaging message.
        Chat => "chat",

        /// Notifies that an error happened.
        Error => "error",

        /// Standard group instant messaging message.
        Groupchat => "groupchat",

        /// Used by servers to notify users when things happen.
        Headline => "headline",

        /// This is an email-like message, it usually contains a
        /// [subject](struct.Subject.html).
        Normal => "normal",
    }, Default = Normal
);

type Lang = String;

generate_elem_id!(
    /// Represents one `<body/>` element, that is the free form text content of
    /// a message.
    Body,
    "body",
    DEFAULT_NS
);

generate_elem_id!(
    /// Defines the subject of a room, or of an email-like normal message.
    Subject,
    "subject",
    DEFAULT_NS
);

generate_elem_id!(
    /// A thread identifier, so that other people can specify to which message
    /// they are replying.
    Thread,
    "thread",
    DEFAULT_NS
);

/// The main structure representing the `<message/>` stanza.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// The JID emitting this stanza.
    pub from: Option<Jid>,

    /// The recipient of this stanza.
    pub to: Option<Jid>,

    /// The @id attribute of this stanza, which is required in order to match a
    /// request with its response.
    pub id: Option<String>,

    /// The type of this message.
    pub type_: MessageType,

    /// A list of bodies, sorted per language.  Use
    /// [get_best_body()](#method.get_best_body) to access them on reception.
    pub bodies: BTreeMap<Lang, Body>,

    /// A list of subjects, sorted per language.  Use
    /// [get_best_subject()](#method.get_best_subject) to access them on
    /// reception.
    pub subjects: BTreeMap<Lang, Subject>,

    /// An optional thread identifier, so that other people can reply directly
    /// to this message.
    pub thread: Option<Thread>,

    /// A list of the extension payloads contained in this stanza.
    pub payloads: Vec<Element>,
}

impl Message {
    /// Creates a new `<message/>` stanza for the given recipient.
    pub fn new<J: Into<Option<Jid>>>(to: J) -> Message {
        Message {
            from: None,
            to: to.into(),
            id: None,
            type_: MessageType::Chat,
            bodies: BTreeMap::new(),
            subjects: BTreeMap::new(),
            thread: None,
            payloads: vec![],
        }
    }

    fn get_best<'a, T>(
        map: &'a BTreeMap<Lang, T>,
        preferred_langs: Vec<&str>,
    ) -> Option<(Lang, &'a T)> {
        if map.is_empty() {
            return None;
        }
        for lang in preferred_langs {
            if let Some(value) = map.get(lang) {
                return Some((Lang::from(lang), value));
            }
        }
        if let Some(value) = map.get("") {
            return Some((Lang::new(), value));
        }
        map.iter().map(|(lang, value)| (lang.clone(), value)).next()
    }

    /// Returns the best matching body from a list of languages.
    ///
    /// For instance, if a message contains both an xml:lang='de', an xml:lang='fr' and an English
    /// body without an xml:lang attribute, and you pass ["fr", "en"] as your preferred languages,
    /// `Some(("fr", the_second_body))` will be returned.
    ///
    /// If no body matches, an undefined body will be returned.
    pub fn get_best_body(&self, preferred_langs: Vec<&str>) -> Option<(Lang, &Body)> {
        Message::get_best::<Body>(&self.bodies, preferred_langs)
    }

    /// Returns the best matching subject from a list of languages.
    ///
    /// For instance, if a message contains both an xml:lang='de', an xml:lang='fr' and an English
    /// subject without an xml:lang attribute, and you pass ["fr", "en"] as your preferred
    /// languages, `Some(("fr", the_second_subject))` will be returned.
    ///
    /// If no subject matches, an undefined subject will be returned.
    pub fn get_best_subject(&self, preferred_langs: Vec<&str>) -> Option<(Lang, &Subject)> {
        Message::get_best::<Subject>(&self.subjects, preferred_langs)
    }
}

impl TryFrom<Element> for Message {
    type Error = Error;

    fn try_from(root: Element) -> Result<Message, Error> {
        check_self!(root, "message", DEFAULT_NS);
        let from = get_attr!(root, "from", Option);
        let to = get_attr!(root, "to", Option);
        let id = get_attr!(root, "id", Option);
        let type_ = get_attr!(root, "type", Default);
        let mut bodies = BTreeMap::new();
        let mut subjects = BTreeMap::new();
        let mut thread = None;
        let mut payloads = vec![];
        for elem in root.children() {
            if elem.is("body", ns::DEFAULT_NS) {
                check_no_children!(elem, "body");
                let lang = get_attr!(elem, "xml:lang", Default);
                let body = Body(elem.text());
                if bodies.insert(lang, body).is_some() {
                    return Err(Error::ParseError(
                        "Body element present twice for the same xml:lang.",
                    ));
                }
            } else if elem.is("subject", ns::DEFAULT_NS) {
                check_no_children!(elem, "subject");
                let lang = get_attr!(elem, "xml:lang", Default);
                let subject = Subject(elem.text());
                if subjects.insert(lang, subject).is_some() {
                    return Err(Error::ParseError(
                        "Subject element present twice for the same xml:lang.",
                    ));
                }
            } else if elem.is("thread", ns::DEFAULT_NS) {
                if thread.is_some() {
                    return Err(Error::ParseError("Thread element present twice."));
                }
                check_no_children!(elem, "thread");
                thread = Some(Thread(elem.text()));
            } else {
                payloads.push(elem.clone())
            }
        }
        Ok(Message {
            from,
            to,
            id,
            type_,
            bodies,
            subjects,
            thread,
            payloads,
        })
    }
}

impl From<Message> for Element {
    fn from(message: Message) -> Element {
        Element::builder("message", ns::DEFAULT_NS)
            .attr("from", message.from)
            .attr("to", message.to)
            .attr("id", message.id)
            .attr("type", message.type_)
            .append_all(message.subjects.into_iter().map(|(lang, subject)| {
                let mut subject = Element::from(subject);
                subject.set_attr(
                    "xml:lang",
                    match lang.as_ref() {
                        "" => None,
                        lang => Some(lang),
                    },
                );
                subject
            }))
            .append_all(message.bodies.into_iter().map(|(lang, body)| {
                let mut body = Element::from(body);
                body.set_attr(
                    "xml:lang",
                    match lang.as_ref() {
                        "" => None,
                        lang => Some(lang),
                    },
                );
                body
            }))
            .append_all(message.payloads.into_iter())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jid::BareJid;
    use std::str::FromStr;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(MessageType, 1);
        assert_size!(Body, 12);
        assert_size!(Subject, 12);
        assert_size!(Thread, 12);
        assert_size!(Message, 144);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(MessageType, 1);
        assert_size!(Body, 24);
        assert_size!(Subject, 24);
        assert_size!(Thread, 24);
        assert_size!(Message, 288);
    }

    #[test]
    fn test_simple() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept'/>"
            .parse()
            .unwrap();
        let message = Message::try_from(elem).unwrap();
        assert_eq!(message.from, None);
        assert_eq!(message.to, None);
        assert_eq!(message.id, None);
        assert_eq!(message.type_, MessageType::Normal);
        assert!(message.payloads.is_empty());
    }

    #[test]
    fn test_serialise() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept'/>"
            .parse()
            .unwrap();
        let mut message = Message::new(None);
        message.type_ = MessageType::Normal;
        let elem2 = message.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_body() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let elem1 = elem.clone();
        let message = Message::try_from(elem).unwrap();
        assert_eq!(message.bodies[""], Body::from_str("Hello world!").unwrap());

        {
            let (lang, body) = message.get_best_body(vec!["en"]).unwrap();
            assert_eq!(lang, "");
            assert_eq!(body, &Body::from_str("Hello world!").unwrap());
        }

        let elem2 = message.into();
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_serialise_body() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let mut message = Message::new(Jid::Bare(BareJid::new("coucou", "example.org")));
        message
            .bodies
            .insert(String::from(""), Body::from_str("Hello world!").unwrap());
        let elem2 = message.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_subject() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><subject>Hello world!</subject></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><subject>Hello world!</subject></message>".parse().unwrap();
        let elem1 = elem.clone();
        let message = Message::try_from(elem).unwrap();
        assert_eq!(
            message.subjects[""],
            Subject::from_str("Hello world!").unwrap()
        );

        {
            let (lang, subject) = message.get_best_subject(vec!["en"]).unwrap();
            assert_eq!(lang, "");
            assert_eq!(subject, &Subject::from_str("Hello world!").unwrap());
        }

        let elem2 = message.into();
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn get_best_body() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><body xml:lang='de'>Hallo Welt!</body><body xml:lang='fr'>Salut le monde !</body><body>Hello world!</body></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><body>Hello world!</body></message>".parse().unwrap();
        let message = Message::try_from(elem).unwrap();

        // Tests basic feature.
        {
            let (lang, body) = message.get_best_body(vec!["fr"]).unwrap();
            assert_eq!(lang, "fr");
            assert_eq!(body, &Body::from_str("Salut le monde !").unwrap());
        }

        // Tests order.
        {
            let (lang, body) = message.get_best_body(vec!["en", "de"]).unwrap();
            assert_eq!(lang, "de");
            assert_eq!(body, &Body::from_str("Hallo Welt!").unwrap());
        }

        // Tests fallback.
        {
            let (lang, body) = message.get_best_body(vec![]).unwrap();
            assert_eq!(lang, "");
            assert_eq!(body, &Body::from_str("Hello world!").unwrap());
        }

        // Tests fallback.
        {
            let (lang, body) = message.get_best_body(vec!["ja"]).unwrap();
            assert_eq!(lang, "");
            assert_eq!(body, &Body::from_str("Hello world!").unwrap());
        }

        let message = Message::new(None);

        // Tests without a body.
        assert_eq!(message.get_best_body(vec!("ja")), None);
    }

    #[test]
    fn test_attention() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<message xmlns='jabber:client' to='coucou@example.org' type='chat'><attention xmlns='urn:xmpp:attention:0'/></message>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<message xmlns='jabber:component:accept' to='coucou@example.org' type='chat'><attention xmlns='urn:xmpp:attention:0'/></message>".parse().unwrap();
        let elem1 = elem.clone();
        let message = Message::try_from(elem).unwrap();
        let elem2 = message.into();
        assert_eq!(elem1, elem2);
    }
}
