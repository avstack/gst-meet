// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::data_forms::{DataForm, DataFormType};
use crate::iq::{IqGetPayload, IqResultPayload};
use crate::ns;
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use std::convert::TryFrom;

generate_element!(
/// Structure representing a `<query xmlns='http://jabber.org/protocol/disco#info'/>` element.
///
/// It should only be used in an `<iq type='get'/>`, as it can only represent
/// the request, and not a result.
DiscoInfoQuery, "query", DISCO_INFO,
attributes: [
    /// Node on which we are doing the discovery.
    node: Option<String> = "node",
]);

impl IqGetPayload for DiscoInfoQuery {}

generate_element!(
#[derive(Eq, Hash)]
/// Structure representing a `<feature xmlns='http://jabber.org/protocol/disco#info'/>` element.
Feature, "feature", DISCO_INFO,
attributes: [
    /// Namespace of the feature we want to represent.
    var: Required<String> = "var",
]);

impl Feature {
    /// Create a new `<feature/>` with the according `@var`.
    pub fn new<S: Into<String>>(var: S) -> Feature {
        Feature { var: var.into() }
    }
}

generate_element!(
    /// Structure representing an `<identity xmlns='http://jabber.org/protocol/disco#info'/>` element.
    Identity, "identity", DISCO_INFO,
    attributes: [
        /// Category of this identity.
        // TODO: use an enum here.
        category: RequiredNonEmpty<String> = "category",

        /// Type of this identity.
        // TODO: use an enum here.
        type_: RequiredNonEmpty<String> = "type",

        /// Lang of the name of this identity.
        lang: Option<String> = "xml:lang",

        /// Name of this identity.
        name: Option<String> = "name",
    ]
);

impl Identity {
    /// Create a new `<identity/>`.
    pub fn new<C, T, L, N>(category: C, type_: T, lang: L, name: N) -> Identity
    where
        C: Into<String>,
        T: Into<String>,
        L: Into<String>,
        N: Into<String>,
    {
        Identity {
            category: category.into(),
            type_: type_.into(),
            lang: Some(lang.into()),
            name: Some(name.into()),
        }
    }

    /// Create a new `<identity/>` without a name.
    pub fn new_anonymous<C, T, L, N>(category: C, type_: T) -> Identity
    where
        C: Into<String>,
        T: Into<String>,
    {
        Identity {
            category: category.into(),
            type_: type_.into(),
            lang: None,
            name: None,
        }
    }
}

/// Structure representing a `<query xmlns='http://jabber.org/protocol/disco#info'/>` element.
///
/// It should only be used in an `<iq type='result'/>`, as it can only
/// represent the result, and not a request.
#[derive(Debug, Clone)]
pub struct DiscoInfoResult {
    /// Node on which we have done this discovery.
    pub node: Option<String>,

    /// List of identities exposed by this entity.
    pub identities: Vec<Identity>,

    /// List of features supported by this entity.
    pub features: Vec<Feature>,

    /// List of extensions reported by this entity.
    pub extensions: Vec<DataForm>,
}

impl IqResultPayload for DiscoInfoResult {}

impl TryFrom<Element> for DiscoInfoResult {
    type Error = Error;

    fn try_from(elem: Element) -> Result<DiscoInfoResult, Error> {
        check_self!(elem, "query", DISCO_INFO, "disco#info result");
        check_no_unknown_attributes!(elem, "disco#info result", ["node"]);

        let mut result = DiscoInfoResult {
            node: get_attr!(elem, "node", Option),
            identities: vec![],
            features: vec![],
            extensions: vec![],
        };

        for child in elem.children() {
            if child.is("identity", ns::DISCO_INFO) {
                let identity = Identity::try_from(child.clone())?;
                result.identities.push(identity);
            } else if child.is("feature", ns::DISCO_INFO) {
                let feature = Feature::try_from(child.clone())?;
                result.features.push(feature);
            } else if child.is("x", ns::DATA_FORMS) {
                let data_form = DataForm::try_from(child.clone())?;
                if data_form.type_ != DataFormType::Result_ {
                    return Err(Error::ParseError(
                        "Data form must have a 'result' type in disco#info.",
                    ));
                }
                if data_form.form_type.is_none() {
                    return Err(Error::ParseError("Data form found without a FORM_TYPE."));
                }
                result.extensions.push(data_form);
            } else {
                return Err(Error::ParseError("Unknown element in disco#info."));
            }
        }

        if result.identities.is_empty() {
            return Err(Error::ParseError(
                "There must be at least one identity in disco#info.",
            ));
        }
        if result.features.is_empty() {
            return Err(Error::ParseError(
                "There must be at least one feature in disco#info.",
            ));
        }
        if !result.features.contains(&Feature {
            var: ns::DISCO_INFO.to_owned(),
        }) {
            return Err(Error::ParseError(
                "disco#info feature not present in disco#info.",
            ));
        }

        Ok(result)
    }
}

