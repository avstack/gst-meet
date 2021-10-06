// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::message::MessagePayload;

generate_element_enum!(
    /// Enum representing chatstate elements part of the
    /// `http://jabber.org/protocol/chatstates` namespace.
    ChatState, "chatstate", CHATSTATES, {
        /// `<active xmlns='http://jabber.org/protocol/chatstates'/>`
        Active => "active",

        /// `<composing xmlns='http://jabber.org/protocol/chatstates'/>`
        Composing => "composing",

        /// `<gone xmlns='http://jabber.org/protocol/chatstates'/>`
        Gone => "gone",

        /// `<inactive xmlns='http://jabber.org/protocol/chatstates'/>`
        Inactive => "inactive",

        /// `<paused xmlns='http://jabber.org/protocol/chatstates'/>`
        Paused => "paused",
    }
);

impl MessagePayload for ChatState {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ns;
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;

    #[test]
    fn test_size() {
        assert_size!(ChatState, 1);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<active xmlns='http://jabber.org/protocol/chatstates'/>"
            .parse()
            .unwrap();
        ChatState::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<coucou xmlns='http://jabber.org/protocol/chatstates'/>"
            .parse()
            .unwrap();
        let error = ChatState::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a chatstate element.");
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_child() {
        let elem: Element = "<gone xmlns='http://jabber.org/protocol/chatstates'><coucou/></gone>"
            .parse()
            .unwrap();
        let error = ChatState::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in chatstate element.");
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<inactive xmlns='http://jabber.org/protocol/chatstates' coucou=''/>"
            .parse()
            .unwrap();
        let error = ChatState::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in chatstate element.");
    }

    #[test]
    fn test_serialise() {
        let chatstate = ChatState::Active;
        let elem: Element = chatstate.into();
        assert!(elem.is("active", ns::CHATSTATES));
    }
}
