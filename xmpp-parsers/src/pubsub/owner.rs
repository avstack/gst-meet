// Copyright (c) 2020 Paul Fariello <paul@fariello.eu>
// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::data_forms::DataForm;
use crate::iq::{IqGetPayload, IqResultPayload, IqSetPayload};
use crate::ns;
use crate::pubsub::{AffiliationAttribute, NodeName, Subscription};
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use std::convert::TryFrom;

generate_element!(
    /// A list of affiliations you have on a service, or on a node.
    Affiliations, "affiliations", PUBSUB_OWNER,
    attributes: [
        /// The node name this request pertains to.
        node: Required<NodeName> = "node",
    ],
    children: [
        /// The actual list of affiliation elements.
        affiliations: Vec<Affiliation> = ("affiliation", PUBSUB_OWNER) => Affiliation
    ]
);

generate_element!(
    /// An affiliation element.
    Affiliation, "affiliation", PUBSUB_OWNER,
    attributes: [
        /// The node this affiliation pertains to.
        jid: Required<Jid> = "jid",

        /// The affiliation you currently have on this node.
        affiliation: Required<AffiliationAttribute> = "affiliation",
    ]
);

generate_element!(
    /// Request to configure a node.
    Configure, "configure", PUBSUB_OWNER,
    attributes: [
        /// The node to be configured.
        node: Option<NodeName> = "node",
    ],
    children: [
        /// The form to configure it.
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    ]
);

generate_element!(
    /// Request to change default configuration.
    Default, "default", PUBSUB_OWNER,
    children: [
        /// The form to configure it.
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    ]
);

generate_element!(
    /// Request to delete a node.
    Delete, "delete", PUBSUB_OWNER,
    attributes: [
        /// The node to be configured.
        node: Required<NodeName> = "node",
    ],
    children: [
        /// Redirection to replace the deleted node.
        redirect: Option<Redirect> = ("redirect", PUBSUB_OWNER) => Redirect
    ]
);

generate_element!(
    /// A redirect element.
    Redirect, "redirect", PUBSUB_OWNER,
    attributes: [
        /// The node this node will be redirected to.
        uri: Required<String> = "uri",
    ]
);

generate_element!(
    /// Request to delete a node.
    Purge, "purge", PUBSUB_OWNER,
    attributes: [
        /// The node to be configured.
        node: Required<NodeName> = "node",
    ]
);

generate_element!(
    /// A request for current subscriptions.
    Subscriptions, "subscriptions", PUBSUB_OWNER,
    attributes: [
        /// The node to query.
        node: Required<NodeName> = "node",
    ],
    children: [
        /// The list of subscription elements returned.
        subscriptions: Vec<SubscriptionElem> = ("subscription", PUBSUB_OWNER) => SubscriptionElem
    ]
);

generate_element!(
    /// A subscription element, describing the state of a subscription.
    SubscriptionElem, "subscription", PUBSUB_OWNER,
    attributes: [
        /// The JID affected by this subscription.
        jid: Required<Jid> = "jid",

        /// The state of the subscription.
        subscription: Required<Subscription> = "subscription",

        /// Subscription unique id.
        subid: Option<String> = "subid",
    ]
);

/// Main payload used to communicate with a PubSubOwner service.
///
/// `<pubsub xmlns="http://jabber.org/protocol/pubsub#owner"/>`
#[derive(Debug, Clone)]
pub enum PubSubOwner {
    /// Manage the affiliations of a node.
    Affiliations(Affiliations),
    /// Request to configure a node, with optional suggested name and suggested configuration.
    Configure(Configure),
    /// Request the default node configuration.
    Default(Default),
    /// Delete a node.
    Delete(Delete),
    /// Purge all items from node.
    Purge(Purge),
    /// Request subscriptions of a node.
    Subscriptions(Subscriptions),
}

impl IqGetPayload for PubSubOwner {}
impl IqSetPayload for PubSubOwner {}
impl IqResultPayload for PubSubOwner {}

impl TryFrom<Element> for PubSubOwner {
    type Error = Error;

    fn try_from(elem: Element) -> Result<PubSubOwner, Error> {
        check_self!(elem, "pubsub", PUBSUB_OWNER);
        check_no_attributes!(elem, "pubsub");

        let mut payload = None;
        for child in elem.children() {
            if child.is("configure", ns::PUBSUB_OWNER) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub owner element.",
                    ));
                }
                let configure = Configure::try_from(child.clone())?;
                payload = Some(PubSubOwner::Configure(configure));
            } else {
                return Err(Error::ParseError("Unknown child in pubsub element."));
            }
        }
        Ok(payload.ok_or(Error::ParseError("No payload in pubsub element."))?)
    }
}