impl From<DiscoInfoResult> for Element {
    fn from(disco: DiscoInfoResult) -> Element {
        Element::builder("query", ns::DISCO_INFO)
            .attr("node", disco.node)
            .append_all(disco.identities.into_iter())
            .append_all(disco.features.into_iter())
            .append_all(disco.extensions.iter().cloned().map(Element::from))
            .build()
    }
}

generate_element!(
/// Structure representing a `<query xmlns='http://jabber.org/protocol/disco#items'/>` element.
///
/// It should only be used in an `<iq type='get'/>`, as it can only represent
/// the request, and not a result.
DiscoItemsQuery, "query", DISCO_ITEMS,
attributes: [
    /// Node on which we are doing the discovery.
    node: Option<String> = "node",
]);

impl IqGetPayload for DiscoItemsQuery {}

generate_element!(
/// Structure representing an `<item xmlns='http://jabber.org/protocol/disco#items'/>` element.
Item, "item", DISCO_ITEMS,
attributes: [
    /// JID of the entity pointed by this item.
    jid: Required<Jid> = "jid",
    /// Node of the entity pointed by this item.
    node: Option<String> = "node",
    /// Name of the entity pointed by this item.
    name: Option<String> = "name",
]);

generate_element!(
    /// Structure representing a `<query
    /// xmlns='http://jabber.org/protocol/disco#items'/>` element.
    ///
    /// It should only be used in an `<iq type='result'/>`, as it can only
    /// represent the result, and not a request.
    DiscoItemsResult, "query", DISCO_ITEMS,
    attributes: [
        /// Node on which we have done this discovery.
        node: Option<String> = "node"
    ],
    children: [
        /// List of items pointed by this entity.
        items: Vec<Item> = ("item", DISCO_ITEMS) => Item
    ]
);

impl IqResultPayload for DiscoItemsResult {}

