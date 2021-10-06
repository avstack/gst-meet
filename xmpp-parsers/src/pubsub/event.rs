// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::data_forms::DataForm;
use crate::date::DateTime;
use crate::message::MessagePayload;
use crate::ns;
use crate::pubsub::{Item as PubSubItem, ItemId, NodeName, Subscription, SubscriptionId};
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use std::convert::TryFrom;

/// Event wrapper for a PubSub `<item/>`.
#[derive(Debug, Clone)]
pub struct Item(pub PubSubItem);

impl_pubsub_item!(Item, PUBSUB_EVENT);

/// Represents an event happening to a PubSub node.
#[derive(Debug, Clone)]
pub enum PubSubEvent {
    /*
    Collection {
    },
    */
    /// This node’s configuration changed.
    Configuration {
        /// The node affected.
        node: NodeName,

        /// The new configuration of this node.
        form: Option<DataForm>,
    },

    /// This node has been deleted, with an optional redirect to another node.
    Delete {
        /// The node affected.
        node: NodeName,

        /// The xmpp: URI of another node replacing this one.
        redirect: Option<String>,
    },

    /// Some items have been published on this node.
    PublishedItems {
        /// The node affected.
        node: NodeName,

        /// The list of published items.
        items: Vec<Item>,
    },

    /// Some items have been removed from this node.
    RetractedItems {
        /// The node affected.
        node: NodeName,

        /// The list of retracted items.
        items: Vec<ItemId>,
    },

    /// All items of this node just got removed at once.
    Purge {
        /// The node affected.
        node: NodeName,
    },

    /// The user’s subscription to this node has changed.
    Subscription {
        /// The node affected.
        node: NodeName,

        /// The time at which this subscription will expire.
        expiry: Option<DateTime>,

        /// The JID of the user affected.
        jid: Option<Jid>,

        /// An identifier for this subscription.
        subid: Option<SubscriptionId>,

        /// The state of this subscription.
        subscription: Option<Subscription>,
    },
}

fn parse_items(elem: Element, node: NodeName) -> Result<PubSubEvent, Error> {
    let mut is_retract = None;
    let mut items = vec![];
    let mut retracts = vec![];
    for child in elem.children() {
        if child.is("item", ns::PUBSUB_EVENT) {
            match is_retract {
                None => is_retract = Some(false),
                Some(false) => (),
                Some(true) => {
                    return Err(Error::ParseError(
                        "Mix of item and retract in items element.",
                    ));
                }
            }
            items.push(Item::try_from(child.clone())?);
        } else if child.is("retract", ns::PUBSUB_EVENT) {
            match is_retract {
                None => is_retract = Some(true),
                Some(true) => (),
                Some(false) => {
                    return Err(Error::ParseError(
                        "Mix of item and retract in items element.",
                    ));
                }
            }
            check_no_children!(child, "retract");
            check_no_unknown_attributes!(child, "retract", ["id"]);
            let id = get_attr!(child, "id", Required);
            retracts.push(id);
        } else {
            return Err(Error::ParseError("Invalid child in items element."));
        }
    }
    Ok(match is_retract {
        Some(false) => PubSubEvent::PublishedItems { node, items },
        Some(true) => PubSubEvent::RetractedItems {
            node,
            items: retracts,
        },
        None => return Err(Error::ParseError("Missing children in items element.")),
    })
}

impl TryFrom<Element> for PubSubEvent {
    type Error = Error;

    fn try_from(elem: Element) -> Result<PubSubEvent, Error> {
        check_self!(elem, "event", PUBSUB_EVENT);
        check_no_attributes!(elem, "event");

        let mut payload = None;
        for child in elem.children() {
            let node = get_attr!(child, "node", Required);
            if child.is("configuration", ns::PUBSUB_EVENT) {
                let mut payloads = child.children().cloned().collect::<Vec<_>>();
                let item = payloads.pop();
                if !payloads.is_empty() {
                    return Err(Error::ParseError(
                        "More than a single payload in configuration element.",
                    ));
                }
                let form = match item {
                    None => None,
                    Some(payload) => Some(DataForm::try_from(payload)?),
                };
                payload = Some(PubSubEvent::Configuration { node, form });
            } else if child.is("delete", ns::PUBSUB_EVENT) {
                let mut redirect = None;
                for item in child.children() {
                    if item.is("redirect", ns::PUBSUB_EVENT) {
                        if redirect.is_some() {
                            return Err(Error::ParseError(
                                "More than one redirect in delete element.",
                            ));
                        }
                        let uri = get_attr!(item, "uri", Required);
                        redirect = Some(uri);
                    } else {
                        return Err(Error::ParseError("Unknown child in delete element."));
                    }
                }
                payload = Some(PubSubEvent::Delete { node, redirect });
            } else if child.is("items", ns::PUBSUB_EVENT) {
                payload = Some(parse_items(child.clone(), node)?);
            } else if child.is("purge", ns::PUBSUB_EVENT) {
                check_no_children!(child, "purge");
                payload = Some(PubSubEvent::Purge { node });
            } else if child.is("subscription", ns::PUBSUB_EVENT) {
                check_no_children!(child, "subscription");
                payload = Some(PubSubEvent::Subscription {
                    node,
                    expiry: get_attr!(child, "expiry", Option),
                    jid: get_attr!(child, "jid", Option),
                    subid: get_attr!(child, "subid", Option),
                    subscription: get_attr!(child, "subscription", Option),
                });
            } else {
                return Err(Error::ParseError("Unknown child in event element."));
            }
        }
        Ok(payload.ok_or(Error::ParseError("No payload in event element."))?)
    }
}

