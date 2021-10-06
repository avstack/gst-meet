// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ibb::{Stanza, StreamId};

generate_element!(
/// Describes an [In-Band Bytestream](https://xmpp.org/extensions/xep-0047.html)
/// Jingle transport, see also the [IBB module](../ibb.rs).
Transport, "transport", JINGLE_IBB,
attributes: [
    /// Maximum size in bytes for each chunk.
    block_size: Required<u16> = "block-size",

    /// The identifier to be used to create a stream.
    sid: Required<StreamId> = "sid",

    /// Which stanza type to use to exchange data.
    stanza: Default<Stanza> = "stanza",
]);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Transport, 16);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Transport, 32);
    }

    #[test]
    fn test_simple() {
        let elem: Element =
            "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='3' sid='coucou'/>"
                .parse()
                .unwrap();
        let transport = Transport::try_from(elem).unwrap();
        assert_eq!(transport.block_size, 3);
        assert_eq!(transport.sid, StreamId(String::from("coucou")));
        assert_eq!(transport.stanza, Stanza::Iq);
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1'/>"
            .parse()
            .unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing.");

        let elem: Element =
            "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='65536'/>"
                .parse()
                .unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(
            message.to_string(),
            "number too large to fit in target type"
        );

        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='-5'/>"
            .parse()
            .unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "invalid digit found in string");

        let elem: Element =
            "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='128'/>"
                .parse()
                .unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing.");
    }

    #[test]
    fn test_invalid_stanza() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:ibb:1' block-size='128' sid='coucou' stanza='fdsq'/>".parse().unwrap();
        let error = Transport::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'stanza' attribute.");
    }
}
