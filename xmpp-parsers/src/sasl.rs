// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ns;
use crate::util::error::Error;
use crate::util::helpers::Base64;
use crate::Element;
use std::collections::BTreeMap;
use std::convert::TryFrom;

generate_attribute!(
    /// The list of available SASL mechanisms.
    Mechanism, "mechanism", {
        /// Uses no hashing mechanism and transmit the password in clear to the
        /// server, using a single step.
        Plain => "PLAIN",

        /// Challenge-based mechanism using HMAC and SHA-1, allows both the
        /// client and the server to avoid having to store the password in
        /// clear.
        ///
        /// See https://tools.ietf.org/html/rfc5802
        ScramSha1 => "SCRAM-SHA-1",

        /// Same as [ScramSha1](#structfield.ScramSha1), with the addition of
        /// channel binding.
        ScramSha1Plus => "SCRAM-SHA-1-PLUS",

        /// Same as [ScramSha1](#structfield.ScramSha1), but using SHA-256
        /// instead of SHA-1 as the hash function.
        ScramSha256 => "SCRAM-SHA-256",

        /// Same as [ScramSha256](#structfield.ScramSha256), with the addition
        /// of channel binding.
        ScramSha256Plus => "SCRAM-SHA-256-PLUS",

        /// Creates a temporary JID on login, which will be destroyed on
        /// disconnect.
        Anonymous => "ANONYMOUS",
    }
);

generate_element!(
    /// The first step of the SASL process, selecting the mechanism and sending
    /// the first part of the handshake.
    Auth, "auth", SASL,
    attributes: [
        /// The mechanism used.
        mechanism: Required<Mechanism> = "mechanism"
    ],
    text: (
        /// The content of the handshake.
        data: Base64<Vec<u8>>
    )
);

generate_element!(
    /// In case the mechanism selected at the [auth](struct.Auth.html) step
    /// requires a second step, the server sends this element with additional
    /// data.
    Challenge, "challenge", SASL,
    text: (
        /// The challenge data.
        data: Base64<Vec<u8>>
    )
);

generate_element!(
    /// In case the mechanism selected at the [auth](struct.Auth.html) step
    /// requires a second step, this contains the client’s response to the
    /// server’s [challenge](struct.Challenge.html).
    Response, "response", SASL,
    text: (
        /// The response data.
        data: Base64<Vec<u8>>
    )
);

generate_empty_element!(
    /// Sent by the client at any point after [auth](struct.Auth.html) if it
    /// wants to cancel the current authentication process.
    Abort,
    "abort",
    SASL
);

generate_element!(
    /// Sent by the server on SASL success.
    Success, "success", SASL,
    text: (
        /// Possible data sent on success.
        data: Base64<Vec<u8>>
    )
);

generate_element_enum!(
    /// List of possible failure conditions for SASL.
    DefinedCondition, "defined-condition", SASL, {
        /// The client aborted the authentication with
        /// [abort](struct.Abort.html).
        Aborted => "aborted",

        /// The account the client is trying to authenticate against has been
        /// disabled.
        AccountDisabled => "account-disabled",

        /// The credentials for this account have expired.
        CredentialsExpired => "credentials-expired",

        /// You must enable StartTLS or use direct TLS before using this
        /// authentication mechanism.
        EncryptionRequired => "encryption-required",

        /// The base64 data sent by the client is invalid.
        IncorrectEncoding => "incorrect-encoding",

        /// The authzid provided by the client is invalid.
        InvalidAuthzid => "invalid-authzid",

        /// The client tried to use an invalid mechanism, or none.
        InvalidMechanism => "invalid-mechanism",

        /// The client sent a bad request.
        MalformedRequest => "malformed-request",

        /// The mechanism selected is weaker than what the server allows.
        MechanismTooWeak => "mechanism-too-weak",

        /// The credentials provided are invalid.
        NotAuthorized => "not-authorized",

        /// The server encountered an issue which may be fixed later, the
        /// client should retry at some point.
        TemporaryAuthFailure => "temporary-auth-failure",
    }
);

type Lang = String;

/// Sent by the server on SASL failure.
#[derive(Debug, Clone)]
pub struct Failure {
    /// One of the allowed defined-conditions for SASL.
    pub defined_condition: DefinedCondition,

    /// A human-readable explanation for the failure.
    pub texts: BTreeMap<Lang, String>,
}

impl TryFrom<Element> for Failure {
    type Error = Error;

