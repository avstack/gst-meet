// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ns;
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use std::convert::TryFrom;
use std::net::IpAddr;

generate_attribute!(
    /// The type of the connection being proposed by this candidate.
    Type, "type", {
        /// Direct connection using NAT assisting technologies like NAT-PMP or
        /// UPnP-IGD.
        Assisted => "assisted",

        /// Direct connection using the given interface.
        Direct => "direct",

        /// SOCKS5 relay.
        Proxy => "proxy",

        /// Tunnel protocol such as Teredo.
        Tunnel => "tunnel",
    }, Default = Direct
);

generate_attribute!(
    /// Which mode to use for the connection.
    Mode, "mode", {
        /// Use TCP, which is the default.
        Tcp => "tcp",

        /// Use UDP.
        Udp => "udp",
    }, Default = Tcp
);

generate_id!(
    /// An identifier for a candidate.
    CandidateId
);

generate_id!(
    /// An identifier for a stream.
    StreamId
);

generate_element!(
    /// A candidate for a connection.
    Candidate, "candidate", JINGLE_S5B,
    attributes: [
        /// The identifier for this candidate.
        cid: Required<CandidateId> = "cid",

        /// The host to connect to.
        host: Required<IpAddr> = "host",

        /// The JID to request at the given end.
        jid: Required<Jid> = "jid",

        /// The port to connect to.
        port: Option<u16> = "port",

        /// The priority of this candidate, computed using this formula:
        /// priority = (2^16)*(type preference) + (local preference)
        priority: Required<u32> = "priority",

        /// The type of the connection being proposed by this candidate.
        type_: Default<Type> = "type",
    ]
);

impl Candidate {
    /// Creates a new candidate with the given parameters.
    pub fn new(cid: CandidateId, host: IpAddr, jid: Jid, priority: u32) -> Candidate {
        Candidate {
            cid,
            host,
            jid,
            priority,
            port: Default::default(),
            type_: Default::default(),
        }
    }

    /// Sets the port of this candidate.
    pub fn with_port(mut self, port: u16) -> Candidate {
        self.port = Some(port);
        self
    }

    /// Sets the type of this candidate.
    pub fn with_type(mut self, type_: Type) -> Candidate {
        self.type_ = type_;
        self
    }
}

/// The payload of a transport.
#[derive(Debug, Clone, PartialEq)]
pub enum TransportPayload {
    /// The responder informs the initiator that the bytestream pointed by this
    /// candidate has been activated.
    Activated(CandidateId),

    /// A list of suggested candidates.
    Candidates(Vec<Candidate>),

    /// Both parties failed to use a candidate, they should fallback to another
    /// transport.
    CandidateError,

    /// The candidate pointed here should be used by both parties.
    CandidateUsed(CandidateId),

    /// This entity can’t connect to the SOCKS5 proxy.
    ProxyError,

    /// XXX: Invalid, should not be found in the wild.
    None,
}

/// Describes a Jingle transport using a direct or proxied connection.
#[derive(Debug, Clone, PartialEq)]
pub struct Transport {
    /// The stream identifier for this transport.
    pub sid: StreamId,

    /// The destination address.
    pub dstaddr: Option<String>,

    /// The mode to be used for the transfer.
    pub mode: Mode,

    /// The payload of this transport.
    pub payload: TransportPayload,
}

impl Transport {
    /// Creates a new transport element.
    pub fn new(sid: StreamId) -> Transport {
        Transport {
            sid,
            dstaddr: None,
            mode: Default::default(),
            payload: TransportPayload::None,
        }
    }

    /// Sets the destination address of this transport.
    pub fn with_dstaddr(mut self, dstaddr: String) -> Transport {
        self.dstaddr = Some(dstaddr);
        self
    }

    /// Sets the mode of this transport.
    pub fn with_mode(mut self, mode: Mode) -> Transport {
        self.mode = mode;
        self
    }

    /// Sets the payload of this transport.
    pub fn with_payload(mut self, payload: TransportPayload) -> Transport {
        self.payload = payload;
        self
    }
}

impl TryFrom<Element> for Transport {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Transport, Error> {
        check_self!(elem, "transport", JINGLE_S5B);
        check_no_unknown_attributes!(elem, "transport", ["sid", "dstaddr", "mode"]);
        let sid = get_attr!(elem, "sid", Required);
        let dstaddr = get_attr!(elem, "dstaddr", Option);
        let mode = get_attr!(elem, "mode", Default);

