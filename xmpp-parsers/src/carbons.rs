// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::forwarding::Forwarded;
use crate::iq::IqSetPayload;
use crate::message::MessagePayload;

generate_empty_element!(
    /// Enable carbons for this session.
    Enable,
    "enable",
    CARBONS
);

impl IqSetPayload for Enable {}

generate_empty_element!(
    /// Disable a previously-enabled carbons.
    Disable,
    "disable",
    CARBONS
);

impl IqSetPayload for Disable {}

generate_empty_element!(
    /// Request the enclosing message to not be copied to other carbons-enabled
    /// resources of the user.
    Private,
    "private",
    CARBONS
);

impl MessagePayload for Private {}

generate_element!(
    /// Wrapper for a message received on another resource.
    Received, "received", CARBONS,

    children: [
        /// Wrapper for the enclosed message.
        forwarded: Required<Forwarded> = ("forwarded", FORWARD) => Forwarded
    ]
);

impl MessagePayload for Received {}

generate_element!(
    /// Wrapper for a message sent from another resource.
    Sent, "sent", CARBONS,

    children: [
        /// Wrapper for the enclosed message.
        forwarded: Required<Forwarded> = ("forwarded", FORWARD) => Forwarded
    ]
);

impl MessagePayload for Sent {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Enable, 0);
        assert_size!(Disable, 0);
        assert_size!(Private, 0);
        assert_size!(Received, 212);
        assert_size!(Sent, 212);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Enable, 0);
        assert_size!(Disable, 0);
        assert_size!(Private, 0);
        assert_size!(Received, 408);
        assert_size!(Sent, 408);
    }

    #[test]
    fn empty_elements() {
        let elem: Element = "<enable xmlns='urn:xmpp:carbons:2'/>".parse().unwrap();
        Enable::try_from(elem).unwrap();

        let elem: Element = "<disable xmlns='urn:xmpp:carbons:2'/>".parse().unwrap();
        Disable::try_from(elem).unwrap();

        let elem: Element = "<private xmlns='urn:xmpp:carbons:2'/>".parse().unwrap();
        Private::try_from(elem).unwrap();
    }

    #[test]
    fn forwarded_elements() {
        let elem: Element = "<received xmlns='urn:xmpp:carbons:2'>
  <forwarded xmlns='urn:xmpp:forward:0'>
    <message xmlns='jabber:client'
             to='juliet@capulet.example/balcony'
             from='romeo@montague.example/home'/>
  </forwarded>
</received>"
            .parse()
            .unwrap();
        let received = Received::try_from(elem).unwrap();
        assert!(received.forwarded.stanza.is_some());

        let elem: Element = "<sent xmlns='urn:xmpp:carbons:2'>
  <forwarded xmlns='urn:xmpp:forward:0'>
    <message xmlns='jabber:client'
             to='juliet@capulet.example/balcony'
             from='romeo@montague.example/home'/>
  </forwarded>
</sent>"
            .parse()
            .unwrap();
        let sent = Sent::try_from(elem).unwrap();
        assert!(sent.forwarded.stanza.is_some());
    }

    #[test]
    fn test_serialize_received() {
        let reference: Element = "<received xmlns='urn:xmpp:carbons:2'><forwarded xmlns='urn:xmpp:forward:0'><message xmlns='jabber:client' to='juliet@capulet.example/balcony' from='romeo@montague.example/home'/></forwarded></received>"
        .parse()
        .unwrap();

        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'><message xmlns='jabber:client' to='juliet@capulet.example/balcony' from='romeo@montague.example/home'/></forwarded>"
          .parse()
          .unwrap();
        let forwarded = Forwarded::try_from(elem).unwrap();

        let received = Received {
            forwarded: forwarded,
        };

        let serialized: Element = received.into();
        assert_eq!(serialized, reference);
    }

    #[test]
    fn test_serialize_sent() {
        let reference: Element = "<sent xmlns='urn:xmpp:carbons:2'><forwarded xmlns='urn:xmpp:forward:0'><message xmlns='jabber:client' to='juliet@capulet.example/balcony' from='romeo@montague.example/home'/></forwarded></sent>"
        .parse()
        .unwrap();

        let elem: Element = "<forwarded xmlns='urn:xmpp:forward:0'><message xmlns='jabber:client' to='juliet@capulet.example/balcony' from='romeo@montague.example/home'/></forwarded>"
          .parse()
          .unwrap();
        let forwarded = Forwarded::try_from(elem).unwrap();

        let sent = Sent {
            forwarded: forwarded,
        };

        let serialized: Element = sent.into();
        assert_eq!(serialized, reference);
    }
}
