// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::data_forms::DataForm;
use crate::iq::{IqGetPayload, IqResultPayload, IqSetPayload};
use crate::ns;
use crate::pubsub::{
    AffiliationAttribute, Item as PubSubItem, NodeName, Subscription, SubscriptionId,
};
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use std::convert::TryFrom;

// TODO: a better solution would be to split this into a query and a result elements, like for
// XEP-0030.
generate_element!(
    /// A list of affiliations you have on a service, or on a node.
    Affiliations, "affiliations", PUBSUB,
    attributes: [
        /// The optional node name this request pertains to.
        node: Option<NodeName> = "node",
    ],
    children: [
        /// The actual list of affiliation elements.
        affiliations: Vec<Affiliation> = ("affiliation", PUBSUB) => Affiliation
    ]
);

generate_element!(
    /// An affiliation element.
    Affiliation, "affiliation", PUBSUB,
    attributes: [
        /// The node this affiliation pertains to.
        node: Required<NodeName> = "node",

        /// The affiliation you currently have on this node.
        affiliation: Required<AffiliationAttribute> = "affiliation",
    ]
);

generate_element!(
    /// Request to configure a new node.
    Configure, "configure", PUBSUB,
    children: [
        /// The form to configure it.
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    ]
);

generate_element!(
    /// Request to create a new node.
    Create, "create", PUBSUB,
    attributes: [
        /// The node name to create, if `None` the service will generate one.
        node: Option<NodeName> = "node",
    ]
);

generate_element!(
    /// Request for a default node configuration.
    Default, "default", PUBSUB,
    attributes: [
        /// The node targeted by this request, otherwise the entire service.
        node: Option<NodeName> = "node",

        // TODO: do we really want to support collection nodes?
        // type: Option<String> = "type",
    ]
);

generate_element!(
    /// A request for a list of items.
    Items, "items", PUBSUB,
    attributes: [
        // TODO: should be an xs:positiveInteger, that is, an unbounded int ≥ 1.
        /// Maximum number of items returned.
        max_items: Option<u32> = "max_items",

        /// The node queried by this request.
        node: Required<NodeName> = "node",

        /// The subscription identifier related to this request.
        subid: Option<SubscriptionId> = "subid",
    ],
    children: [
        /// The actual list of items returned.
        items: Vec<Item> = ("item", PUBSUB) => Item
    ]
);

impl Items {
    /// Create a new items request.
    pub fn new(node: &str) -> Items {
        Items {
            node: NodeName(String::from(node)),
            max_items: None,
            subid: None,
            items: Vec::new(),
        }
    }
}

/// Response wrapper for a PubSub `<item/>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Item(pub PubSubItem);

impl_pubsub_item!(Item, PUBSUB);

generate_element!(
    /// The options associated to a subscription request.
    Options, "options", PUBSUB,
    attributes: [
        /// The JID affected by this request.
        jid: Required<Jid> = "jid",

        /// The node affected by this request.
        node: Option<NodeName> = "node",

        /// The subscription identifier affected by this request.
        subid: Option<SubscriptionId> = "subid",
    ],
    children: [
        /// The form describing the subscription.
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    ]
);

generate_element!(
    /// Request to publish items to a node.
    Publish, "publish", PUBSUB,
    attributes: [
        /// The target node for this operation.
        node: Required<NodeName> = "node",
    ],
    children: [
        /// The items you want to publish.
        items: Vec<Item> = ("item", PUBSUB) => Item
    ]
);

generate_element!(
    /// The options associated to a publish request.
    PublishOptions, "publish-options", PUBSUB,
    children: [
        /// The form describing these options.
        form: Option<DataForm> = ("x", DATA_FORMS) => DataForm
    ]
);

generate_attribute!(
    /// Whether a retract request should notify subscribers or not.
    Notify,
    "notify",
    bool
);

generate_element!(
    /// A request to retract some items from a node.
    Retract, "retract", PUBSUB,
    attributes: [
        /// The node affected by this request.
        node: Required<NodeName> = "node",

        /// Whether a retract request should notify subscribers or not.
        notify: Default<Notify> = "notify",
    ],
    children: [
        /// The items affected by this request.
        items: Vec<Item> = ("item", PUBSUB) => Item
    ]
);

/// Indicate that the subscription can be configured.
#[derive(Debug, Clone, PartialEq)]
pub struct SubscribeOptions {
    /// If `true`, the configuration is actually required.
    required: bool,
}