        let mut payload = None;
        for child in elem.children() {
            payload = Some(if child.is("candidate", ns::JINGLE_S5B) {
                let mut candidates =
                    match payload {
                        Some(TransportPayload::Candidates(candidates)) => candidates,
                        Some(_) => return Err(Error::ParseError(
                            "Non-candidate child already present in JingleS5B transport element.",
                        )),
                        None => vec![],
                    };
                candidates.push(Candidate::try_from(child.clone())?);
                TransportPayload::Candidates(candidates)
            } else if child.is("activated", ns::JINGLE_S5B) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Non-activated child already present in JingleS5B transport element.",
                    ));
                }
                let cid = get_attr!(child, "cid", Required);
                TransportPayload::Activated(cid)
            } else if child.is("candidate-error", ns::JINGLE_S5B) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Non-candidate-error child already present in JingleS5B transport element.",
                    ));
                }
                TransportPayload::CandidateError
            } else if child.is("candidate-used", ns::JINGLE_S5B) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Non-candidate-used child already present in JingleS5B transport element.",
                    ));
                }
                let cid = get_attr!(child, "cid", Required);
                TransportPayload::CandidateUsed(cid)
            } else if child.is("proxy-error", ns::JINGLE_S5B) {
                if payload.is_some() {
                    return Err(Error::ParseError(
                        "Non-proxy-error child already present in JingleS5B transport element.",
                    ));
                }
                TransportPayload::ProxyError
            } else {
                return Err(Error::ParseError(
                    "Unknown child in JingleS5B transport element.",
                ));
            });
        }
        let payload = payload.unwrap_or(TransportPayload::None);
        Ok(Transport {
            sid,
            dstaddr,
            mode,
            payload,
        })
    }
}

impl From<Transport> for Element {
    fn from(transport: Transport) -> Element {
        Element::builder("transport", ns::JINGLE_S5B)
            .attr("sid", transport.sid)
            .attr("dstaddr", transport.dstaddr)
            .attr("mode", transport.mode)
            .append_all(match transport.payload {
                TransportPayload::Candidates(candidates) => candidates
                    .into_iter()
                    .map(Element::from)
                    .collect::<Vec<_>>(),
                TransportPayload::Activated(cid) => {
                    vec![Element::builder("activated", ns::JINGLE_S5B)
                        .attr("cid", cid)
                        .build()]
                }
                TransportPayload::CandidateError => {
                    vec![Element::builder("candidate-error", ns::JINGLE_S5B).build()]
                }
                TransportPayload::CandidateUsed(cid) => {
                    vec![Element::builder("candidate-used", ns::JINGLE_S5B)
                        .attr("cid", cid)
                        .build()]
                }
                TransportPayload::ProxyError => {
                    vec![Element::builder("proxy-error", ns::JINGLE_S5B).build()]
                }
                TransportPayload::None => vec![],
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jid::BareJid;
    use std::str::FromStr;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Type, 1);
        assert_size!(Mode, 1);
        assert_size!(CandidateId, 12);
        assert_size!(StreamId, 12);
        assert_size!(Candidate, 84);
        assert_size!(TransportPayload, 16);
        assert_size!(Transport, 44);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Type, 1);
        assert_size!(Mode, 1);
        assert_size!(CandidateId, 24);
        assert_size!(StreamId, 24);
        assert_size!(Candidate, 136);
        assert_size!(TransportPayload, 32);
        assert_size!(Transport, 88);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:s5b:1' sid='coucou'/>"
            .parse()
            .unwrap();
        let transport = Transport::try_from(elem).unwrap();
        assert_eq!(transport.sid, StreamId(String::from("coucou")));
        assert_eq!(transport.dstaddr, None);
        assert_eq!(transport.mode, Mode::Tcp);
        match transport.payload {
            TransportPayload::None => (),
            _ => panic!("Wrong element inside transport!"),
        }
    }

    #[test]
    fn test_serialise_activated() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:s5b:1' sid='coucou'><activated cid='coucou'/></transport>".parse().unwrap();
        let transport = Transport {
            sid: StreamId(String::from("coucou")),
            dstaddr: None,
            mode: Mode::Tcp,
            payload: TransportPayload::Activated(CandidateId(String::from("coucou"))),
        };
        let elem2: Element = transport.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_serialise_candidate() {
        let elem: Element = "<transport xmlns='urn:xmpp:jingle:transports:s5b:1' sid='coucou'><candidate cid='coucou' host='127.0.0.1' jid='coucou@coucou' priority='0'/></transport>".parse().unwrap();
        let transport = Transport {
            sid: StreamId(String::from("coucou")),
            dstaddr: None,
            mode: Mode::Tcp,
            payload: TransportPayload::Candidates(vec![Candidate {
                cid: CandidateId(String::from("coucou")),
                host: IpAddr::from_str("127.0.0.1").unwrap(),
                jid: Jid::Bare(BareJid::new("coucou", "coucou")),
                port: None,
                priority: 0u32,
                type_: Type::Direct,
            }]),
        };
        let elem2: Element = transport.into();
        assert_eq!(elem, elem2);
    }
}
