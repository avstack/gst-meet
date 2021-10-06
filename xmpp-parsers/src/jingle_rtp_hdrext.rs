// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

generate_attribute!(
    /// Which party is allowed to send the negotiated RTP Header Extensions.
    Senders, "senders", {
        /// Both parties can send them.
        Both => "both",

        /// Only the initiator can send them.
        Initiator => "initiator",

        /// Only the responder can send them.
        Responder => "responder",
    }, Default = Both
);

generate_element!(
    /// Header extensions to be used in a RTP description.
    RtpHdrext, "rtp-hdrext", JINGLE_RTP_HDREXT,
    attributes: [
        /// The ID of the extensions.
        id: Required<String> = "id",

        /// The URI that defines the extension.
        uri: Required<String> = "uri",

        /// Which party is allowed to send the negotiated RTP Header Extensions.
        senders: Default<Senders> = "senders",
    ]
);

impl RtpHdrext {
    /// Create a new RTP header extension element.
    pub fn new(id: String, uri: String) -> RtpHdrext {
        RtpHdrext {
            id,
            uri,
            senders: Default::default(),
        }
    }

    /// Set the senders.
    pub fn with_senders(mut self, senders: Senders) -> RtpHdrext {
        self.senders = senders;
        self
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
        assert_size!(Senders, 1);
        assert_size!(RtpHdrext, 28);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Senders, 1);
        assert_size!(RtpHdrext, 56);
    }

    #[test]
    fn parse_exthdr() {
        let elem: Element = "
        <rtp-hdrext xmlns='urn:xmpp:jingle:apps:rtp:rtp-hdrext:0'
                    uri='urn:ietf:params:rtp-hdrext:toffset'
                    id='1'/>"
            .parse()
            .unwrap();
        let rtp_hdrext = RtpHdrext::try_from(elem).unwrap();
        assert_eq!(rtp_hdrext.id, "1");
        assert_eq!(rtp_hdrext.uri, "urn:ietf:params:rtp-hdrext:toffset");
        assert_eq!(rtp_hdrext.senders, Senders::Both);
    }
}
