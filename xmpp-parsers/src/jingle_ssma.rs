// Copyright (c) 2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

generate_element!(
    /// Source element for the ssrc SDP attribute.
    Source, "source", JINGLE_SSMA,
    attributes: [
        /// Maps to the ssrc-id parameter.
        id: Required<String> = "ssrc",

        /// XXX: wtf is that?  It can be either name='jvb-a0' or name='jvb-v0' at avstack’s jicofo.
        name: Option<String> = "name",
    ],
    children: [
        /// List of attributes for this source.
        parameters: Vec<Parameter> = ("parameter", JINGLE_SSMA) => Parameter,

        /// ssrc-info for this source.
        info: Option<SsrcInfo> = ("ssrc-info", JITSI_MEET) => SsrcInfo
    ]
);

impl Source {
    /// Create a new SSMA Source element.
    pub fn new(id: String) -> Source {
        Source {
            id,
            parameters: Vec::new(),
            info: None,
            name: None,
        }
    }
}

generate_element!(
    /// Parameter associated with a ssrc.
    Parameter, "parameter", JINGLE_SSMA,
    attributes: [
        /// The name of the parameter.
        name: Required<String> = "name",

        /// The optional value of the parameter.
        value: Option<String> = "value",
    ]
);

generate_element!(
    /// ssrc-info associated with a ssrc.
    SsrcInfo, "ssrc-info", JITSI_MEET,
    attributes: [
        /// The owner of the ssrc.
        owner: Required<String> = "owner"
    ]
);

generate_element!(
    /// Element grouping multiple ssrc.
    Group, "ssrc-group", JINGLE_SSMA,
    attributes: [
        /// The semantics of this group.
        semantics: Required<String> = "semantics",
    ],
    children: [
        /// The various ssrc concerned by this group.
        sources: Vec<Source> = ("source", JINGLE_SSMA) => Source
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
        assert_size!(Source, 24);
        assert_size!(Parameter, 24);
        assert_size!(Group, 24);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Source, 48);
        assert_size!(Parameter, 48);
        assert_size!(Group, 48);
    }

    #[test]
    fn parse_source() {
        let elem: Element = "
<source ssrc='1656081975' xmlns='urn:xmpp:jingle:apps:rtp:ssma:0'>
    <parameter name='cname' value='Yv/wvbCdsDW2Prgd'/>
    <parameter name='msid' value='MLTJKIHilGn71fNQoszkQ4jlPTuS5vJyKVIv MLTJKIHilGn71fNQoszkQ4jlPTuS5vJyKVIva0'/>
</source>"
                .parse()
                .unwrap();
        let mut ssrc = Source::try_from(elem).unwrap();
        assert_eq!(ssrc.id, "1656081975");
        assert_eq!(ssrc.parameters.len(), 2);
        let parameter = ssrc.parameters.pop().unwrap();
        assert_eq!(parameter.name, "msid");
        assert_eq!(
            parameter.value.unwrap(),
            "MLTJKIHilGn71fNQoszkQ4jlPTuS5vJyKVIv MLTJKIHilGn71fNQoszkQ4jlPTuS5vJyKVIva0"
        );
        let parameter = ssrc.parameters.pop().unwrap();
        assert_eq!(parameter.name, "cname");
        assert_eq!(parameter.value.unwrap(), "Yv/wvbCdsDW2Prgd");
    }

    #[test]
    fn parse_source_group() {
        let elem: Element = "
<ssrc-group semantics='FID' xmlns='urn:xmpp:jingle:apps:rtp:ssma:0'>
    <source ssrc='2301230316'/>
    <source ssrc='386328120'/>
</ssrc-group>"
            .parse()
            .unwrap();
        let mut group = Group::try_from(elem).unwrap();
        assert_eq!(group.semantics, "FID");
        assert_eq!(group.sources.len(), 2);
        let source = group.sources.pop().unwrap();
        assert_eq!(source.id, "386328120");
        let source = group.sources.pop().unwrap();
        assert_eq!(source.id, "2301230316");
    }
}
