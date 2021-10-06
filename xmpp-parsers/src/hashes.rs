// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::error::Error;
use crate::util::helpers::Base64;
use minidom::IntoAttributeValue;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// List of the algorithms we support, or Unknown.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Algo {
    /// The Secure Hash Algorithm 1, with known vulnerabilities, do not use it.
    ///
    /// See https://tools.ietf.org/html/rfc3174
    Sha_1,

    /// The Secure Hash Algorithm 2, in its 256-bit version.
    ///
    /// See https://tools.ietf.org/html/rfc6234
    Sha_256,

    /// The Secure Hash Algorithm 2, in its 512-bit version.
    ///
    /// See https://tools.ietf.org/html/rfc6234
    Sha_512,

    /// The Secure Hash Algorithm 3, based on Keccak, in its 256-bit version.
    ///
    /// See https://keccak.team/files/Keccak-submission-3.pdf
    Sha3_256,

    /// The Secure Hash Algorithm 3, based on Keccak, in its 512-bit version.
    ///
    /// See https://keccak.team/files/Keccak-submission-3.pdf
    Sha3_512,

    /// The BLAKE2 hash algorithm, for a 256-bit output.
    ///
    /// See https://tools.ietf.org/html/rfc7693
    Blake2b_256,

    /// The BLAKE2 hash algorithm, for a 512-bit output.
    ///
    /// See https://tools.ietf.org/html/rfc7693
    Blake2b_512,

    /// An unknown hash not in this list, you can probably reject it.
    Unknown(String),
}

impl FromStr for Algo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Algo, Error> {
        Ok(match s {
            "" => return Err(Error::ParseError("'algo' argument can’t be empty.")),

            "sha-1" => Algo::Sha_1,
            "sha-256" => Algo::Sha_256,
            "sha-512" => Algo::Sha_512,
            "sha3-256" => Algo::Sha3_256,
            "sha3-512" => Algo::Sha3_512,
            "blake2b-256" => Algo::Blake2b_256,
            "blake2b-512" => Algo::Blake2b_512,
            value => Algo::Unknown(value.to_owned()),
        })
    }
}

impl From<Algo> for String {
    fn from(algo: Algo) -> String {
        String::from(match algo {
            Algo::Sha_1 => "sha-1",
            Algo::Sha_256 => "sha-256",
            Algo::Sha_512 => "sha-512",
            Algo::Sha3_256 => "sha3-256",
            Algo::Sha3_512 => "sha3-512",
            Algo::Blake2b_256 => "blake2b-256",
            Algo::Blake2b_512 => "blake2b-512",
            Algo::Unknown(text) => return text,
        })
    }
}

impl IntoAttributeValue for Algo {
    fn into_attribute_value(self) -> Option<String> {
        Some(String::from(self))
    }
}

generate_element!(
    /// This element represents a hash of some data, defined by the hash
    /// algorithm used and the computed value.
    Hash, "hash", HASHES,
    attributes: [
        /// The algorithm used to create this hash.
        algo: Required<Algo> = "algo"
    ],
    text: (
        /// The hash value, as a vector of bytes.
        hash: Base64<Vec<u8>>
    )
);

impl Hash {
    /// Creates a [Hash] element with the given algo and data.
    pub fn new(algo: Algo, hash: Vec<u8>) -> Hash {
        Hash { algo, hash }
    }

    /// Like [new](#method.new) but takes base64-encoded data before decoding
    /// it.
    pub fn from_base64(algo: Algo, hash: &str) -> Result<Hash, Error> {
        Ok(Hash::new(algo, base64::decode(hash)?))
    }

    /// Like [new](#method.new) but takes hex-encoded data before decoding it.
    pub fn from_hex(algo: Algo, hex: &str) -> Result<Hash, ParseIntError> {
        let mut bytes = vec![];
        for i in 0..hex.len() / 2 {
            let byte = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)?;
            bytes.push(byte);
        }
        Ok(Hash::new(algo, bytes))
    }

    /// Like [new](#method.new) but takes hex-encoded data before decoding it.
    pub fn from_colon_separated_hex(algo: Algo, hex: &str) -> Result<Hash, ParseIntError> {
        let mut bytes = vec![];
        for i in 0..(1 + hex.len()) / 3 {
            let byte = u8::from_str_radix(&hex[3 * i..3 * i + 2], 16)?;
            if 3 * i + 2 < hex.len() {
                assert_eq!(&hex[3 * i + 2..3 * i + 3], ":");
            }
            bytes.push(byte);
        }
        Ok(Hash::new(algo, bytes))
    }

    /// Formats this hash into base64.
    pub fn to_base64(&self) -> String {
        base64::encode(&self.hash[..])
    }

    /// Formats this hash into hexadecimal.
    pub fn to_hex(&self) -> String {
        self.hash
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<_>>()
            .join("")
    }

    /// Formats this hash into colon-separated hexadecimal.
    pub fn to_colon_separated_hex(&self) -> String {
        self.hash
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<_>>()
            .join(":")
    }
}

/// Helper for parsing and serialising a SHA-1 attribute.
#[derive(Debug, Clone, PartialEq)]
pub struct Sha1HexAttribute(Hash);

impl FromStr for Sha1HexAttribute {
    type Err = ParseIntError;

    fn from_str(hex: &str) -> Result<Self, Self::Err> {
        let hash = Hash::from_hex(Algo::Sha_1, hex)?;
        Ok(Sha1HexAttribute(hash))
    }
}

impl IntoAttributeValue for Sha1HexAttribute {
    fn into_attribute_value(self) -> Option<String> {
        Some(self.to_hex())
    }
}

impl DerefMut for Sha1HexAttribute {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for Sha1HexAttribute {
    type Target = Hash;

    fn deref(&self) -> &Self::Target {
        &self.0
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
        assert_size!(Algo, 16);
        assert_size!(Hash, 28);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Algo, 32);
        assert_size!(Hash, 56);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<hash xmlns='urn:xmpp:hashes:2' algo='sha-256'>2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=</hash>".parse().unwrap();
        let hash = Hash::try_from(elem).unwrap();
        assert_eq!(hash.algo, Algo::Sha_256);
        assert_eq!(
            hash.hash,
            base64::decode("2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=").unwrap()
        );
    }

    #[test]
    fn value_serialisation() {
        let elem: Element = "<hash xmlns='urn:xmpp:hashes:2' algo='sha-256'>2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU=</hash>".parse().unwrap();
        let hash = Hash::try_from(elem).unwrap();
        assert_eq!(
            hash.to_base64(),
            "2XarmwTlNxDAMkvymloX3S5+VbylNrJt/l5QyPa+YoU="
        );
        assert_eq!(
            hash.to_hex(),
            "d976ab9b04e53710c0324bf29a5a17dd2e7e55bca536b26dfe5e50c8f6be6285"
        );
        assert_eq!(hash.to_colon_separated_hex(), "d9:76:ab:9b:04:e5:37:10:c0:32:4b:f2:9a:5a:17:dd:2e:7e:55:bc:a5:36:b2:6d:fe:5e:50:c8:f6:be:62:85");
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>"
            .parse()
            .unwrap();
        let error = Hash::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a hash element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<hash xmlns='urn:xmpp:hashes:2'><coucou/></hash>"
            .parse()
            .unwrap();
        let error = Hash::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in hash element.");
    }
}
