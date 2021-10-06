// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::jingle::ContentId;

generate_attribute!(
    /// The semantics of the grouping.
    Semantics, "semantics", {
        /// Lip synchronsation.
        Ls => "LS",

        /// Bundle.
        Bundle => "BUNDLE",
    }
);

generate_element!(
    /// Describes a content that should be grouped with other ones.
    Content, "content", JINGLE_GROUPING,
    attributes: [
        /// The name of the matching [`Content`](crate::jingle::Content).
        name: Required<ContentId> = "name",
    ]
);

impl Content {
    /// Creates a new <content/> element.
    pub fn new(name: &str) -> Content {
        Content {
            name: ContentId(name.to_string()),
        }
    }
}

generate_element!(
    /// A semantic group of contents.
    Group, "group", JINGLE_GROUPING,
    attributes: [
        /// Semantics of the grouping.
        semantics: Required<Semantics> = "semantics",
    ],
    children: [
        /// List of contents that should be grouped with each other.
        contents: Vec<Content> = ("content", JINGLE_GROUPING) => Content
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
        assert_size!(Semantics, 1);
        assert_size!(Content, 12);
        assert_size!(Group, 16);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Semantics, 1);
        assert_size!(Content, 24);
        assert_size!(Group, 32);
    }

    #[test]
    fn parse_group() {
        let elem: Element = "
        <group xmlns='urn:xmpp:jingle:apps:grouping:0' semantics='BUNDLE'>
            <content name='voice'/>
            <content name='webcam'/>
        </group>"
            .parse()
            .unwrap();
        let group = Group::try_from(elem).unwrap();
        assert_eq!(group.semantics, Semantics::Bundle);
        assert_eq!(group.contents.len(), 2);
        assert_eq!(
            group.contents,
            &[Content::new("voice"), Content::new("webcam")]
        );
    }
}