    fn try_from(root: Element) -> Result<Failure, Error> {
        check_self!(root, "failure", SASL);
        check_no_attributes!(root, "failure");

        let mut defined_condition = None;
        let mut texts = BTreeMap::new();

        for child in root.children() {
            if child.is("text", ns::SASL) {
                check_no_unknown_attributes!(child, "text", ["xml:lang"]);
                check_no_children!(child, "text");
                let lang = get_attr!(child, "xml:lang", Default);
                if texts.insert(lang, child.text()).is_some() {
                    return Err(Error::ParseError(
                        "Text element present twice for the same xml:lang in failure element.",
                    ));
                }
            } else if child.has_ns(ns::SASL) {
                if defined_condition.is_some() {
                    return Err(Error::ParseError(
                        "Failure must not have more than one defined-condition.",
                    ));
                }
                check_no_attributes!(child, "defined-condition");
                check_no_children!(child, "defined-condition");
                let condition = match DefinedCondition::try_from(child.clone()) {
                    Ok(condition) => condition,
                    // TODO: do we really want to eat this error?
                    Err(_) => DefinedCondition::NotAuthorized,
                };
                defined_condition = Some(condition);
            } else {
                return Err(Error::ParseError("Unknown element in Failure."));
            }
        }
        let defined_condition =
            defined_condition.ok_or(Error::ParseError("Failure must have a defined-condition."))?;

        Ok(Failure {
            defined_condition,
            texts,
        })
    }
}

impl From<Failure> for Element {
    fn from(failure: Failure) -> Element {
        Element::builder("failure", ns::SASL)
            .append(failure.defined_condition)
            .append_all(failure.texts.into_iter().map(|(lang, text)| {
                Element::builder("text", ns::SASL)
                    .attr("xml:lang", lang)
                    .append(text)
            }))
            .build()
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
        assert_size!(Mechanism, 1);
        assert_size!(Auth, 16);
        assert_size!(Challenge, 12);
        assert_size!(Response, 12);
        assert_size!(Abort, 0);
        assert_size!(Success, 12);
        assert_size!(DefinedCondition, 1);
        assert_size!(Failure, 16);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Mechanism, 1);
        assert_size!(Auth, 32);
        assert_size!(Challenge, 24);
        assert_size!(Response, 24);
        assert_size!(Abort, 0);
        assert_size!(Success, 24);
        assert_size!(DefinedCondition, 1);
        assert_size!(Failure, 32);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<auth xmlns='urn:ietf:params:xml:ns:xmpp-sasl' mechanism='PLAIN'/>"
            .parse()
            .unwrap();
        let auth = Auth::try_from(elem).unwrap();
        assert_eq!(auth.mechanism, Mechanism::Plain);
        assert!(auth.data.is_empty());
    }

    #[test]
    fn section_6_5_1() {
        let elem: Element =
            "<failure xmlns='urn:ietf:params:xml:ns:xmpp-sasl'><aborted/></failure>"
                .parse()
                .unwrap();
        let failure = Failure::try_from(elem).unwrap();
        assert_eq!(failure.defined_condition, DefinedCondition::Aborted);
        assert!(failure.texts.is_empty());
    }

    #[test]
    fn section_6_5_2() {
        let elem: Element = "<failure xmlns='urn:ietf:params:xml:ns:xmpp-sasl'>
            <account-disabled/>
            <text xml:lang='en'>Call 212-555-1212 for assistance.</text>
        </failure>"
            .parse()
            .unwrap();
        let failure = Failure::try_from(elem).unwrap();
        assert_eq!(failure.defined_condition, DefinedCondition::AccountDisabled);
        assert_eq!(
            failure.texts["en"],
            String::from("Call 212-555-1212 for assistance.")
        );
    }

    /// Some servers apparently use a non-namespaced 'lang' attribute, which is invalid as not part
    /// of the schema.  This tests whether we can parse it when disabling validation.
    #[cfg(feature = "disable-validation")]
    #[test]
    fn invalid_failure_with_non_prefixed_text_lang() {
        let elem: Element = "<failure xmlns='urn:ietf:params:xml:ns:xmpp-sasl'>
            <not-authorized xmlns='urn:ietf:params:xml:ns:xmpp-sasl'/>
            <text xmlns='urn:ietf:params:xml:ns:xmpp-sasl' lang='en'>Invalid username or password</text>
        </failure>"
            .parse()
            .unwrap();
        let failure = Failure::try_from(elem).unwrap();
        assert_eq!(failure.defined_condition, DefinedCondition::NotAuthorized);
        assert_eq!(
            failure.texts[""],
            String::from("Invalid username or password")
        );
    }
}