#[cfg(test)]
mod tests {
    use super::*;
    use jid::BareJid;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Identity, 48);
        assert_size!(Feature, 12);
        assert_size!(DiscoInfoQuery, 12);
        assert_size!(DiscoInfoResult, 48);

        assert_size!(Item, 64);
        assert_size!(DiscoItemsQuery, 12);
        assert_size!(DiscoItemsResult, 24);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Identity, 96);
        assert_size!(Feature, 24);
        assert_size!(DiscoInfoQuery, 24);
        assert_size!(DiscoInfoResult, 96);

        assert_size!(Item, 128);
        assert_size!(DiscoItemsQuery, 24);
        assert_size!(DiscoItemsResult, 48);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><feature var='http://jabber.org/protocol/disco#info'/></query>".parse().unwrap();
        let query = DiscoInfoResult::try_from(elem).unwrap();
        assert!(query.node.is_none());
        assert_eq!(query.identities.len(), 1);
        assert_eq!(query.features.len(), 1);
        assert!(query.extensions.is_empty());
    }

    #[test]
    fn test_identity_after_feature() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><feature var='http://jabber.org/protocol/disco#info'/><identity category='client' type='pc'/></query>".parse().unwrap();
        let query = DiscoInfoResult::try_from(elem).unwrap();
        assert_eq!(query.identities.len(), 1);
        assert_eq!(query.features.len(), 1);
        assert!(query.extensions.is_empty());
    }

    #[test]
    fn test_feature_after_dataform() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><x xmlns='jabber:x:data' type='result'><field var='FORM_TYPE' type='hidden'><value>coucou</value></field></x><feature var='http://jabber.org/protocol/disco#info'/></query>".parse().unwrap();
        let query = DiscoInfoResult::try_from(elem).unwrap();
        assert_eq!(query.identities.len(), 1);
        assert_eq!(query.features.len(), 1);
        assert_eq!(query.extensions.len(), 1);
    }

    #[test]
    fn test_extension() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><feature var='http://jabber.org/protocol/disco#info'/><x xmlns='jabber:x:data' type='result'><field var='FORM_TYPE' type='hidden'><value>example</value></field></x></query>".parse().unwrap();
        let elem1 = elem.clone();
        let query = DiscoInfoResult::try_from(elem).unwrap();
        assert!(query.node.is_none());
        assert_eq!(query.identities.len(), 1);
        assert_eq!(query.features.len(), 1);
        assert_eq!(query.extensions.len(), 1);
        assert_eq!(query.extensions[0].form_type, Some(String::from("example")));

        let elem2 = query.into();
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_invalid() {
        let elem: Element =
            "<query xmlns='http://jabber.org/protocol/disco#info'><coucou/></query>"
                .parse()
                .unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown element in disco#info.");
    }

    #[test]
    fn test_invalid_identity() {
        let elem: Element =
            "<query xmlns='http://jabber.org/protocol/disco#info'><identity/></query>"
                .parse()
                .unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'category' missing.");

        let elem: Element =
            "<query xmlns='http://jabber.org/protocol/disco#info'><identity category=''/></query>"
                .parse()
                .unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'category' must not be empty.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='coucou'/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='coucou' type=''/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' must not be empty.");
    }

    #[test]
    fn test_invalid_feature() {
        let elem: Element =
            "<query xmlns='http://jabber.org/protocol/disco#info'><feature/></query>"
                .parse()
                .unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'var' missing.");
    }

    #[test]
    fn test_invalid_result() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'/>"
            .parse()
            .unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(
            message,
            "There must be at least one identity in disco#info."
        );

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "There must be at least one feature in disco#info.");

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#info'><identity category='client' type='pc'/><feature var='http://jabber.org/protocol/disco#items'/></query>".parse().unwrap();
        let error = DiscoInfoResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "disco#info feature not present in disco#info.");
    }

    #[test]
    fn test_simple_items() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items'/>"
            .parse()
            .unwrap();
        let query = DiscoItemsQuery::try_from(elem).unwrap();
        assert!(query.node.is_none());

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items' node='coucou'/>"
            .parse()
            .unwrap();
        let query = DiscoItemsQuery::try_from(elem).unwrap();
        assert_eq!(query.node, Some(String::from("coucou")));
    }

    #[test]
    fn test_simple_items_result() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items'/>"
            .parse()
            .unwrap();
        let query = DiscoItemsResult::try_from(elem).unwrap();
        assert!(query.node.is_none());
        assert!(query.items.is_empty());

        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items' node='coucou'/>"
            .parse()
            .unwrap();
        let query = DiscoItemsResult::try_from(elem).unwrap();
        assert_eq!(query.node, Some(String::from("coucou")));
        assert!(query.items.is_empty());
    }

    #[test]
    fn test_answers_items_result() {
        let elem: Element = "<query xmlns='http://jabber.org/protocol/disco#items'><item jid='component'/><item jid='component2' node='test' name='A component'/></query>".parse().unwrap();
        let query = DiscoItemsResult::try_from(elem).unwrap();
        let elem2 = Element::from(query);
        let query = DiscoItemsResult::try_from(elem2).unwrap();
        assert_eq!(query.items.len(), 2);
        assert_eq!(query.items[0].jid, BareJid::domain("component"));
        assert_eq!(query.items[0].node, None);
        assert_eq!(query.items[0].name, None);
        assert_eq!(query.items[1].jid, BareJid::domain("component2"));
        assert_eq!(query.items[1].node, Some(String::from("test")));
        assert_eq!(query.items[1].name, Some(String::from("A component")));
    }
}
