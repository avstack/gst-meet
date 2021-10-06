// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::message::MessagePayload;

generate_element!(
    /// Defines that the message containing this payload should replace a
    /// previous message, identified by the id.
    Replace, "replace", MESSAGE_CORRECT,
    attributes: [
        /// The 'id' attribute of the message getting corrected.
        id: Required<String> = "id",
    ]
);

impl MessagePayload for Replace {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Replace, 12);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Replace, 24);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' id='coucou'/>"
            .parse()
            .unwrap();
        Replace::try_from(elem).unwrap();
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' coucou=''/>"
            .parse()
            .unwrap();
        let error = Replace::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in replace element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'><coucou/></replace>"
            .parse()
            .unwrap();
        let error = Replace::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in replace element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>"
            .parse()
            .unwrap();
        let error = Replace::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'id' missing.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0' id='coucou'/>"
            .parse()
            .unwrap();
        let replace = Replace {
            id: String::from("coucou"),
        };
        let elem2 = replace.into();
        assert_eq!(elem, elem2);
    }
}
