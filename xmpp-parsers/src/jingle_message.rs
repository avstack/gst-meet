// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::jingle::SessionId;
use crate::ns;
use crate::util::error::Error;
use crate::Element;
use std::convert::TryFrom;

/// Defines a protocol for broadcasting Jingle requests to all of the clients
/// of a user.
#[derive(Debug, Clone)]
pub enum JingleMI {
    /// Indicates we want to start a Jingle session.
    Propose {
        /// The generated session identifier, must be unique between two users.
        sid: SessionId,

        /// The application description of the proposed session.
        // TODO: Use a more specialised type here.
        description: Element,
    },

    /// Cancels a previously proposed session.
    Retract(SessionId),

    /// Accepts a session proposed by the other party.
    Accept(SessionId),

    /// Proceed with a previously proposed session.
    Proceed(SessionId),

    /// Rejects a session proposed by the other party.
    Reject(SessionId),
}

fn get_sid(elem: Element) -> Result<SessionId, Error> {
    check_no_unknown_attributes!(elem, "Jingle message", ["id"]);
    Ok(SessionId(get_attr!(elem, "id", Required)))
}

fn check_empty_and_get_sid(elem: Element) -> Result<SessionId, Error> {
    check_no_children!(elem, "Jingle message");
    get_sid(elem)
}

impl TryFrom<Element> for JingleMI {
    type Error = Error;

    fn try_from(elem: Element) -> Result<JingleMI, Error> {
        if !elem.has_ns(ns::JINGLE_MESSAGE) {
            return Err(Error::ParseError("This is not a Jingle message element."));
        }
        Ok(match elem.name() {
            "propose" => {
                let mut description = None;
                for child in elem.children() {
                    if child.name() != "description" {
                        return Err(Error::ParseError("Unknown child in propose element."));
                    }
                    if description.is_some() {
                        return Err(Error::ParseError("Too many children in propose element."));
                    }
                    description = Some(child.clone());
                }
                JingleMI::Propose {
                    sid: get_sid(elem)?,
                    description: description.ok_or(Error::ParseError(
                        "Propose element doesn’t contain a description.",
                    ))?,
                }
            }
            "retract" => JingleMI::Retract(check_empty_and_get_sid(elem)?),
            "accept" => JingleMI::Accept(check_empty_and_get_sid(elem)?),
            "proceed" => JingleMI::Proceed(check_empty_and_get_sid(elem)?),
            "reject" => JingleMI::Reject(check_empty_and_get_sid(elem)?),
            _ => return Err(Error::ParseError("This is not a Jingle message element.")),
        })
    }
}

impl From<JingleMI> for Element {
    fn from(jingle_mi: JingleMI) -> Element {
        match jingle_mi {
            JingleMI::Propose { sid, description } => {
                Element::builder("propose", ns::JINGLE_MESSAGE)
                    .attr("id", sid)
                    .append(description)
            }
            JingleMI::Retract(sid) => {
                Element::builder("retract", ns::JINGLE_MESSAGE).attr("id", sid)
            }
            JingleMI::Accept(sid) => Element::builder("accept", ns::JINGLE_MESSAGE).attr("id", sid),
            JingleMI::Proceed(sid) => {
                Element::builder("proceed", ns::JINGLE_MESSAGE).attr("id", sid)
            }
            JingleMI::Reject(sid) => Element::builder("reject", ns::JINGLE_MESSAGE).attr("id", sid),
        }
        .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(JingleMI, 92);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(JingleMI, 184);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<accept xmlns='urn:xmpp:jingle-message:0' id='coucou'/>"
            .parse()
            .unwrap();
        JingleMI::try_from(elem).unwrap();
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element =
            "<propose xmlns='urn:xmpp:jingle-message:0' id='coucou'><coucou/></propose>"
                .parse()
                .unwrap();
        let error = JingleMI::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in propose element.");
    }
}
