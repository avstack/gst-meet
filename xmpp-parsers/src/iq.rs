// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017 Maxime “pep” Buquet <pep@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ns;
use crate::stanza_error::StanzaError;
use crate::util::error::Error;
use crate::Element;
use jid::Jid;
use minidom::IntoAttributeValue;
use std::convert::TryFrom;

/// Should be implemented on every known payload of an `<iq type='get'/>`.
pub trait IqGetPayload: TryFrom<Element> + Into<Element> {}

/// Should be implemented on every known payload of an `<iq type='set'/>`.
pub trait IqSetPayload: TryFrom<Element> + Into<Element> {}

/// Should be implemented on every known payload of an `<iq type='result'/>`.
pub trait IqResultPayload: TryFrom<Element> + Into<Element> {}

/// Represents one of the four possible iq types.
#[derive(Debug, Clone)]
pub enum IqType {
    /// This is a request for accessing some data.
    Get(Element),

    /// This is a request for modifying some data.
    Set(Element),

    /// This is a result containing some data.
    Result(Option<Element>),

    /// A get or set request failed.
    Error(StanzaError),
}

impl<'a> IntoAttributeValue for &'a IqType {
    fn into_attribute_value(self) -> Option<String> {
        Some(
            match *self {
                IqType::Get(_) => "get",
                IqType::Set(_) => "set",
                IqType::Result(_) => "result",
                IqType::Error(_) => "error",
            }
            .to_owned(),
        )
    }
}

/// The main structure representing the `<iq/>` stanza.
#[derive(Debug, Clone)]
pub struct Iq {
    /// The JID emitting this stanza.
    pub from: Option<Jid>,

    /// The recipient of this stanza.
    pub to: Option<Jid>,

    /// The @id attribute of this stanza, which is required in order to match a
    /// request with its result/error.
    pub id: String,

    /// The payload content of this stanza.
    pub payload: IqType,
}

impl Iq {
    /// Creates an `<iq/>` stanza containing a get request.
    pub fn from_get<S: Into<String>>(id: S, payload: impl IqGetPayload) -> Iq {
        Iq {
            from: None,
            to: None,
            id: id.into(),
            payload: IqType::Get(payload.into()),
        }
    }

    /// Creates an `<iq/>` stanza containing a set request.
    pub fn from_set<S: Into<String>>(id: S, payload: impl IqSetPayload) -> Iq {
        Iq {
            from: None,
            to: None,
            id: id.into(),
            payload: IqType::Set(payload.into()),
        }
    }

    /// Creates an empty `<iq type="result"/>` stanza.
    pub fn empty_result<S: Into<String>>(to: Jid, id: S) -> Iq {
        Iq {
            from: None,
            to: Some(to),
            id: id.into(),
            payload: IqType::Result(None),
        }
    }

    /// Creates an `<iq/>` stanza containing a result.
    pub fn from_result<S: Into<String>>(id: S, payload: Option<impl IqResultPayload>) -> Iq {
        Iq {
            from: None,
            to: None,
            id: id.into(),
            payload: IqType::Result(payload.map(Into::into)),
        }
    }

    /// Creates an `<iq/>` stanza containing an error.
    pub fn from_error<S: Into<String>>(id: S, payload: StanzaError) -> Iq {
        Iq {
            from: None,
            to: None,
            id: id.into(),
            payload: IqType::Error(payload),
        }
    }

    /// Sets the recipient of this stanza.
    pub fn with_to(mut self, to: Jid) -> Iq {
        self.to = Some(to);
        self
    }

    /// Sets the emitter of this stanza.
    pub fn with_from(mut self, from: Jid) -> Iq {
        self.from = Some(from);
        self
    }

    /// Sets the id of this stanza, in order to later match its response.
    pub fn with_id(mut self, id: String) -> Iq {
        self.id = id;
        self
    }
}

impl TryFrom<Element> for Iq {
    type Error = Error;

    fn try_from(root: Element) -> Result<Iq, Error> {
        check_self!(root, "iq", DEFAULT_NS);
        let from = get_attr!(root, "from", Option);
        let to = get_attr!(root, "to", Option);
        let id = get_attr!(root, "id", Required);
        let type_: String = get_attr!(root, "type", Required);

        let mut payload = None;
        let mut error_payload = None;
        for elem in root.children() {
            if payload.is_some() {
                return Err(Error::ParseError("Wrong number of children in iq element."));
            }
            if type_ == "error" {
                if elem.is("error", ns::DEFAULT_NS) {
                    if error_payload.is_some() {
                        return Err(Error::ParseError("Wrong number of children in iq element."));
                    }
                    error_payload = Some(StanzaError::try_from(elem.clone())?);
                } else if root.children().count() != 2 {
                    return Err(Error::ParseError("Wrong number of children in iq element."));
                }
            } else {
                payload = Some(elem.clone());
            }
        }

        let type_ = if type_ == "get" {
            if let Some(payload) = payload {
                IqType::Get(payload)
            } else {
                return Err(Error::ParseError("Wrong number of children in iq element."));
            }
        } else if type_ == "set" {
            if let Some(payload) = payload {
                IqType::Set(payload)
            } else {
                return Err(Error::ParseError("Wrong number of children in iq element."));
            }
        } else if type_ == "result" {
            if let Some(payload) = payload {
                IqType::Result(Some(payload))
            } else {
                IqType::Result(None)
            }
        } else if type_ == "error" {
            if let Some(payload) = error_payload {
                IqType::Error(payload)
            } else {
                return Err(Error::ParseError("Wrong number of children in iq element."));
            }
        } else {
            return Err(Error::ParseError("Unknown iq type."));
        };

        Ok(Iq {
            from,
            to,
            id,
            payload: type_,
        })
    }
}

