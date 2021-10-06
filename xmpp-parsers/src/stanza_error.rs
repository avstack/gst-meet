// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::message::MessagePayload;
use crate::ns;
use crate::presence::PresencePayload;
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use std::collections::BTreeMap;
use std::convert::TryFrom;

generate_attribute!(
    /// The type of the error.
    ErrorType, "type", {
        /// Retry after providing credentials.
        Auth => "auth",

        /// Do not retry (the error cannot be remedied).
        Cancel => "cancel",

        /// Proceed (the condition was only a warning).
        Continue => "continue",

        /// Retry after changing the data sent.
        Modify => "modify",

        /// Retry after waiting (the error is temporary).
        Wait => "wait",
    }
);

generate_element_enum!(
    /// List of valid error conditions.
    DefinedCondition, "condition", XMPP_STANZAS, {
        /// The sender has sent a stanza containing XML that does not conform
        /// to the appropriate schema or that cannot be processed (e.g., an IQ
        /// stanza that includes an unrecognized value of the 'type' attribute,
        /// or an element that is qualified by a recognized namespace but that
        /// violates the defined syntax for the element); the associated error
        /// type SHOULD be "modify".
        BadRequest => "bad-request",

        /// Access cannot be granted because an existing resource exists with
        /// the same name or address; the associated error type SHOULD be
        /// "cancel".
        Conflict => "conflict",

        /// The feature represented in the XML stanza is not implemented by the
        /// intended recipient or an intermediate server and therefore the
        /// stanza cannot be processed (e.g., the entity understands the
        /// namespace but does not recognize the element name); the associated
        /// error type SHOULD be "cancel" or "modify".
        FeatureNotImplemented => "feature-not-implemented",

        /// The requesting entity does not possess the necessary permissions to
        /// perform an action that only certain authorized roles or individuals
        /// are allowed to complete (i.e., it typically relates to
        /// authorization rather than authentication); the associated error
        /// type SHOULD be "auth".
        Forbidden => "forbidden",

        /// The recipient or server can no longer be contacted at this address,
        /// typically on a permanent basis (as opposed to the <redirect/> error
        /// condition, which is used for temporary addressing failures); the
        /// associated error type SHOULD be "cancel" and the error stanza
        /// SHOULD include a new address (if available) as the XML character
        /// data of the <gone/> element (which MUST be a Uniform Resource
        /// Identifier [URI] or Internationalized Resource Identifier [IRI] at
        /// which the entity can be contacted, typically an XMPP IRI as
        /// specified in [XMPP‑URI]).
        Gone => "gone",

        /// The server has experienced a misconfiguration or other internal
        /// error that prevents it from processing the stanza; the associated
        /// error type SHOULD be "cancel".
        InternalServerError => "internal-server-error",

        /// The addressed JID or item requested cannot be found; the associated
        /// error type SHOULD be "cancel".
        ItemNotFound => "item-not-found",

        /// The sending entity has provided (e.g., during resource binding) or
        /// communicated (e.g., in the 'to' address of a stanza) an XMPP
        /// address or aspect thereof that violates the rules defined in
        /// [XMPP‑ADDR]; the associated error type SHOULD be "modify".
        JidMalformed => "jid-malformed",

        /// The recipient or server understands the request but cannot process
        /// it because the request does not meet criteria defined by the
        /// recipient or server (e.g., a request to subscribe to information
        /// that does not simultaneously include configuration parameters
        /// needed by the recipient); the associated error type SHOULD be
        /// "modify".
        NotAcceptable => "not-acceptable",

        /// The recipient or server does not allow any entity to perform the
        /// action (e.g., sending to entities at a blacklisted domain); the
        /// associated error type SHOULD be "cancel".
        NotAllowed => "not-allowed",

        /// The sender needs to provide credentials before being allowed to
        /// perform the action, or has provided improper credentials (the name
        /// "not-authorized", which was borrowed from the "401 Unauthorized"
        /// error of [HTTP], might lead the reader to think that this condition
        /// relates to authorization, but instead it is typically used in
        /// relation to authentication); the associated error type SHOULD be
        /// "auth".
        NotAuthorized => "not-authorized",

        /// The entity has violated some local service policy (e.g., a message
        /// contains words that are prohibited by the service) and the server
        /// MAY choose to specify the policy in the <text/> element or in an
        /// application-specific condition element; the associated error type
        /// SHOULD be "modify" or "wait" depending on the policy being
        /// violated.
        PolicyViolation => "policy-violation",

        /// The intended recipient is temporarily unavailable, undergoing
        /// maintenance, etc.; the associated error type SHOULD be "wait".
        RecipientUnavailable => "recipient-unavailable",

        /// The recipient or server is redirecting requests for this
        /// information to another entity, typically in a temporary fashion (as
        /// opposed to the <gone/> error condition, which is used for permanent
        /// addressing failures); the associated error type SHOULD be "modify"
        /// and the error stanza SHOULD contain the alternate address in the
        /// XML character data of the <redirect/> element (which MUST be a URI
        /// or IRI with which the sender can communicate, typically an XMPP IRI
        /// as specified in [XMPP‑URI]).
        Redirect => "redirect",

        /// The requesting entity is not authorized to access the requested
        /// service because prior registration is necessary (examples of prior
        /// registration include members-only rooms in XMPP multi-user chat
        /// [XEP‑0045] and gateways to non-XMPP instant messaging services,
        /// which traditionally required registration in order to use the
        /// gateway [XEP‑0100]); the associated error type SHOULD be "auth".
        RegistrationRequired => "registration-required",

        /// A remote server or service specified as part or all of the JID of
        /// the intended recipient does not exist or cannot be resolved (e.g.,
        /// there is no _xmpp-server._tcp DNS SRV record, the A or AAAA
        /// fallback resolution fails, or A/AAAA lookups succeed but there is
        /// no response on the IANA-registered port 5269); the associated error
        /// type SHOULD be "cancel".
        RemoteServerNotFound => "remote-server-not-found",

        /// A remote server or service specified as part or all of the JID of
        /// the intended recipient (or needed to fulfill a request) was
        /// resolved but communications could not be established within a
        /// reasonable amount of time (e.g., an XML stream cannot be
        /// established at the resolved IP address and port, or an XML stream
        /// can be established but stream negotiation fails because of problems
        /// with TLS, SASL, Server Dialback, etc.); the associated error type
        /// SHOULD be "wait" (unless the error is of a more permanent nature,
        /// e.g., the remote server is found but it cannot be authenticated or
        /// it violates security policies).
        RemoteServerTimeout => "remote-server-timeout",

        /// The server or recipient is busy or lacks the system resources
        /// necessary to service the request; the associated error type SHOULD
        /// be "wait".
        ResourceConstraint => "resource-constraint",

        /// The server or recipient does not currently provide the requested
        /// service; the associated error type SHOULD be "cancel".
        ServiceUnavailable => "service-unavailable",

        /// The requesting entity is not authorized to access the requested
        /// service because a prior subscription is necessary (examples of
        /// prior subscription include authorization to receive presence
        /// information as defined in [XMPP‑IM] and opt-in data feeds for XMPP
        /// publish-subscribe as defined in [XEP‑0060]); the associated error
        /// type SHOULD be "auth".
        SubscriptionRequired => "subscription-required",

        /// The error condition is not one of those defined by the other
        /// conditions in this list; any error type can be associated with this
        /// condition, and it SHOULD NOT be used except in conjunction with an
        /// application-specific condition.
        UndefinedCondition => "undefined-condition",

        /// The recipient or server understood the request but was not
        /// expecting it at this time (e.g., the request was out of order); the
        /// associated error type SHOULD be "wait" or "modify".
        UnexpectedRequest => "unexpected-request",
    }
);

