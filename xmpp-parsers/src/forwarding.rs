// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::delay::Delay;
use crate::message::Message;

generate_element!(
    /// Contains a forwarded stanza, either standalone or part of another
    /// extension (such as carbons).
    Forwarded, "forwarded", FORWARD,
    children: [
        /// When the stanza originally got sent.
        delay: Option<Delay> = ("delay", DELAY) => Delay,

        // XXX: really?  Option?
        /// The stanza being forwarded.
        stanza: Option<Message> = ("message", DEFAULT_NS) => Message

        // TODO: also handle the two other stanza possibilities.
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Forwarded, 212);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Forwarded, 408);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'/>".parse().unwrap();
        Forwarded::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'><coucou/></forwarded>"
            .parse()
            .unwrap();
        let error = Forwarded::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in forwarded element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'/>".parse().unwrap();
        let forwarded = Forwarded {
            delay: None,
            stanza: None,
        };
        let elem2 = forwarded.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_serialize_with_delay_and_stanza() {
        let reference: Element = "<forwarded xmlns='urn:xmpp:forward:0'><delay xmlns='urn:xmpp:delay' from='capulet.com' stamp='2002-09-10T23:08:25+00:00'/><message xmlns='jabber:client' to='juliet@capulet.example/balcony' from='romeo@montague.example/home'/></forwarded>"
        .parse()
        .unwrap();

        let elem: Element = "<message xmlns='jabber:client' to='juliet@capulet.example/balcony' from='romeo@montague.example/home'/>"
          .parse()
          .unwrap();
        let message = Message::try_from(elem).unwrap();

        let elem: Element =
            "<delay xmlns='urn:xmpp:delay' from='capulet.com' stamp='2002-09-10T23:08:25Z'/>"
                .parse()
                .unwrap();
        let delay = Delay::try_from(elem).unwrap();

        let forwarded = Forwarded {
            delay: Some(delay),
            stanza: Some(message),
        };

        let serialized: Element = forwarded.into();
        assert_eq!(serialized, reference);
    }
}
