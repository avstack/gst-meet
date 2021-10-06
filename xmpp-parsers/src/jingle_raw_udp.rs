// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::jingle_ice_udp::Type;
use std::net::IpAddr;

generate_element!(
    /// Wrapper element for an raw UDP transport.
    Transport, "transport", JINGLE_RAW_UDP,
    children: [
        /// List of candidates for this raw UDP session.
        candidates: Vec<Candidate> = ("candidate", JINGLE_RAW_UDP) => Candidate
    ]
);

impl Transport {
    /// Create a new ICE-UDP transport.
    pub fn new() -> Transport {
        Transport {
            candidates: Vec::new(),
        }
    }

    /// Add a candidate to this transport.
    pub fn add_candidate(mut self, candidate: Candidate) -> Self {
        self.candidates.push(candidate);
        self
    }
}

generate_element!(
    /// A candidate for an ICE-UDP session.
    Candidate, "candidate", JINGLE_RAW_UDP,
    attributes: [
        /// A Component ID as defined in ICE-CORE.
        component: Required<u8> = "component",

        /// An index, starting at 0, that enables the parties to keep track of updates to the
        /// candidate throughout the life of the session.
        generation: Required<u8> = "generation",

        /// A unique identifier for the candidate.
        id: Required<String> = "id",

        /// The Internet Protocol (IP) address for the candidate transport mechanism; this can be
        /// either an IPv4 address or an IPv6 address.
        ip: Required<IpAddr> = "ip",

        /// The port at the candidate IP address.
        port: Required<u16> = "port",

        /// A Candidate Type as defined in ICE-CORE.
        type_: Option<Type> = "type",
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Transport, 12);
        assert_size!(Candidate, 40);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Transport, 24);
        assert_size!(Candidate, 56);
    }

    #[test]
    fn example_1() {
        let elem: Element = "
<transport xmlns='urn:xmpp:jingle:transports:raw-udp:1'>
    <candidate component='1'
               generation='0'
               id='a9j3mnbtu1'
               ip='10.1.1.104'
               port='13540'/>
</transport>"
            .parse()
            .unwrap();
        let mut transport = Transport::try_from(elem).unwrap();
        assert_eq!(transport.candidates.len(), 1);
        let candidate = transport.candidates.pop().unwrap();
        assert_eq!(candidate.component, 1);
        assert_eq!(candidate.generation, 0);
        assert_eq!(candidate.id, "a9j3mnbtu1");
        assert_eq!(candidate.ip, "10.1.1.104".parse::<IpAddr>().unwrap());
        assert_eq!(candidate.port, 13540u16);
        assert!(candidate.type_.is_none());
    }
}
