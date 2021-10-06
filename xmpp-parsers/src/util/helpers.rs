// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::error::Error;
use jid::Jid;
use std::str::FromStr;

/// Codec for text content.
pub struct Text;

impl Text {
    pub fn decode(s: &str) -> Result<String, Error> {
        Ok(s.to_owned())
    }

    pub fn encode(string: &str) -> Option<String> {
        Some(string.to_owned())
    }
}

/// Codec for plain text content.
pub struct PlainText;

impl PlainText {
    pub fn decode(s: &str) -> Result<Option<String>, Error> {
        Ok(match s {
            "" => None,
            text => Some(text.to_owned()),
        })
    }

    pub fn encode(string: &Option<String>) -> Option<String> {
        string.as_ref().map(ToOwned::to_owned)
    }
}

/// Codec for trimmed plain text content.
pub struct TrimmedPlainText;

impl TrimmedPlainText {
    pub fn decode(s: &str) -> Result<String, Error> {
        Ok(match s.trim() {
            "" => return Err(Error::ParseError("URI missing in uri.")),
            text => text.to_owned(),
        })
    }

    pub fn encode(string: &str) -> Option<String> {
        Some(string.to_owned())
    }
}

/// Codec wrapping base64 encode/decode.
pub struct Base64;

impl Base64 {
    pub fn decode(s: &str) -> Result<Vec<u8>, Error> {
        Ok(base64::decode(s)?)
    }

    pub fn encode(b: &[u8]) -> Option<String> {
        Some(base64::encode(b))
    }
}

/// Codec wrapping base64 encode/decode, while ignoring whitespace characters.
pub struct WhitespaceAwareBase64;

impl WhitespaceAwareBase64 {
    pub fn decode(s: &str) -> Result<Vec<u8>, Error> {
        let s: String = s
            .chars()
            .filter(|ch| *ch != ' ' && *ch != '\n' && *ch != '\t')
            .collect();
        Ok(base64::decode(&s)?)
    }

    pub fn encode(b: &[u8]) -> Option<String> {
        Some(base64::encode(b))
    }
}

/// Codec for colon-separated bytes of uppercase hexadecimal.
pub struct ColonSeparatedHex;

impl ColonSeparatedHex {
    pub fn decode(s: &str) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![];
        for i in 0..(1 + s.len()) / 3 {
            let byte = u8::from_str_radix(&s[3 * i..3 * i + 2], 16)?;
            if 3 * i + 2 < s.len() {
                assert_eq!(&s[3 * i + 2..3 * i + 3], ":");
            }
            bytes.push(byte);
        }
        Ok(bytes)
    }

    pub fn encode(b: &[u8]) -> Option<String> {
        let mut bytes = vec![];
        for byte in b {
            bytes.push(format!("{:02X}", byte));
        }
        Some(bytes.join(":"))
    }
}

/// Codec for a JID.
pub struct JidCodec;

impl JidCodec {
    pub fn decode(s: &str) -> Result<Jid, Error> {
        Ok(Jid::from_str(s)?)
    }

    pub fn encode(jid: &Jid) -> Option<String> {
        Some(jid.to_string())
    }
}