type Lang = String;

/// The representation of a stanza error.
#[derive(Debug, Clone)]
pub struct StanzaError {
    /// The type of this error.
    pub type_: ErrorType,

    /// The JID of the entity who set this error.
    pub by: Option<Jid>,

    /// One of the defined conditions for this error to happen.
    pub defined_condition: DefinedCondition,

    /// Human-readable description of this error.
    pub texts: BTreeMap<Lang, String>,

    /// A protocol-specific extension for this error.
    pub other: Option<Element>,
}

impl MessagePayload for StanzaError {}
impl PresencePayload for StanzaError {}

impl StanzaError {
    /// Create a new `<error/>` with the according content.
    pub fn new<L, T>(
        type_: ErrorType,
        defined_condition: DefinedCondition,
        lang: L,
        text: T,
    ) -> StanzaError
    where
        L: Into<Lang>,
        T: Into<String>,
    {
        StanzaError {
            type_,
            by: None,
            defined_condition,
            texts: {
                let mut map = BTreeMap::new();
                map.insert(lang.into(), text.into());
                map
            },
            other: None,
        }
    }
}

impl TryFrom<Element> for StanzaError {
    type Error = Error;

    fn try_from(elem: Element) -> Result<StanzaError, Error> {
        check_self!(elem, "error", DEFAULT_NS);
        check_no_unknown_attributes!(elem, "error", ["type", "by"]);

        let mut stanza_error = StanzaError {
            type_: get_attr!(elem, "type", Required),
            by: get_attr!(elem, "by", Option),
            defined_condition: DefinedCondition::UndefinedCondition,
            texts: BTreeMap::new(),
            other: None,
        };
        let mut defined_condition = None;

        for child in elem.children() {
            if child.is("text", ns::XMPP_STANZAS) {
                check_no_children!(child, "text");
                check_no_unknown_attributes!(child, "text", ["xml:lang"]);
                let lang = get_attr!(elem, "xml:lang", Default);
                if stanza_error.texts.insert(lang, child.text()).is_some() {
                    return Err(Error::ParseError(
                        "Text element present twice for the same xml:lang.",
                    ));
                }
            } else if child.has_ns(ns::XMPP_STANZAS) {
                if defined_condition.is_some() {
                    return Err(Error::ParseError(
                        "Error must not have more than one defined-condition.",
                    ));
                }
                check_no_children!(child, "defined-condition");
                check_no_attributes!(child, "defined-condition");
                let condition = DefinedCondition::try_from(child.clone())?;
                defined_condition = Some(condition);
            } else {
                if stanza_error.other.is_some() {
                    return Err(Error::ParseError(
                        "Error must not have more than one other element.",
                    ));
                }
                stanza_error.other = Some(child.clone());
            }
        }
        stanza_error.defined_condition =
            defined_condition.ok_or(Error::ParseError("Error must have a defined-condition."))?;

        Ok(stanza_error)
    }
}

