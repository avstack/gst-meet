// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

generate_elem_id!(
    /// Represents a global, memorable, friendly or informal name chosen by a user.
    Nick,
    "nick",
    NICK
);

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "disable-validation"))]
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Nick, 12);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Nick, 24);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<nick xmlns='http://jabber.org/protocol/nick'>Link Mauve</nick>"
            .parse()
            .unwrap();
        let nick = Nick::try_from(elem).unwrap();
        assert_eq!(&nick.0, "Link Mauve");
    }

    #[test]
    fn test_serialise() {
        let elem1 = Element::from(Nick(String::from("Link Mauve")));
        let elem2: Element = "<nick xmlns='http://jabber.org/protocol/nick'>Link Mauve</nick>"
            .parse()
            .unwrap();
        assert_eq!(elem1, elem2);
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid() {
        let elem: Element = "<nick xmlns='http://jabber.org/protocol/nick'><coucou/></nick>"
            .parse()
            .unwrap();
        let error = Nick::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in nick element.");
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "<nick xmlns='http://jabber.org/protocol/nick' coucou=''/>"
            .parse()
            .unwrap();
        let error = Nick::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in nick element.");
    }
}