impl TryFrom<Element> for SubscribeOptions {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Self, Error> {
        check_self!(elem, "subscribe-options", PUBSUB);
        check_no_attributes!(elem, "subscribe-options");
        let mut required = false;
        for child in elem.children() {
            if child.is("required", ns::PUBSUB) {
                if required {
                    return Err(Error::ParseError(
                        "More than one required element in subscribe-options.",
                    ));
                }
                required = true;
            } else {
                return Err(Error::ParseError(
                    "Unknown child in subscribe-options element.",
                ));
            }
        }
        Ok(SubscribeOptions { required })
    }
}

impl From<SubscribeOptions> for Element {
    fn from(subscribe_options: SubscribeOptions) -> Element {
        Element::builder("subscribe-options", ns::PUBSUB)
            .append_all(if subscribe_options.required {
                Some(Element::builder("required", ns::PUBSUB))
            } else {
                None
            })
            .build()
    }
}

generate_element!(
    /// A request to subscribe a JID to a node.
    Subscribe, "subscribe", PUBSUB,
    attributes: [
        /// The JID being subscribed.
        jid: Required<Jid> = "jid",

        /// The node to subscribe to.
        node: Option<NodeName> = "node",
    ]
);

generate_element!(
    /// A request for current subscriptions.
    Subscriptions, "subscriptions", PUBSUB,
    attributes: [
        /// The node to query.
        node: Option<NodeName> = "node",
    ],
    children: [
        /// The list of subscription elements returned.
        subscription: Vec<SubscriptionElem> = ("subscription", PUBSUB) => SubscriptionElem
    ]
);

generate_element!(
    /// A subscription element, describing the state of a subscription.
    SubscriptionElem, "subscription", PUBSUB,
    attributes: [
        /// The JID affected by this subscription.
        jid: Required<Jid> = "jid",

        /// The node affected by this subscription.
        node: Option<NodeName> = "node",

        /// The subscription identifier for this subscription.
        subid: Option<SubscriptionId> = "subid",

        /// The state of the subscription.
        subscription: Option<Subscription> = "subscription",
    ],
    children: [
        /// The options related to this subscription.
        subscribe_options: Option<SubscribeOptions> = ("subscribe-options", PUBSUB) => SubscribeOptions
    ]
);

generate_element!(
    /// An unsubscribe request.
    Unsubscribe, "unsubscribe", PUBSUB,
    attributes: [
        /// The JID affected by this request.
        jid: Required<Jid> = "jid",

        /// The node affected by this request.
        node: Option<NodeName> = "node",

        /// The subscription identifier for this subscription.
        subid: Option<SubscriptionId> = "subid",
    ]
);

/// Main payload used to communicate with a PubSub service.
///
/// `<pubsub xmlns="http://jabber.org/protocol/pubsub"/>`
#[derive(Debug, Clone, PartialEq)]
pub enum PubSub {
    /// Request to create a new node, with optional suggested name and suggested configuration.
    Create {
        /// The create request.
        create: Create,

        /// The configure request for the new node.
        configure: Option<Configure>,
    },

    /// A subcribe request.
    Subscribe {
        /// The subscribe request.
        subscribe: Option<Subscribe>,

        /// The options related to this subscribe request.
        options: Option<Options>,
    },

    /// Request to publish items to a node, with optional options.
    Publish {
        /// The publish request.
        publish: Publish,

        /// The options related to this publish request.
        publish_options: Option<PublishOptions>,
    },

    /// A list of affiliations you have on a service, or on a node.
    Affiliations(Affiliations),

    /// Request for a default node configuration.
    Default(Default),

    /// A request for a list of items.
    Items(Items),

    /// A request to retract some items from a node.
    Retract(Retract),

    /// A request about a subscription.
    Subscription(SubscriptionElem),

    /// A request for current subscriptions.
    Subscriptions(Subscriptions),

    /// An unsubscribe request.
    Unsubscribe(Unsubscribe),
}

impl IqGetPayload for PubSub {}
impl IqSetPayload for PubSub {}
impl IqResultPayload for PubSub {}

impl TryFrom<Element> for PubSub {
    type Error = Error;

