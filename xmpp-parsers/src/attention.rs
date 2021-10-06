// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::message::MessagePayload;

generate_empty_element!(
    /// Requests the attention of the recipient.
    Attention,
    "attention",
    ATTENTION
);

impl MessagePayload for Attention {}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "disable-validation"))]
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;

    #[test]
    fn test_size() {
        assert_size!(Attention, 0);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<attention xmlns='urn:xmpp:attention:0'/>".parse().unwrap();
        Attention::try_from(elem).unwrap();
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_child() {
        let elem: Element = "<attention xmlns='urn:xmpp:attention:0'><coucou/></attention>"
            .parse()
            .unwrap();
        let error = Attention::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in attention element.");
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<attention xmlns='urn:xmpp:attention:0' coucou=''/>"
            .parse()
            .unwrap();
        let error = Attention::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in attention element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<attention xmlns='urn:xmpp:attention:0'/>".parse().unwrap();
        let attention = Attention;
        let elem2: Element = attention.into();
        assert_eq!(elem, elem2);
    }
}
