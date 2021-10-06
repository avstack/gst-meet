// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::message::MessagePayload;
use crate::presence::PresencePayload;

generate_element!(
    /// Unique identifier given to a MUC participant.
    ///
    /// It allows clients to identify a MUC participant across reconnects and
    /// renames. It thus prevents impersonification of anonymous users.
    OccupantId, "occupant-id", OID,

    attributes: [
        /// The id associated to the sending user by the MUC service.
        id: Required<String> = "id",
    ]
);

impl MessagePayload for OccupantId {}
impl PresencePayload for OccupantId {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(OccupantId, 12);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(OccupantId, 24);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<occupant-id xmlns='urn:xmpp:occupant-id:0' id='coucou'/>"
            .parse()
            .unwrap();
        let origin_id = OccupantId::try_from(elem).unwrap();
        assert_eq!(origin_id.id, "coucou");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<occupant-id xmlns='urn:xmpp:occupant-id:0'><coucou/></occupant-id>"
            .parse()
            .unwrap();
        let error = OccupantId::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in occupant-id element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<occupant-id xmlns='urn:xmpp:occupant-id:0'/>"
            .parse()
            .unwrap();
        let error = OccupantId::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'id' missing.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<occupant-id xmlns='urn:xmpp:occupant-id:0' id='coucou'/>"
            .parse()
            .unwrap();
        let occupant_id = OccupantId {
            id: String::from("coucou"),
        };
        let elem2 = occupant_id.into();
        assert_eq!(elem, elem2);
    }
}
