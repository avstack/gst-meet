// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::date::DateTime;
use crate::presence::PresencePayload;

generate_element!(
    /// Represents the last time the user interacted with their system.
    Idle, "idle", IDLE,
    attributes: [
        /// The time at which the user stopped interacting.
        since: Required<DateTime> = "since",
    ]
);

impl PresencePayload for Idle {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    fn test_size() {
        assert_size!(Idle, 16);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-21T20:19:55+01:00'/>"
            .parse()
            .unwrap();
        Idle::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1'><coucou/></idle>"
            .parse()
            .unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in idle element.");
    }

    #[test]
    fn test_invalid_id() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1'/>".parse().unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'since' missing.");
    }

    #[test]
    fn test_invalid_date() {
        // There is no thirteenth month.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-13-01T12:23:34Z'/>"
            .parse()
            .unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input is out of range");

        // Timezone ≥24:00 aren’t allowed.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-27T12:11:02+25:00'/>"
            .parse()
            .unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input is out of range");

        // Timezone without the : separator aren’t allowed.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-27T12:11:02+0100'/>"
            .parse()
            .unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input contains invalid characters");

        // No seconds, error message could be improved.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-27T12:11+01:00'/>"
            .parse()
            .unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input contains invalid characters");

        // TODO: maybe we’ll want to support this one, as per XEP-0082 §4.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='20170527T12:11:02+01:00'/>"
            .parse()
            .unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input contains invalid characters");

        // No timezone.
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-27T12:11:02'/>"
            .parse()
            .unwrap();
        let error = Idle::try_from(elem).unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "premature end of input");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<idle xmlns='urn:xmpp:idle:1' since='2017-05-21T20:19:55+01:00'/>"
            .parse()
            .unwrap();
        let idle = Idle {
            since: DateTime::from_str("2017-05-21T20:19:55+01:00").unwrap(),
        };
        let elem2 = idle.into();
        assert_eq!(elem, elem2);
    }
}
