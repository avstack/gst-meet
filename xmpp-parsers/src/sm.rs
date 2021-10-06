// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::stanza_error::DefinedCondition;

generate_element!(
    /// Acknowledgement of the currently received stanzas.
    A, "a", SM,
    attributes: [
        /// The last handled stanza.
        h: Required<u32> = "h",
    ]
);

impl A {
    /// Generates a new `<a/>` element.
    pub fn new(h: u32) -> A {
        A { h }
    }
}

generate_attribute!(
    /// Whether to allow resumption of a previous stream.
    ResumeAttr,
    "resume",
    bool
);

generate_element!(
    /// Client request for enabling stream management.
    #[derive(Default)]
    Enable, "enable", SM,
    attributes: [
        /// The preferred resumption time in seconds by the client.
        // TODO: should be the infinite integer set ≥ 1.
        max: Option<u32> = "max",

        /// Whether the client wants to be allowed to resume the stream.
        resume: Default<ResumeAttr> = "resume",
    ]
);

impl Enable {
    /// Generates a new `<enable/>` element.
    pub fn new() -> Self {
        Enable::default()
    }

    /// Sets the preferred resumption time in seconds.
    pub fn with_max(mut self, max: u32) -> Self {
        self.max = Some(max);
        self
    }

    /// Asks for resumption to be possible.
    pub fn with_resume(mut self) -> Self {
        self.resume = ResumeAttr::True;
        self
    }
}

generate_id!(
    /// A random identifier used for stream resumption.
    StreamId
);

generate_element!(
    /// Server response once stream management is enabled.
    Enabled, "enabled", SM,
    attributes: [
        /// A random identifier used for stream resumption.
        id: Option<StreamId> = "id",

        /// The preferred IP, domain, IP:port or domain:port location for
        /// resumption.
        location: Option<String> = "location",

        /// The preferred resumption time in seconds by the server.
        // TODO: should be the infinite integer set ≥ 1.
        max: Option<u32> = "max",

        /// Whether stream resumption is allowed.
        resume: Default<ResumeAttr> = "resume",
    ]
);

generate_element!(
    /// A stream management error happened.
    Failed, "failed", SM,
    attributes: [
        /// The last handled stanza.
        h: Option<u32> = "h",
    ],
    children: [
        /// The error returned.
        // XXX: implement the * handling.
        error: Option<DefinedCondition> = ("*", XMPP_STANZAS) => DefinedCondition
    ]
);

generate_empty_element!(
    /// Requests the currently received stanzas by the other party.
    R,
    "r",
    SM
);

generate_element!(
    /// Requests a stream resumption.
    Resume, "resume", SM,
    attributes: [
        /// The last handled stanza.
        h: Required<u32> = "h",

        /// The previous id given by the server on
        /// [enabled](struct.Enabled.html).
        previd: Required<StreamId> = "previd",
    ]
);

generate_element!(
    /// The response by the server for a successfully resumed stream.
    Resumed, "resumed", SM,
    attributes: [
        /// The last handled stanza.
        h: Required<u32> = "h",

        /// The previous id given by the server on
        /// [enabled](struct.Enabled.html).
        previd: Required<StreamId> = "previd",
    ]
);

// TODO: add support for optional and required.
generate_empty_element!(
    /// Represents availability of Stream Management in `<stream:features/>`.
    StreamManagement,
    "sm",
    SM
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(A, 4);
        assert_size!(ResumeAttr, 1);
        assert_size!(Enable, 12);
        assert_size!(StreamId, 12);
        assert_size!(Enabled, 36);
        assert_size!(Failed, 12);
        assert_size!(R, 0);
        assert_size!(Resume, 16);
        assert_size!(Resumed, 16);
        assert_size!(StreamManagement, 0);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(A, 4);
        assert_size!(ResumeAttr, 1);
        assert_size!(Enable, 12);
        assert_size!(StreamId, 24);
        assert_size!(Enabled, 64);
        assert_size!(Failed, 12);
        assert_size!(R, 0);
        assert_size!(Resume, 32);
        assert_size!(Resumed, 32);
        assert_size!(StreamManagement, 0);
    }

    #[test]
    fn a() {
        let elem: Element = "<a xmlns='urn:xmpp:sm:3' h='5'".parse().unwrap();
        let a = A::try_from(elem).unwrap();
        assert_eq!(a.h, 5);
    }

    #[test]
    fn stream_feature() {
        let elem: Element = "<sm xmlns='urn:xmpp:sm:3'/>".parse().unwrap();
        StreamManagement::try_from(elem).unwrap();
    }

    #[test]
    fn resume() {
        let elem: Element = "<enable xmlns='urn:xmpp:sm:3' resume='true'/>"
            .parse()
            .unwrap();
        let enable = Enable::try_from(elem).unwrap();
        assert_eq!(enable.max, None);
        assert_eq!(enable.resume, ResumeAttr::True);

        let elem: Element = "<enabled xmlns='urn:xmpp:sm:3' resume='true' id='coucou' max='600'/>"
            .parse()
            .unwrap();
        let enabled = Enabled::try_from(elem).unwrap();
        let previd = enabled.id.unwrap();
        assert_eq!(enabled.resume, ResumeAttr::True);
        assert_eq!(previd, StreamId(String::from("coucou")));
        assert_eq!(enabled.max, Some(600));
        assert_eq!(enabled.location, None);

        let elem: Element = "<resume xmlns='urn:xmpp:sm:3' h='5' previd='coucou'/>"
            .parse()
            .unwrap();
        let resume = Resume::try_from(elem).unwrap();
        assert_eq!(resume.h, 5);
        assert_eq!(resume.previd, previd);

        let elem: Element = "<resumed xmlns='urn:xmpp:sm:3' h='5' previd='coucou'/>"
            .parse()
            .unwrap();
        let resumed = Resumed::try_from(elem).unwrap();
        assert_eq!(resumed.h, 5);
        assert_eq!(resumed.previd, previd);
    }

    #[test]
    fn test_serialize_failed() {
        let reference: Element = "<failed xmlns='urn:xmpp:sm:3'><unexpected-request xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/></failed>"
        .parse()
        .unwrap();

        let elem: Element = "<unexpected-request xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/>"
            .parse()
            .unwrap();

        let error = DefinedCondition::try_from(elem).unwrap();

        let failed = Failed {
            h: None,
            error: Some(error),
        };
        let serialized: Element = failed.into();
        assert_eq!(serialized, reference);
    }
}
