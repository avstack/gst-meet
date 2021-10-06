// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::iq::{IqGetPayload, IqResultPayload};

generate_empty_element!(
    /// Represents a query for the software version a remote entity is using.
    ///
    /// It should only be used in an `<iq type='get'/>`, as it can only
    /// represent the request, and not a result.
    VersionQuery,
    "query",
    VERSION
);

impl IqGetPayload for VersionQuery {}

generate_element!(
    /// Represents the answer about the software version we are using.
    ///
    /// It should only be used in an `<iq type='result'/>`, as it can only
    /// represent the result, and not a request.
    VersionResult, "query", VERSION,
    children: [
        /// The name of this client.
        name: Required<String> = ("name", VERSION) => String,

        /// The version of this client.
        version: Required<String> = ("version", VERSION) => String,

        /// The OS this client is running on.
        os: Option<String> = ("os", VERSION) => String
    ]
);

impl IqResultPayload for VersionResult {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(VersionQuery, 0);
        assert_size!(VersionResult, 36);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(VersionQuery, 0);
        assert_size!(VersionResult, 72);
    }

    #[test]
    fn simple() {
        let elem: Element =
            "<query xmlns='jabber:iq:version'><name>xmpp-rs</name><version>0.3.0</version></query>"
                .parse()
                .unwrap();
        let version = VersionResult::try_from(elem).unwrap();
        assert_eq!(version.name, String::from("xmpp-rs"));
        assert_eq!(version.version, String::from("0.3.0"));
        assert_eq!(version.os, None);
    }

    #[test]
    fn serialisation() {
        let version = VersionResult {
            name: String::from("xmpp-rs"),
            version: String::from("0.3.0"),
            os: None,
        };
        let elem1 = Element::from(version);
        let elem2: Element =
            "<query xmlns='jabber:iq:version'><name>xmpp-rs</name><version>0.3.0</version></query>"
                .parse()
                .unwrap();
        println!("{:?}", elem1);
        assert_eq!(elem1, elem2);
    }
}
