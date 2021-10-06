// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use crate::ns;
use crate::util::error::Error;
use crate::Element;
use std::convert::TryFrom;

generate_attribute!(
    /// Whether a conference bookmark should be joined automatically.
    Autojoin,
    "autojoin",
    bool
);

/// A conference bookmark.
#[derive(Debug, Clone)]
pub struct Conference {
    /// Whether a conference bookmark should be joined automatically.
    pub autojoin: Autojoin,

    /// A user-defined name for this conference.
    pub name: Option<String>,

    /// The nick the user will use to join this conference.
    pub nick: Option<String>,

    /// The password required to join this conference.
    pub password: Option<String>,

    /// Extensions elements.
    pub extensions: Option<Vec<Element>>,
}

impl Conference {
    /// Create a new conference.
    pub fn new() -> Conference {
        Conference {
            autojoin: Autojoin::False,
            name: None,
            nick: None,
            password: None,
            extensions: None,
        }
    }
}

impl TryFrom<Element> for Conference {
    type Error = Error;

    fn try_from(root: Element) -> Result<Conference, Error> {
        check_self!(root, "conference", BOOKMARKS2, "Conference");
        check_no_unknown_attributes!(root, "Conference", ["autojoin", "name"]);

        let mut conference = Conference {
            autojoin: get_attr!(root, "autojoin", Default),
            name: get_attr!(root, "name", Option),
            nick: None,
            password: None,
            extensions: None,
        };

        for child in root.children().cloned() {
            if child.is("extensions", ns::BOOKMARKS2) {
                if conference.extensions.is_some() {
                    return Err(Error::ParseError(
                        "Conference must not have more than one extensions element.",
                    ));
                }
                conference.extensions = Some(child.children().cloned().collect());
            } else if child.is("nick", ns::BOOKMARKS2) {
                if conference.nick.is_some() {
                    return Err(Error::ParseError(
                        "Conference must not have more than one nick.",
                    ));
                }
                check_no_children!(child, "nick");
                check_no_attributes!(child, "nick");
                conference.nick = Some(child.text());
            } else if child.is("password", ns::BOOKMARKS2) {
                if conference.password.is_some() {
                    return Err(Error::ParseError(
                        "Conference must not have more than one password.",
                    ));
                }
                check_no_children!(child, "password");
                check_no_attributes!(child, "password");
                conference.password = Some(child.text());
            }
        }

        Ok(conference)
    }
}

impl From<Conference> for Element {
    fn from(conference: Conference) -> Element {
        Element::builder("conference", ns::BOOKMARKS2)
            .attr("autojoin", conference.autojoin)
            .attr("name", conference.name)
            .append_all(
                conference
                    .nick
                    .map(|nick| Element::builder("nick", ns::BOOKMARKS2).append(nick)),
            )
            .append_all(
                conference
                    .password
                    .map(|password| Element::builder("password", ns::BOOKMARKS2).append(password)),
            )
            .append_all(match conference.extensions {
                Some(extensions) => extensions,
                None => vec![],
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pubsub::pubsub::Item as PubSubItem;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Conference, 52);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Conference, 104);
    }

    #[test]
    fn simple() {
        let elem: Element = "<conference xmlns='urn:xmpp:bookmarks:1'/>"
            .parse()
            .unwrap();
        let elem1 = elem.clone();
        let conference = Conference::try_from(elem).unwrap();
        assert_eq!(conference.autojoin, Autojoin::False);
        assert_eq!(conference.name, None);
        assert_eq!(conference.nick, None);
        assert_eq!(conference.password, None);

        let elem2 = Element::from(Conference::new());
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn complete() {
        let elem: Element = "<conference xmlns='urn:xmpp:bookmarks:1' autojoin='true' name='Test MUC'><nick>Coucou</nick><password>secret</password><extensions><test xmlns='urn:xmpp:unknown' /></extensions></conference>".parse().unwrap();
        let conference = Conference::try_from(elem).unwrap();
        assert_eq!(conference.autojoin, Autojoin::True);
        assert_eq!(conference.name, Some(String::from("Test MUC")));
        assert_eq!(conference.clone().nick.unwrap(), "Coucou");
        assert_eq!(conference.clone().password.unwrap(), "secret");
        assert_eq!(conference.clone().extensions.unwrap().len(), 1);
        assert!(conference.clone().extensions.unwrap()[0].is("test", "urn:xmpp:unknown"));
    }

    #[test]
    fn wrapped() {
        let elem: Element = "<item xmlns='http://jabber.org/protocol/pubsub' id='test-muc@muc.localhost'><conference xmlns='urn:xmpp:bookmarks:1' autojoin='true' name='Test MUC'><nick>Coucou</nick><password>secret</password></conference></item>".parse().unwrap();
        let item = PubSubItem::try_from(elem).unwrap();
        let payload = item.payload.clone().unwrap();
        println!("FOO: payload: {:?}", payload);
        // let conference = Conference::try_from(payload).unwrap();
        let conference = Conference::try_from(payload);
        println!("FOO: conference: {:?}", conference);
        /*
        assert_eq!(conference.autojoin, Autojoin::True);
        assert_eq!(conference.name, Some(String::from("Test MUC")));
        assert_eq!(conference.clone().nick.unwrap(), "Coucou");
        assert_eq!(conference.clone().password.unwrap(), "secret");

        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='urn:xmpp:bookmarks:1'><item xmlns='http://jabber.org/protocol/pubsub#event' id='test-muc@muc.localhost'><conference xmlns='urn:xmpp:bookmarks:1' autojoin='true' name='Test MUC'><nick>Coucou</nick><password>secret</password></conference></item></items></event>".parse().unwrap();
        let mut items = match PubSubEvent::try_from(elem) {
            Ok(PubSubEvent::PublishedItems { node, items }) => {
                assert_eq!(&node.0, ns::BOOKMARKS2);
                items
            }
            _ => panic!(),
        };
        assert_eq!(items.len(), 1);
        let item = items.pop().unwrap();
        let payload = item.payload.clone().unwrap();
        let conference = Conference::try_from(payload).unwrap();
        assert_eq!(conference.autojoin, Autojoin::True);
        assert_eq!(conference.name, Some(String::from("Test MUC")));
        assert_eq!(conference.clone().nick.unwrap(), "Coucou");
        assert_eq!(conference.clone().password.unwrap(), "secret");
        */
    }
}
