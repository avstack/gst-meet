// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::hashes::{Algo, Hash};
use crate::util::error::Error;
use crate::util::helpers::ColonSeparatedHex;

generate_attribute!(
    /// Indicates which of the end points should initiate the TCP connection establishment.
    Setup, "setup", {
        /// The endpoint will initiate an outgoing connection.
        Active => "active",

        /// The endpoint will accept an incoming connection.
        Passive => "passive",

        /// The endpoint is willing to accept an incoming connection or to initiate an outgoing
        /// connection.
        Actpass => "actpass",

        /*
        /// The endpoint does not want the connection to be established for the time being.
        ///
        /// Note that this value isn’t used, as per the XEP.
        Holdconn => "holdconn",
        */
    }
);

// TODO: use a hashes::Hash instead of two different fields here.
generate_element!(
    /// Fingerprint of the key used for a DTLS handshake.
    Fingerprint, "fingerprint", JINGLE_DTLS,
    attributes: [
        /// The hash algorithm used for this fingerprint.
        hash: Required<Algo> = "hash",

        /// Indicates which of the end points should initiate the TCP connection establishment.
        setup: Option<Setup> = "setup",

        /// Indicates whether DTLS is mandatory
        required: Option<String> = "required"
    ],
    text: (
        /// Hash value of this fingerprint.
        value: ColonSeparatedHex<Vec<u8>>
    )
);

impl Fingerprint {
    /// Create a new Fingerprint from a Setup and a Hash.
    pub fn from_hash(setup: Setup, hash: Hash) -> Fingerprint {
        Fingerprint {
            hash: hash.algo,
            setup: Some(setup),
            value: hash.hash,
            required: None,
        }
    }

    /// Create a new Fingerprint from a Setup and parsing the hash.
    pub fn from_colon_separated_hex(
        setup: Setup,
        algo: &str,
        hash: &str,
    ) -> Result<Fingerprint, Error> {
        let algo = algo.parse()?;
        let hash = Hash::from_colon_separated_hex(algo, hash)?;
        Ok(Fingerprint::from_hash(setup, hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Setup, 1);
        assert_size!(Fingerprint, 32);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Setup, 1);
        assert_size!(Fingerprint, 64);
    }

    #[test]
    fn test_ex1() {
        let elem: Element = "<fingerprint xmlns='urn:xmpp:jingle:apps:dtls:0' hash='sha-256' setup='actpass'>02:1A:CC:54:27:AB:EB:9C:53:3F:3E:4B:65:2E:7D:46:3F:54:42:CD:54:F1:7A:03:A2:7D:F9:B0:7F:46:19:B2</fingerprint>"
                .parse()
                .unwrap();
        let fingerprint = Fingerprint::try_from(elem).unwrap();
        assert_eq!(fingerprint.setup, Some(Setup::Actpass));
        assert_eq!(fingerprint.hash, Algo::Sha_256);
        assert_eq!(
            fingerprint.value,
            [
                2, 26, 204, 84, 39, 171, 235, 156, 83, 63, 62, 75, 101, 46, 125, 70, 63, 84, 66,
                205, 84, 241, 122, 3, 162, 125, 249, 176, 127, 70, 25, 178
            ]
        );
    }
}