impl From<Iq> for Element {
    fn from(iq: Iq) -> Element {
        let mut stanza = Element::builder("iq", ns::DEFAULT_NS)
            .attr("from", iq.from)
            .attr("to", iq.to)
            .attr("id", iq.id)
            .attr("type", &iq.payload)
            .build();
        let elem = match iq.payload {
            IqType::Get(elem) | IqType::Set(elem) | IqType::Result(Some(elem)) => elem,
            IqType::Error(error) => error.into(),
            IqType::Result(None) => return stanza,
        };
        stanza.append_child(elem);
        stanza
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::disco::DiscoInfoQuery;
    use crate::stanza_error::{DefinedCondition, ErrorType};

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(IqType, 136);
        assert_size!(Iq, 228);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(IqType, 272);
        assert_size!(Iq, 456);
    }

    #[test]
    fn test_require_type() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept'/>".parse().unwrap();
        let error = Iq::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'id' missing.");

        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' id='coucou'/>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' id='coucou'/>"
            .parse()
            .unwrap();
        let error = Iq::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");
    }

    #[test]
    fn test_get() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='get' id='foo'>
            <foo xmlns='bar'/>
        </iq>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='get' id='foo'>
            <foo xmlns='bar'/>
        </iq>"
            .parse()
            .unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let query: Element = "<foo xmlns='bar'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(&iq.id, "foo");
        assert!(match iq.payload {
            IqType::Get(element) => element == query,
            _ => false,
        });
    }

    #[test]
    fn test_set() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='set' id='vcard'>
            <vCard xmlns='vcard-temp'/>
        </iq>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='set' id='vcard'>
            <vCard xmlns='vcard-temp'/>
        </iq>"
            .parse()
            .unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let vcard: Element = "<vCard xmlns='vcard-temp'/>".parse().unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(&iq.id, "vcard");
        assert!(match iq.payload {
            IqType::Set(element) => element == vcard,
            _ => false,
        });
    }

    #[test]
    fn test_result_empty() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='result' id='res'/>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='result' id='res'/>"
            .parse()
            .unwrap();
        let iq = Iq::try_from(elem).unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(&iq.id, "res");
        assert!(match iq.payload {
            IqType::Result(None) => true,
            _ => false,
        });
    }

    #[test]
    fn test_result() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='result' id='res'>
            <query xmlns='http://jabber.org/protocol/disco#items'/>
        </iq>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='result' id='res'>
            <query xmlns='http://jabber.org/protocol/disco#items'/>
        </iq>"
            .parse()
            .unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let query: Element = "<query xmlns='http://jabber.org/protocol/disco#items'/>"
            .parse()
            .unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(&iq.id, "res");
        assert!(match iq.payload {
            IqType::Result(Some(element)) => element == query,
            _ => false,
        });
    }

    #[test]
    fn test_error() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='error' id='err1'>
            <ping xmlns='urn:xmpp:ping'/>
            <error type='cancel'>
                <service-unavailable xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/>
            </error>
        </iq>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='error' id='err1'>
            <ping xmlns='urn:xmpp:ping'/>
            <error type='cancel'>
                <service-unavailable xmlns='urn:ietf:params:xml:ns:xmpp-stanzas'/>
            </error>
        </iq>"
            .parse()
            .unwrap();
        let iq = Iq::try_from(elem).unwrap();
        assert_eq!(iq.from, None);
        assert_eq!(iq.to, None);
        assert_eq!(iq.id, "err1");
        match iq.payload {
            IqType::Error(error) => {
                assert_eq!(error.type_, ErrorType::Cancel);
                assert_eq!(error.by, None);
                assert_eq!(
                    error.defined_condition,
                    DefinedCondition::ServiceUnavailable
                );
                assert_eq!(error.texts.len(), 0);
                assert_eq!(error.other, None);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn test_children_invalid() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='error' id='error'/>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='error' id='error'/>"
            .parse()
            .unwrap();
        let error = Iq::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Wrong number of children in iq element.");
    }

    #[test]
    fn test_serialise() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='result' id='res'/>"
            .parse()
            .unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='result' id='res'/>"
            .parse()
            .unwrap();
        let iq2 = Iq {
            from: None,
            to: None,
            id: String::from("res"),
            payload: IqType::Result(None),
        };
        let elem2 = iq2.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_disco() {
        #[cfg(not(feature = "component"))]
        let elem: Element = "<iq xmlns='jabber:client' type='get' id='disco'><query xmlns='http://jabber.org/protocol/disco#info'/></iq>".parse().unwrap();
        #[cfg(feature = "component")]
        let elem: Element = "<iq xmlns='jabber:component:accept' type='get' id='disco'><query xmlns='http://jabber.org/protocol/disco#info'/></iq>".parse().unwrap();
        let iq = Iq::try_from(elem).unwrap();
        let disco_info = match iq.payload {
            IqType::Get(payload) => DiscoInfoQuery::try_from(payload).unwrap(),
            _ => panic!(),
        };
        assert!(disco_info.node.is_none());
    }
}