    fn try_from(elem: Element) -> Result<PubSub, Error> {
        check_self!(elem, "pubsub", PUBSUB);
        check_no_attributes!(elem, "pubsub");

        let mut payload = None;
        for child in elem.children() {
            if child.is("create", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let create = Create::try_from(child.clone())?;
                payload = Some(PubSub::Create {
                    create,
                    configure: None,
                });
            } else if child.is("subscribe", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let subscribe = Subscribe::try_from(child.clone())?;
                payload = Some(PubSub::Subscribe {
                    subscribe: Some(subscribe),
                    options: None,
                });
            } else if child.is("options", ns::PUBSUB) {
                if let Some(PubSub::Subscribe { subscribe, options }) = payload {
                    if options.is_some() {
                        return Err(Error::ParseError(
                            "Options is already defined in pubsub element.",
                        ));
                    }
                    let options = Some(Options::try_from(child.clone())?);
                    payload = Some(PubSub::Subscribe { subscribe, options });
                } else if payload.is_none() {
                    let options = Options::try_from(child.clone())?;
                    payload = Some(PubSub::Subscribe {
                        subscribe: None,
                        options: Some(options),
                    });
                } else {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
            } else if child.is("configure", ns::PUBSUB) {
                if let Some(PubSub::Create { create, configure }) = payload {
                    if configure.is_some() {
                        return Err(Error::ParseError(
                            "Configure is already defined in pubsub element.",
                        ));
                    }
                    let configure = Some(Configure::try_from(child.clone())?);
                    payload = Some(PubSub::Create { create, configure });
                } else {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
            } else if child.is("publish", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let publish = Publish::try_from(child.clone())?;
                payload = Some(PubSub::Publish {
                    publish,
                    publish_options: None,
                });
            } else if child.is("publish-options", ns::PUBSUB) {
                if let Some(PubSub::Publish {
                    publish,
                    publish_options,
                }) = payload
                {
                    if publish_options.is_some() {
                        return Err(Error::ParseError(
                            "Publish-options are already defined in pubsub element.",
                        ));
                    }
                    let publish_options = Some(PublishOptions::try_from(child.clone())?);
                    payload = Some(PubSub::Publish {
                        publish,
                        publish_options,
                    });
                } else {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
            } else if child.is("affiliations", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let affiliations = Affiliations::try_from(child.clone())?;
                payload = Some(PubSub::Affiliations(affiliations));
            } else if child.is("default", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let default = Default::try_from(child.clone())?;
                payload = Some(PubSub::Default(default));
            } else if child.is("items", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let items = Items::try_from(child.clone())?;
                payload = Some(PubSub::Items(items));
            } else if child.is("retract", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let retract = Retract::try_from(child.clone())?;
                payload = Some(PubSub::Retract(retract));
            } else if child.is("subscription", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let subscription = SubscriptionElem::try_from(child.clone())?;
                payload = Some(PubSub::Subscription(subscription));
            } else if child.is("subscriptions", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let subscriptions = Subscriptions::try_from(child.clone())?;
                payload = Some(PubSub::Subscriptions(subscriptions));
            } else if child.is("unsubscribe", ns::PUBSUB) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Payload is already defined in pubsub element.",
                    ));
                }
                let unsubscribe = Unsubscribe::try_from(child.clone())?;
                payload = Some(PubSub::Unsubscribe(unsubscribe));
            } else {
                return Err(Error::ParseError("Unknown child in pubsub element."));
            }
        }
        Ok(payload.ok_or(Error::ParseError("No payload in pubsub element."))?)
    }
}

