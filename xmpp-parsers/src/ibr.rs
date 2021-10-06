// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::data_forms::DataForm;
use crate::iq::{IqGetPayload, IqResultPayload, IqSetPayload};
use crate::ns;
use crate::util::error::Error;
use crate::Element;
use std::collections::HashMap;
use std::convert::TryFrom;

/// Query for registering against a service.
#[derive(Debug, Clone)]
pub struct Query {
    /// Deprecated fixed list of possible fields to fill before the user can
    /// register.
    pub fields: HashMap<String, String>,

    /// Whether this account is already registered.
    pub registered: bool,

    /// Whether to remove this account.
    pub remove: bool,

    /// A data form the user must fill before being allowed to register.
    pub form: Option<DataForm>,
    // Not yet implemented.
    //pub oob: Option<Oob>,
}

impl IqGetPayload for Query {}
impl IqSetPayload for Query {}
impl IqResultPayload for Query {}

impl TryFrom<Element> for Query {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Query, Error> {
        check_self!(elem, "query", REGISTER, "IBR query");
        let mut query = Query {
            registered: false,
            fields: HashMap::new(),
            remove: false,
            form: None,
        };
        for child in elem.children() {
            let namespace = child.ns();
            if namespace == ns::REGISTER {
                let name = child.name();
                let fields = vec![
                    "address",
                    "city",
                    "date",
                    "email",
                    "first",
                    "instructions",
                    "key",
                    "last",
                    "misc",
                    "name",
                    "nick",
                    "password",
                    "phone",
                    "state",
                    "text",
                    "url",
                    "username",
                    "zip",
                ];
                if fields.binary_search(&name).is_ok() {
                    query.fields.insert(name.to_owned(), child.text());
                } else if name == "registered" {
                    query.registered = true;
                } else if name == "remove" {
                    query.remove = true;
                } else {
                    return Err(Error::ParseError("Wrong field in ibr element."));
                }
            } else if child.is("x", ns::DATA_FORMS) {
                query.form = Some(DataForm::try_from(child.clone())?);
            } else {
                return Err(Error::ParseError("Unknown child in ibr element."));
            }
        }
        Ok(query)
    }
}

impl From<Query> for Element {
    fn from(query: Query) -> Element {
        Element::builder("query", ns::REGISTER)
            .append_all(if query.registered {
                Some(Element::builder("registered", ns::REGISTER))
            } else {
                None
            })
            .append_all(
                query
                    .fields
                    .into_iter()
                    .map(|(name, value)| Element::builder(name, ns::REGISTER).append(value)),
            )
            .append_all(if query.remove {
                Some(Element::builder("remove", ns::REGISTER))
            } else {
                None
            })
            .append_all(query.form.map(Element::from))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Query, 88);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Query, 160);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<query xmlns='jabber:iq:register'/>".parse().unwrap();
        Query::try_from(elem).unwrap();
    }

    #[test]
    fn test_ex2() {
        let elem: Element = r#"
<query xmlns='jabber:iq:register'>
  <instructions>
    Choose a username and password for use with this service.
    Please also provide your email address.
  </instructions>
  <username/>
  <password/>
  <email/>
</query>
"#
        .parse()
        .unwrap();
        let query = Query::try_from(elem).unwrap();
        assert_eq!(query.registered, false);
        assert_eq!(query.fields["instructions"], "\n    Choose a username and password for use with this service.\n    Please also provide your email address.\n  ");
        assert_eq!(query.fields["username"], "");
        assert_eq!(query.fields["password"], "");
        assert_eq!(query.fields["email"], "");
        assert_eq!(query.fields.contains_key("name"), false);

        // FIXME: HashMap doesn’t keep the order right.
        //let elem2 = query.into();
        //assert_eq!(elem, elem2);
    }

    #[test]
    fn test_ex9() {
        let elem: Element = "<query xmlns='jabber:iq:register'><instructions>Use the enclosed form to register. If your Jabber client does not support Data Forms, visit http://www.shakespeare.lit/contests.php</instructions><x xmlns='jabber:x:data' type='form'><title>Contest Registration</title><instructions>Please provide the following information to sign up for our special contests!</instructions><field type='hidden' var='FORM_TYPE'><value>jabber:iq:register</value></field><field label='Given Name' var='first'><required/></field><field label='Family Name' var='last'><required/></field><field label='Email Address' var='email'><required/></field><field type='list-single' label='Gender' var='x-gender'><option label='Male'><value>M</value></option><option label='Female'><value>F</value></option></field></x></query>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();
        let query = Query::try_from(elem).unwrap();
        assert_eq!(query.registered, false);
        assert!(!query.fields["instructions"].is_empty());
        let form = query.form.clone().unwrap();
        assert!(!form.instructions.unwrap().is_empty());
        let elem2 = query.into();
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_ex10() {
        let elem: Element = "<query xmlns='jabber:iq:register'><x xmlns='jabber:x:data' type='submit'><field type='hidden' var='FORM_TYPE'><value>jabber:iq:register</value></field><field label='Given Name' var='first'><value>Juliet</value></field><field label='Family Name' var='last'><value>Capulet</value></field><field label='Email Address' var='email'><value>juliet@capulet.com</value></field><field type='list-single' label='Gender' var='x-gender'><value>F</value></field></x></query>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();
        let query = Query::try_from(elem).unwrap();
        assert_eq!(query.registered, false);
        for _ in &query.fields {
            panic!();
        }
        let elem2 = query.into();
        assert_eq!(elem1, elem2);
    }
}