impl From<StanzaError> for Element {
    fn from(err: StanzaError) -> Element {
        Element::builder("error", ns::DEFAULT_NS)
            .attr("type", err.type_)
            .attr("by", err.by)
            .append(err.defined_condition)
            .append_all(err.texts.into_iter().map(|(lang, text)| {
                Element::builder("text", ns::XMPP_STANZAS)
                    .attr("xml:lang", lang)
                    .append(text)
            }))
            .append_all(err.other)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(ErrorType, 1);
        assert_size!(DefinedCondition, 1);
        assert_size!(StanzaError, 132);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(ErrorType, 1);
        assert_size!(DefinedCondition, 1);
        assert_size!(StanzaError, 264);
    }

    #[test]
    fn test_simple() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<error xmlns='jabber:client' type='cancel'><undefined-condition xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/></error>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<error xmlns='jabber:component:accept' type='cancel'><undefined-condition xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/></error>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap();
        assert_eq!(error.type_, ErrorType::Cancel);
        assert_eq!(
            error.defined_condition,
            DefinedCondition::UndefinedCondition
        );
    }

    #[test]
    fn test_invalid_type() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<error xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<error xmlns='jabber:component:accept'/>".parse().unwrap();
        let error = StanzaError::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");

        #[cfg(not(feature = "component"))]
        let elem: Element = "<error xmlns='jabber:client' type='coucou'/>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<error xmlns='jabber:component:accept' type='coucou'/>"
            .parse()
            .unwrap();
        let error = StanzaError::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'type' attribute.");
    }

    #[test]
    fn test_invalid_condition() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<error xmlns='jabber:client' type='cancel'/>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<error xmlns='jabber:component:accept' type='cancel'/>"
            .parse()
            .unwrap();
        let error = StanzaError::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Error must have a defined-condition.");
    }
}
