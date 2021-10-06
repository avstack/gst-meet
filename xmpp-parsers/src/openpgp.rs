// Copyright (c) 2019 Maxime “pep” Buquet <pep@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::date::DateTime;
use crate::pubsub::PubSubPayload;
use crate::util::helpers::Base64;

// TODO: Merge this container with the PubKey struct
generate_element!(
    /// Data contained in the PubKey element
    PubKeyData, "data", OX,
    text: (
        /// Base64 data
        data: Base64<Vec<u8>>
    )
);

generate_element!(
    /// Pubkey element to be used in PubSub publish payloads.
    PubKey, "pubkey", OX,
    attributes: [
        /// Last updated date
        date: Option<DateTime> = "date"
    ],
    children: [
        /// Public key as base64 data
        data: Required<PubKeyData> = ("data", OX) => PubKeyData
    ]
);

impl PubSubPayload for PubKey {}

generate_element!(
    /// Public key metadata
    PubKeyMeta, "pubkey-metadata", OX,
    attributes: [
        /// OpenPGP v4 fingerprint
        v4fingerprint: Required<String> = "v4-fingerprint",
        /// Time the key was published or updated
        date: Required<DateTime> = "date",
    ]
);

generate_element!(
    /// List of public key metadata
    PubKeysMeta, "public-key-list", OX,
    children: [
        /// Public keys
        pubkeys: Vec<PubKeyMeta> = ("pubkey-metadata", OX) => PubKeyMeta
    ]
);

impl PubSubPayload for PubKeysMeta {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ns;
    use crate::pubsub::{
        pubsub::{Item as PubSubItem, Publish},
        Item, NodeName,
    };
    use crate::Element;
    use std::str::FromStr;

    #[test]
    fn pubsub_publish_pubkey_data() {
        let pubkey = PubKey {
            date: None,
            data: PubKeyData {
                data: (&"Foo").as_bytes().to_vec(),
            },
        };
        println!("Foo1: {:?}", pubkey);

        let pubsub = Publish {
            node: NodeName(format!("{}:{}", ns::OX_PUBKEYS, "some-fingerprint")),
            items: vec![PubSubItem(Item::new(None, None, Some(pubkey)))],
        };
        println!("Foo2: {:?}", pubsub);
    }

    #[test]
    fn pubsub_publish_pubkey_meta() {
        let pubkeymeta = PubKeysMeta {
            pubkeys: vec![PubKeyMeta {
                v4fingerprint: "some-fingerprint".to_owned(),
                date: DateTime::from_str("2019-03-30T18:30:25Z").unwrap(),
            }],
        };
        println!("Foo1: {:?}", pubkeymeta);

        let pubsub = Publish {
            node: NodeName("foo".to_owned()),
            items: vec![PubSubItem(Item::new(None, None, Some(pubkeymeta)))],
        };
        println!("Foo2: {:?}", pubsub);
    }

    #[test]
    fn test_serialize_pubkey() {
        let reference: Element = "<pubkey xmlns='urn:xmpp:openpgp:0'><data>AAAA</data></pubkey>"
            .parse()
            .unwrap();

        let pubkey = PubKey {
            date: None,
            data: PubKeyData {
                data: b"\0\0\0".to_vec(),
            },
        };

        let serialized: Element = pubkey.into();
        assert_eq!(serialized, reference);
    }
}
