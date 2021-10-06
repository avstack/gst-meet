// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::iq::{IqGetPayload, IqResultPayload, IqSetPayload};
use crate::ns;
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use std::convert::TryFrom;

generate_empty_element!(
    /// The element requesting the blocklist, the result iq will contain a
    /// [BlocklistResult].
    BlocklistRequest,
    "blocklist",
    BLOCKING
);

impl IqGetPayload for BlocklistRequest {}

macro_rules! generate_blocking_element {
    ($(#[$meta:meta])* $elem:ident, $name:tt) => (
        $(#[$meta])*
        #[derive(Debug, Clone)]
        pub struct $elem {
            /// List of JIDs affected by this command.
            pub items: Vec<Jid>,
        }

        impl TryFrom<Element> for $elem {
            type Error = Error;

            fn try_from(elem: Element) -> Result<$elem, Error> {
                check_self!(elem, $name, BLOCKING);
                check_no_attributes!(elem, $name);
                let mut items = vec!();
                for child in elem.children() {
                    check_self!(child, "item", BLOCKING);
                    check_no_unknown_attributes!(child, "item", ["jid"]);
                    check_no_children!(child, "item");
                    items.push(get_attr!(child, "jid", Required));
                }
                Ok($elem { items })
            }
        }

        impl From<$elem> for Element {
            fn from(elem: $elem) -> Element {
                Element::builder($name, ns::BLOCKING)
                        .append_all(elem.items.into_iter().map(|jid| {
                             Element::builder("item", ns::BLOCKING)
                                     .attr("jid", jid)
                         }))
                        .build()
            }
        }
    );
}

generate_blocking_element!(
    /// The element containing the current blocklist, as a reply from
    /// [BlocklistRequest].
    BlocklistResult,
    "blocklist"
);

impl IqResultPayload for BlocklistResult {}

// TODO: Prevent zero elements from being allowed.
generate_blocking_element!(
    /// A query to block one or more JIDs.
    Block,
    "block"
);

impl IqSetPayload for Block {}

generate_blocking_element!(
    /// A query to unblock one or more JIDs, or all of them.
    ///
    /// Warning: not putting any JID there means clearing out the blocklist.
    Unblock,
    "unblock"
);

impl IqSetPayload for Unblock {}

generate_empty_element!(
    /// The application-specific error condition when a message is blocked.
    Blocked,
    "blocked",
    BLOCKING_ERRORS
);

#[cfg(test)]
mod tests {
    use super::*;
    use jid::BareJid;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(BlocklistRequest, 0);
        assert_size!(BlocklistResult, 12);
        assert_size!(Block, 12);
        assert_size!(Unblock, 12);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(BlocklistRequest, 0);
        assert_size!(BlocklistResult, 24);
        assert_size!(Block, 24);
        assert_size!(Unblock, 24);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<blocklist xmlns='urn:xmpp:blocking'/>".parse().unwrap();
        let request_elem = elem.clone();
        BlocklistRequest::try_from(request_elem).unwrap();

        let result_elem = elem.clone();
        let result = BlocklistResult::try_from(result_elem).unwrap();
        assert!(result.items.is_empty());

        let elem: Element = "<block xmlns='urn:xmpp:blocking'/>".parse().unwrap();
        let block = Block::try_from(elem).unwrap();
        assert!(block.items.is_empty());

        let elem: Element = "<unblock xmlns='urn:xmpp:blocking'/>".parse().unwrap();
        let unblock = Unblock::try_from(elem).unwrap();
        assert!(unblock.items.is_empty());
    }

    #[test]
    fn test_items() {
        let elem: Element = "<blocklist xmlns='urn:xmpp:blocking'><item jid='coucou@coucou'/><item jid='domain'/></blocklist>".parse().unwrap();
        let two_items = vec![
            Jid::Bare(BareJid {
                node: Some(String::from("coucou")),
                domain: String::from("coucou"),
            }),
            Jid::Bare(BareJid {
                node: None,
                domain: String::from("domain"),
            }),
        ];

        let result_elem = elem.clone();
        let result = BlocklistResult::try_from(result_elem).unwrap();
        assert_eq!(result.items, two_items);

        let elem: Element = "<block xmlns='urn:xmpp:blocking'><item jid='coucou@coucou'/><item jid='domain'/></block>".parse().unwrap();
        let block = Block::try_from(elem).unwrap();
        assert_eq!(block.items, two_items);

        let elem: Element = "<unblock xmlns='urn:xmpp:blocking'><item jid='coucou@coucou'/><item jid='domain'/></unblock>".parse().unwrap();
        let unblock = Unblock::try_from(elem).unwrap();
        assert_eq!(unblock.items, two_items);
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid() {
        let elem: Element = "<blocklist xmlns='urn:xmpp:blocking' coucou=''/>"
            .parse()
            .unwrap();
        let request_elem = elem.clone();
        let error = BlocklistRequest::try_from(request_elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in blocklist element.");

        let result_elem = elem.clone();
        let error = BlocklistResult::try_from(result_elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in blocklist element.");

        let elem: Element = "<block xmlns='urn:xmpp:blocking' coucou=''/>"
            .parse()
            .unwrap();
        let error = Block::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in block element.");

        let elem: Element = "<unblock xmlns='urn:xmpp:blocking' coucou=''/>"
            .parse()
            .unwrap();
        let error = Unblock::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in unblock element.");
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_non_empty_blocklist_request() {
        let elem: Element = "<blocklist xmlns='urn:xmpp:blocking'><item jid='coucou@coucou'/><item jid='domain'/></blocklist>".parse().unwrap();
        let error = BlocklistRequest::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in blocklist element.");
    }
}