impl From<PubSub> for Element {
    fn from(pubsub: PubSub) -> Element {
        Element::builder("pubsub", ns::PUBSUB)
            .append_all(match pubsub {
                PubSub::Create { create, configure } => {
                    let mut elems = vec![Element::from(create)];
                    if let Some(configure) = configure {
                        elems.push(Element::from(configure));
                    }
                    elems
                }
                PubSub::Subscribe { subscribe, options } => {
                    let mut elems = vec![];
                    if let Some(subscribe) = subscribe {
                        elems.push(Element::from(subscribe));
                    }
                    if let Some(options) = options {
                        elems.push(Element::from(options));
                    }
                    elems
                }
                PubSub::Publish {
                    publish,
                    publish_options,
                } => {
                    let mut elems = vec![Element::from(publish)];
                    if let Some(publish_options) = publish_options {
                        elems.push(Element::from(publish_options));
                    }
                    elems
                }
                PubSub::Affiliations(affiliations) => vec![Element::from(affiliations)],
                PubSub::Default(default) => vec![Element::from(default)],
                PubSub::Items(items) => vec![Element::from(items)],
                PubSub::Retract(retract) => vec![Element::from(retract)],
                PubSub::Subscription(subscription) => vec![Element::from(subscription)],
                PubSub::Subscriptions(subscriptions) => vec![Element::from(subscriptions)],
                PubSub::Unsubscribe(unsubscribe) => vec![Element::from(unsubscribe)],
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_forms::{DataForm, DataFormType, Field, FieldType};
    use jid::FullJid;

    #[test]
    fn create() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create/></pubsub>"
            .parse()
            .unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Create { create, configure } => {
                assert!(create.node.is_none());
                assert!(configure.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);

        let elem: Element =
            "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create node='coucou'/></pubsub>"
                .parse()
                .unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Create { create, configure } => {
                assert_eq!(&create.node.unwrap().0, "coucou");
                assert!(configure.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn create_and_configure_empty() {
        let elem: Element =
            "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create/><configure/></pubsub>"
                .parse()
                .unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Create { create, configure } => {
                assert!(create.node.is_none());
                assert!(configure.unwrap().form.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn create_and_configure_simple() {
        // XXX: Do we want xmpp-parsers to always specify the field type in the output Element?
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><create node='foo'/><configure><x xmlns='jabber:x:data' type='submit'><field var='FORM_TYPE' type='hidden'><value>http://jabber.org/protocol/pubsub#node_config</value></field><field var='pubsub#access_model' type='list-single'><value>whitelist</value></field></x></configure></pubsub>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();

        let pubsub = PubSub::Create {
            create: Create {
                node: Some(NodeName(String::from("foo"))),
            },
            configure: Some(Configure {
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
            }),
        };

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn publish() {
        let elem: Element =
            "<pubsub xmlns='http://jabber.org/protocol/pubsub'><publish node='coucou'/></pubsub>"
                .parse()
                .unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Publish {
                publish,
                publish_options,
            } => {
                assert_eq!(&publish.node.0, "coucou");
                assert!(publish_options.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn publish_with_publish_options() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><publish node='coucou'/><publish-options/></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Publish {
                publish,
                publish_options,
            } => {
                assert_eq!(&publish.node.0, "coucou");
                assert!(publish_options.unwrap().form.is_none());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn invalid_empty_pubsub() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'/>"
            .parse()
            .unwrap();
        let error = PubSub::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "No payload in pubsub element.");
    }

    #[test]
    fn publish_option() {
        let elem: Element = "<publish-options xmlns='http://jabber.org/protocol/pubsub'><x xmlns='jabber:x:data' type='submit'><field var='FORM_TYPE' type='hidden'><value>http://jabber.org/protocol/pubsub#publish-options</value></field></x></publish-options>".parse().unwrap();
        let publish_options = PublishOptions::try_from(elem).unwrap();
        assert_eq!(
            &publish_options.form.unwrap().form_type.unwrap(),
            "http://jabber.org/protocol/pubsub#publish-options"
        );
    }

    #[test]
    fn subscribe_options() {
        let elem1: Element = "<subscribe-options xmlns='http://jabber.org/protocol/pubsub'/>"
            .parse()
            .unwrap();
        let subscribe_options1 = SubscribeOptions::try_from(elem1).unwrap();
        assert_eq!(subscribe_options1.required, false);

        let elem2: Element = "<subscribe-options xmlns='http://jabber.org/protocol/pubsub'><required/></subscribe-options>".parse().unwrap();
        let subscribe_options2 = SubscribeOptions::try_from(elem2).unwrap();
        assert_eq!(subscribe_options2.required, true);
    }

    #[test]
    fn test_options_without_subscribe() {
        let elem: Element = "<pubsub xmlns='http://jabber.org/protocol/pubsub'><options xmlns='http://jabber.org/protocol/pubsub' jid='juliet@capulet.lit/balcony'><x xmlns='jabber:x:data' type='submit'/></options></pubsub>".parse().unwrap();
        let elem1 = elem.clone();
        let pubsub = PubSub::try_from(elem).unwrap();
        match pubsub.clone() {
            PubSub::Subscribe { subscribe, options } => {
                assert!(subscribe.is_none());
                assert!(options.is_some());
            }
            _ => panic!(),
        }

        let elem2 = Element::from(pubsub);
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_serialize_options() {
        let reference: Element = "<options xmlns='http://jabber.org/protocol/pubsub' jid='juliet@capulet.lit/balcony'><x xmlns='jabber:x:data' type='submit'/></options>"
        .parse()
        .unwrap();

        let elem: Element = "<x xmlns='jabber:x:data' type='submit'/>".parse().unwrap();

        let form = DataForm::try_from(elem).unwrap();

        let options = Options {
            jid: Jid::Full(FullJid::new("juliet", "capulet.lit", "balcony")),
            node: None,
            subid: None,
            form: Some(form),
        };
        let serialized: Element = options.into();
        assert_eq!(serialized, reference);
    }

    #[test]
    fn test_serialize_publish_options() {
        let reference: Element = "<publish-options xmlns='http://jabber.org/protocol/pubsub'><x xmlns='jabber:x:data' type='submit'/></publish-options>"
        .parse()
        .unwrap();

        let elem: Element = "<x xmlns='jabber:x:data' type='submit'/>".parse().unwrap();

        let form = DataForm::try_from(elem).unwrap();

        let options = PublishOptions { form: Some(form) };
        let serialized: Element = options.into();
        assert_eq!(serialized, reference);
    }
}