impl From<PubSubEvent> for Element {
    fn from(event: PubSubEvent) -> Element {
        let payload = match event {
            PubSubEvent::Configuration { node, form } => {
                Element::builder("configuration", ns::PUBSUB_EVENT)
                    .attr("node", node)
                    .append_all(form.map(Element::from))
            }
            PubSubEvent::Delete { node, redirect } => Element::builder("purge", ns::PUBSUB_EVENT)
                .attr("node", node)
                .append_all(redirect.map(|redirect| {
                    Element::builder("redirect", ns::PUBSUB_EVENT).attr("uri", redirect)
                })),
            PubSubEvent::PublishedItems { node, items } => {
                Element::builder("items", ns::PUBSUB_EVENT)
                    .attr("node", node)
                    .append_all(items.into_iter())
            }
            PubSubEvent::RetractedItems { node, items } => {
                Element::builder("items", ns::PUBSUB_EVENT)
                    .attr("node", node)
                    .append_all(
                        items
                            .into_iter()
                            .map(|id| Element::builder("retract", ns::PUBSUB_EVENT).attr("id", id)),
                    )
            }
            PubSubEvent::Purge { node } => {
                Element::builder("purge", ns::PUBSUB_EVENT).attr("node", node)
            }
            PubSubEvent::Subscription {
                node,
                expiry,
                jid,
                subid,
                subscription,
            } => Element::builder("subscription", ns::PUBSUB_EVENT)
                .attr("node", node)
                .attr("expiry", expiry)
                .attr("jid", jid)
                .attr("subid", subid)
                .attr("subscription", subscription),
        };
        Element::builder("event", ns::PUBSUB_EVENT)
            .append(payload)
            .build()
    }
}

impl MessagePayload for PubSubEvent {}

#[cfg(test)]
mod tests {
    use super::*;
    use jid::BareJid;

    #[test]
    fn missing_items() {
        let elem: Element =
            "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='coucou'/></event>"
                .parse()
                .unwrap();
        let error = PubSubEvent::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Missing children in items element.");
    }

    #[test]
    fn test_simple_items() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='coucou'><item id='test' publisher='test@coucou'/></items></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::PublishedItems { node, items } => {
                assert_eq!(node, NodeName(String::from("coucou")));
                assert_eq!(items[0].id, Some(ItemId(String::from("test"))));
                assert_eq!(
                    items[0].publisher.clone().unwrap(),
                    BareJid::new("test", "coucou")
                );
                assert_eq!(items[0].payload, None);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_pep() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='something'><item><foreign xmlns='example:namespace'/></item></items></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::PublishedItems { node, items } => {
                assert_eq!(node, NodeName(String::from("something")));
                assert_eq!(items[0].id, None);
                assert_eq!(items[0].publisher, None);
                match items[0].payload {
                    Some(ref elem) => assert!(elem.is("foreign", "example:namespace")),
                    _ => panic!(),
                }
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_retract() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><items node='something'><retract id='coucou'/><retract id='test'/></items></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::RetractedItems { node, items } => {
                assert_eq!(node, NodeName(String::from("something")));
                assert_eq!(items[0], ItemId(String::from("coucou")));
                assert_eq!(items[1], ItemId(String::from("test")));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_delete() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><delete node='coucou'><redirect uri='hello'/></delete></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::Delete { node, redirect } => {
                assert_eq!(node, NodeName(String::from("coucou")));
                assert_eq!(redirect, Some(String::from("hello")));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_purge() {
        let elem: Element =
            "<event xmlns='http://jabber.org/protocol/pubsub#event'><purge node='coucou'/></event>"
                .parse()
                .unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::Purge { node } => {
                assert_eq!(node, NodeName(String::from("coucou")));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_simple_configure() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><configuration node='coucou'><x xmlns='jabber:x:data' type='result'><field var='FORM_TYPE' type='hidden'><value>http://jabber.org/protocol/pubsub#node_config</value></field></x></configuration></event>".parse().unwrap();
        let event = PubSubEvent::try_from(elem).unwrap();
        match event {
            PubSubEvent::Configuration { node, form: _ } => {
                assert_eq!(node, NodeName(String::from("coucou")));
                //assert_eq!(form.type_, Result_);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_invalid() {
        let elem: Element =
            "<event xmlns='http://jabber.org/protocol/pubsub#event'><coucou node='test'/></event>"
                .parse()
                .unwrap();
        let error = PubSubEvent::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in event element.");
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event' coucou=''/>"
            .parse()
            .unwrap();
        let error = PubSubEvent::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in event element.");
    }

    #[test]
    fn test_ex221_subscription() {
        let elem: Element = "<event xmlns='http://jabber.org/protocol/pubsub#event'><subscription expiry='2006-02-28T23:59:59+00:00' jid='francisco@denmark.lit' node='princely_musings' subid='ba49252aaa4f5d320c24d3766f0bdcade78c78d3' subscription='subscribed'/></event>"
        .parse()
        .unwrap();
        let event = PubSubEvent::try_from(elem.clone()).unwrap();
        match event.clone() {
            PubSubEvent::Subscription {
                node,
                expiry,
                jid,
                subid,
                subscription,
            } => {
                assert_eq!(node, NodeName(String::from("princely_musings")));
                assert_eq!(
                    subid,
                    Some(SubscriptionId(String::from(
                        "ba49252aaa4f5d320c24d3766f0bdcade78c78d3"
                    )))
                );
                assert_eq!(subscription, Some(Subscription::Subscribed));
                assert_eq!(jid.unwrap(), BareJid::new("francisco", "denmark.lit"));
                assert_eq!(expiry, Some("2006-02-28T23:59:59Z".parse().unwrap()));
            }
            _ => panic!(),
        }

        let elem2: Element = event.into();
        assert_eq!(elem, elem2);
    }
}
