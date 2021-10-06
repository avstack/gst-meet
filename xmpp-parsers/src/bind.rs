// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::iq::{IqResultPayload, IqSetPayload};
use crate::ns;
use crate::util::error::Error;
use crate::Element;
use jid::{FullJid, Jid};
use std::convert::TryFrom;
use std::str::FromStr;

/// The request for resource binding, which is the process by which a client
/// can obtain a full JID and start exchanging on the XMPP network.
///
/// See https://xmpp.org/rfcs/rfc6120.html#bind
#[derive(Debug, Clone, PartialEq)]
pub struct BindQuery {
    /// Requests this resource, the server may associate another one though.
    ///
    /// If this is None, we request no particular resource, and a random one
    /// will be affected by the server.
    resource: Option<String>,
}

impl BindQuery {
    /// Creates a resource binding request.
    pub fn new(resource: Option<String>) -> BindQuery {
        BindQuery { resource }
    }
}

impl IqSetPayload for BindQuery {}

impl TryFrom<Element> for BindQuery {
    type Error = Error;

    fn try_from(elem: Element) -> Result<BindQuery, Error> {
        check_self!(elem, "bind", BIND);
        check_no_attributes!(elem, "bind");

        let mut resource = None;
        for child in elem.children() {
            if resource.is_some() {
                return Err(Error::ParseError("Bind can only have one child."));
            }
            if child.is("resource", ns::BIND) {
                check_no_attributes!(child, "resource");
                check_no_children!(child, "resource");
                resource = Some(child.text());
            } else {
                return Err(Error::ParseError("Unknown element in bind request."));
            }
        }

        Ok(BindQuery { resource })
    }
}

impl From<BindQuery> for Element {
    fn from(bind: BindQuery) -> Element {
        Element::builder("bind", ns::BIND)
            .append_all(
                bind.resource
                    .map(|resource| Element::builder("resource", ns::BIND).append(resource)),
            )
            .build()
    }
}

/// The response for resource binding, containing the client’s full JID.
///
/// See https://xmpp.org/rfcs/rfc6120.html#bind
#[derive(Debug, Clone, PartialEq)]
pub struct BindResponse {
    /// The full JID returned by the server for this client.
    jid: FullJid,
}

impl IqResultPayload for BindResponse {}

impl From<BindResponse> for FullJid {
    fn from(bind: BindResponse) -> FullJid {
        bind.jid
    }
}

impl From<BindResponse> for Jid {
    fn from(bind: BindResponse) -> Jid {
        Jid::Full(bind.jid)
    }
}

impl TryFrom<Element> for BindResponse {
    type Error = Error;

    fn try_from(elem: Element) -> Result<BindResponse, Error> {
        check_self!(elem, "bind", BIND);
        check_no_attributes!(elem, "bind");

        let mut jid = None;
        for child in elem.children() {
            if jid.is_some() {
                return Err(Error::ParseError("Bind can only have one child."));
            }
            if child.is("jid", ns::BIND) {
                check_no_attributes!(child, "jid");
                check_no_children!(child, "jid");
                jid = Some(FullJid::from_str(&child.text())?);
            } else {
                return Err(Error::ParseError("Unknown element in bind response."));
            }
        }

        Ok(BindResponse {
            jid: match jid {
                None => {
                    return Err(Error::ParseError(
                        "Bind response must contain a jid element.",
                    ))
                }
                Some(jid) => jid,
            },
        })
    }
}

impl From<BindResponse> for Element {
    fn from(bind: BindResponse) -> Element {
        Element::builder("bind", ns::BIND)
            .append(Element::builder("jid", ns::BIND).append(bind.jid))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(BindQuery, 12);
        assert_size!(BindResponse, 36);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(BindQuery, 24);
        assert_size!(BindResponse, 72);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'/>"
            .parse()
            .unwrap();
        let bind = BindQuery::try_from(elem).unwrap();
        assert_eq!(bind.resource, None);

        let elem: Element =
            "<bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'><resource>Hello™</resource></bind>"
                .parse()
                .unwrap();
        let bind = BindQuery::try_from(elem).unwrap();
        // FIXME: “™” should be resourceprep’d into “TM” here…
        //assert_eq!(bind.resource.unwrap(), "HelloTM");
        assert_eq!(bind.resource.unwrap(), "Hello™");

        let elem: Element = "<bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'><jid>coucou@linkmauve.fr/HelloTM</jid></bind>"
            .parse()
            .unwrap();
        let bind = BindResponse::try_from(elem).unwrap();
        assert_eq!(bind.jid, FullJid::new("coucou", "linkmauve.fr", "HelloTM"));
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_resource() {
        let elem: Element = "<bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'><resource attr='coucou'>resource</resource></bind>"
            .parse()
            .unwrap();
        let error = BindQuery::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in resource element.");

        let elem: Element = "<bind xmlns='urn:ietf:params:xml:ns:xmpp-bind'><resource><hello-world/>resource</resource></bind>"
            .parse()
            .unwrap();
        let error = BindQuery::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in resource element.");
    }
}
