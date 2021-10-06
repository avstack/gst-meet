// Copyright (c) 2018 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use jid::BareJid;

generate_element!(
    /// The stream opening for client-server communications.
    Stream, "stream", STREAM,
    attributes: [
        /// The JID of the entity opening this stream.
        from: Option<BareJid> = "from",

        /// The JID of the entity receiving this stream opening.
        to: Option<BareJid> = "to",

        /// The id of the stream, used for authentication challenges.
        id: Option<String> = "id",

        /// The XMPP version used during this stream.
        version: Option<String> = "version",

        /// The default human language for all subsequent stanzas, which will
        /// be transmitted to other entities for better localisation.
        xml_lang: Option<String> = "xml:lang",
    ]
);

impl Stream {
    /// Creates a simple client→server `<stream:stream>` element.
    pub fn new(to: BareJid) -> Stream {
        Stream {
            from: None,
            to: Some(to),
            id: None,
            version: Some(String::from("1.0")),
            xml_lang: None,
        }
    }

    /// Sets the [@from](#structfield.from) attribute on this `<stream:stream>`
    /// element.
    pub fn with_from(mut self, from: BareJid) -> Stream {
        self.from = Some(from);
        self
    }

    /// Sets the [@id](#structfield.id) attribute on this `<stream:stream>`
    /// element.
    pub fn with_id(mut self, id: String) -> Stream {
        self.id = Some(id);
        self
    }

    /// Sets the [@xml:lang](#structfield.xml_lang) attribute on this
    /// `<stream:stream>` element.
    pub fn with_lang(mut self, xml_lang: String) -> Stream {
        self.xml_lang = Some(xml_lang);
        self
    }

    /// Checks whether the version matches the expected one.
    pub fn is_version(&self, version: &str) -> bool {
        match self.version {
            None => false,
            Some(ref self_version) => self_version == &String::from(version),
        }
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
        assert_size!(Stream, 84);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Stream, 168);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<stream:stream xmlns='jabber:client' xmlns:stream='http://etherx.jabber.org/streams' xml:lang='en' version='1.0' id='abc' from='some-server.example'/>".parse().unwrap();
        let stream = Stream::try_from(elem).unwrap();
        assert_eq!(stream.from, Some(BareJid::domain("some-server.example")));
        assert_eq!(stream.to, None);
        assert_eq!(stream.id, Some(String::from("abc")));
        assert_eq!(stream.version, Some(String::from("1.0")));
        assert_eq!(stream.xml_lang, Some(String::from("en")));
    }
}