impl From<PubSubOwner> for Element {
    fn from(pubsub: PubSubOwner) -> Element {
        Element::builder("pubsub", ns::PUBSUB_OWNER)
            .append_all(match pubsub {
                PubSubOwner::Affiliations(affiliations) => vec![Element::from(affiliations)],
                PubSubOwner::Configure(configure) => vec![Element::from(configure)],
                PubSubOwner::Default(default) => vec![Element::from(default)],
                PubSubOwner::Delete(delete) => vec![Element::from(delete)],
                PubSubOwner::Purge(purge) => vec![Element::from(purge)],
                PubSubOwner::Subscriptions(subscriptions) => vec![Element::from(subscriptions)],
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_forms::{DataForm, DataFormType, Field, FieldType};
    use jid::BareJid;
    use std::str::FromStr;

    #[test]
    fn affiliations() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub#owner'><affiliations node='foo'><affiliation jid='hamlet@denmark.lit' affiliation='owner'/><affiliation jid='polonius@denmark.lit' affiliation='outcast'/></affiliations></pubsub>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();

        let pubsub = PubSubOwner::Affiliations(Affiliations {
            node: NodeName(String::from("foo")),
            affiliations: vec![
                Affiliation {
                    jid: Jid::Bare(BareJid::from_str("hamlet@denmark.lit").unwrap()),
                    affiliation: AffiliationAttribute::Owner,
                },
                Affiliation {
                    jid: Jid::Bare(BareJid::from_str("polonius@denmark.lit").unwrap()),
                    affiliation: AffiliationAttribute::Outcast,
                },
            ],
        });

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn configure() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub#owner'><configure node='foo'><x xmlns='jabber:x:data' type='submit'><field var='FORM_TYPE' type='hidden'><value>http://jabber.org/protocol/pubsub#node_config</value></field><field var='pubsub#access_model' type='list-single'><value>whitelist</value></field></x></configure></pubsub>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();

        let pubsub = PubSubOwner::Configure(Configure {
            node: Some(NodeName(String::from("foo"))),
            form: Some(DataForm {
                type_: DataFormType::Submit,
                form_type: Some(String::from(ns::PUBSUB_CONFIGURE)),
                title: None,
                instructions: None,
                fields: vec![Field {
                    var: String::from("pubsub#access_model"),
                    type_: FieldType::ListSingle,
                    label: None,
                    required: false,
                    options: vec![],
                    values: vec![String::from("whitelist")],
                    media: vec![],
                }],
            }),
        });

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_serialize_configure() {
        let reference: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub#owner'><configure node='foo'><x xmlns='jabber:x:data' type='submit'/></configure></pubsub>"
        .parse()
        .unwrap();

        let elem: Element = "<x xmlns='jabber:x:data' type='submit'/>".parse().unwrap();

        let form = DataForm::try_from(elem).unwrap();

        let configure = PubSubOwner::Configure(Configure {
            node: Some(NodeName(String::from("foo"))),
            form: Some(form),
        });
        let serialized: Element = configure.into();
        assert_eq!(serialized, reference);
    }

    #[test]
    fn default() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub#owner'><default><x xmlns='jabber:x:data' type='submit'><field var='FORM_TYPE' type='hidden'><value>http://jabber.org/protocol/pubsub#node_config</value></field><field var='pubsub#access_model' type='list-single'><value>whitelist</value></field></x></default></pubsub>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();

        let pubsub = PubSubOwner::Default(Default {
            form: Some(DataForm {
                type_: DataFormType::Submit,
                form_type: Some(String::from(ns::PUBSUB_CONFIGURE)),
                title: None,
                instructions: None,
                fields: vec![Field {
                    var: String::from("pubsub#access_model"),
                    type_: FieldType::ListSingle,
                    label: None,
                    required: false,
                    options: vec![],
                    values: vec![String::from("whitelist")],
                    media: vec![],
                }],
            }),
        });

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn delete() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub#owner'><delete node='foo'><redirect uri='xmpp:hamlet@denmark.lit?;node=blog'/></delete></pubsub>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();

        let pubsub = PubSubOwner::Delete(Delete {
            node: NodeName(String::from("foo")),
            redirect: Some(Redirect {
                uri: String::from("xmpp:hamlet@denmark.lit?;node=blog"),
            }),
        });

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn purge() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub#owner'><purge node='foo'></purge></pubsub>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();

        let pubsub = PubSubOwner::Purge(Purge {
            node: NodeName(String::from("foo")),
        });

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn subscriptions() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub#owner'><subscriptions node='foo'><subscription jid='hamlet@denmark.lit' subscription='subscribed'/><subscription jid='polonius@denmark.lit' subscription='unconfigured'/><subscription jid='bernardo@denmark.lit' subscription='subscribed' subid='123-abc'/><subscription jid='bernardo@denmark.lit' subscription='subscribed' subid='004-yyy'/></subscriptions></pubsub>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();

        let pubsub = PubSubOwner::Subscriptions(Subscriptions {
            node: NodeName(String::from("foo")),
            subscriptions: vec![
                SubscriptionElem {
                    jid: Jid::Bare(BareJid::from_str("hamlet@denmark.lit").unwrap()),
                    subscription: Subscription::Subscribed,
                    subid: None,
                },
                SubscriptionElem {
                    jid: Jid::Bare(BareJid::from_str("polonius@denmark.lit").unwrap()),
                    subscription: Subscription::Unconfigured,
                    subid: None,
                },
                SubscriptionElem {
                    jid: Jid::Bare(BareJid::from_str("bernardo@denmark.lit").unwrap()),
                    subscription: Subscription::Subscribed,
                    subid: Some(String::from("123-abc")),
                },
                SubscriptionElem {
                    jid: Jid::Bare(BareJid::from_str("bernardo@denmark.lit").unwrap()),
                    subscription: Subscription::Subscribed,
                    subid: Some(String::from("004-yyy")),
                },
            ],
        });

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }
}
