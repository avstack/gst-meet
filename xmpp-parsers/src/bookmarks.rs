// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use jid::BareJid;

generate_attribute!(
    /// Whether a conference bookmark should be joined automatically.
    Autojoin,
    "autojoin",
    bool
);

generate_element!(
    /// A conference bookmark.
    Conference, "conference", BOOKMARKS,
    attributes: [
        /// Whether a conference bookmark should be joined automatically.
        autojoin: Default<Autojoin> = "autojoin",

        /// The JID of the conference.
        jid: Required<BareJid> = "jid",

        /// A user-defined name for this conference.
        name: Option<String> = "name",
    ],
    children: [
        /// The nick the user will use to join this conference.
        nick: Option<String> = ("nick", BOOKMARKS) => String,

        /// The password required to join this conference.
        password: Option<String> = ("password", BOOKMARKS) => String
    ]
);

generate_element!(
    /// An URL bookmark.
    Url, "url", BOOKMARKS,
    attributes: [
        /// A user-defined name for this URL.
        name: Option<String> = "name",

        /// The URL of this bookmark.
        url: Required<String> = "url",
    ]
);

generate_element!(
    /// Container element for multiple bookmarks.
    #[derive(Default)]
    Storage, "storage", BOOKMARKS,
    children: [
        /// Conferences the user has expressed an interest in.
        conferences: Vec<Conference> = ("conference", BOOKMARKS) => Conference,

        /// URLs the user is interested in.
        urls: Vec<Url> = ("url", BOOKMARKS) => Url
    ]
);

impl Storage {
    /// Create an empty bookmarks storage.
    pub fn new() -> Storage {
        Storage::default()
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
        assert_size!(Conference, 64);
        assert_size!(Url, 24);
        assert_size!(Storage, 24);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Conference, 128);
        assert_size!(Url, 48);
        assert_size!(Storage, 48);
    }

    #[test]
    fn empty() {
        let elem: Element = "<storage xmlns='storage:bookmarks'/>".parse().unwrap();
        let elem1 = elem.clone();
        let storage = Storage::try_from(elem).unwrap();
        assert_eq!(storage.conferences.len(), 0);
        assert_eq!(storage.urls.len(), 0);

        let elem2 = Element::from(Storage::new());
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn complete() {
        let elem: Element = "<storage xmlns='storage:bookmarks'><url name='Example' url='https://example.org/'/><conference autojoin='true' jid='test-muc@muc.localhost' name='Test MUC'><nick>Coucou</nick><password>secret</password></conference></storage>".parse().unwrap();
        let storage = Storage::try_from(elem).unwrap();
        assert_eq!(storage.urls.len(), 1);
        assert_eq!(storage.urls[0].clone().name.unwrap(), "Example");
        assert_eq!(storage.urls[0].url, "https://example.org/");
        assert_eq!(storage.conferences.len(), 1);
        assert_eq!(storage.conferences[0].autojoin, Autojoin::True);
        assert_eq!(
            storage.conferences[0].jid,
            BareJid::new("test-muc", "muc.localhost")
        );
        assert_eq!(storage.conferences[0].clone().name.unwrap(), "Test MUC");
        assert_eq!(storage.conferences[0].clone().nick.unwrap(), "Coucou");
        assert_eq!(storage.conferences[0].clone().password.unwrap(), "secret");
    }
}
