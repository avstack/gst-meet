// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::message::MessagePayload;
use jid::Jid;

generate_element!(
    /// Gives the identifier a service has stamped on this stanza, often in
    /// order to identify it inside of [an archive](../mam/index.html).
    StanzaId, "stanza-id", SID,
    attributes: [
        /// The id associated to this stanza by another entity.
        id: Required<String> = "id",

        /// The entity who stamped this stanza-id.
        by: Required<Jid> = "by",
    ]
);

impl MessagePayload for StanzaId {}

generate_element!(
    /// A hack for MUC before version 1.31 to track a message which may have
    /// its 'id' attribute changed.
    OriginId, "origin-id", SID,
    attributes: [
        /// The id this client set for this stanza.
        id: Required<String> = "id",
    ]
);

impl MessagePayload for OriginId {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::error::Error;
    use crate::Element;
    use jid::BareJid;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(StanzaId, 52);
        assert_size!(OriginId, 12);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(StanzaId, 104);
        assert_size!(OriginId, 24);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0' id='coucou' by='coucou@coucou'/>"
            .parse()
            .unwrap();
        let stanza_id = StanzaId::try_from(elem).unwrap();
        assert_eq!(stanza_id.id, String::from("coucou"));
        assert_eq!(stanza_id.by, BareJid::new("coucou", "coucou"));

        let elem: Element = "<origin-id xmlns='urn:xmpp:sid:0' id='coucou'/>"
            .parse()
            .unwrap();
        let origin_id = OriginId::try_from(elem).unwrap();
        assert_eq!(origin_id.id, String::from("coucou"));
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0'><coucou/></stanza-id>"
            .parse()
            .unwrap();
        let error = StanzaId::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in stanza-id element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0'/>".parse().unwrap();
        let error = StanzaId::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'id' missing.");
    }

    #[test]
    fn test_invalid_by() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0' id='coucou'/>"
            .parse()
            .unwrap();
        let error = StanzaId::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'by' missing.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<stanza-id xmlns='urn:xmpp:sid:0' id='coucou' by='coucou@coucou'/>"
            .parse()
            .unwrap();
        let stanza_id = StanzaId {
            id: String::from("coucou"),
            by: Jid::Bare(BareJid::new("coucou", "coucou")),
        };
        let elem2 = stanza_id.into();
        assert_eq!(elem, elem2);
    }
}
