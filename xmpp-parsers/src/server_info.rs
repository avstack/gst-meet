// Copyright (C) 2019 Maxime “pep” Buquet <pep@bouah.net>
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::data_forms::{DataForm, DataFormType, Field, FieldType};
use crate::ns;
use crate::util::error::Error;
use std::convert::TryFrom;

/// Structure representing a `http://jabber.org/network/serverinfo` form type.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ServerInfo {
    /// Abuse addresses
    pub abuse: Vec<String>,

    /// Admin addresses
    pub admin: Vec<String>,

    /// Feedback addresses
    pub feedback: Vec<String>,

    /// Sales addresses
    pub sales: Vec<String>,

    /// Security addresses
    pub security: Vec<String>,

    /// Support addresses
    pub support: Vec<String>,
}

impl TryFrom<DataForm> for ServerInfo {
    type Error = Error;

    fn try_from(form: DataForm) -> Result<ServerInfo, Error> {
        if form.type_ != DataFormType::Result_ {
            return Err(Error::ParseError("Wrong type of form."));
        }
        if form.form_type != Some(String::from(ns::SERVER_INFO)) {
            return Err(Error::ParseError("Wrong FORM_TYPE for form."));
        }
        let mut server_info = ServerInfo::default();
        for field in form.fields {
            if field.type_ != FieldType::ListMulti {
                return Err(Error::ParseError("Field is not of the required type."));
            }
            if field.var == "abuse-addresses" {
                server_info.abuse = field.values;
            } else if field.var == "admin-addresses" {
                server_info.admin = field.values;
            } else if field.var == "feedback-addresses" {
                server_info.feedback = field.values;
            } else if field.var == "sales-addresses" {
                server_info.sales = field.values;
            } else if field.var == "security-addresses" {
                server_info.security = field.values;
            } else if field.var == "support-addresses" {
                server_info.support = field.values;
            } else {
                return Err(Error::ParseError("Unknown form field var."));
            }
        }

        Ok(server_info)
    }
}

impl From<ServerInfo> for DataForm {
    fn from(server_info: ServerInfo) -> DataForm {
        DataForm {
            type_: DataFormType::Result_,
            form_type: Some(String::from(ns::SERVER_INFO)),
            title: None,
            instructions: None,
            fields: vec![
                generate_address_field("abuse-addresses", server_info.abuse),
                generate_address_field("admin-addresses", server_info.admin),
                generate_address_field("feedback-addresses", server_info.feedback),
                generate_address_field("sales-addresses", server_info.sales),
                generate_address_field("security-addresses", server_info.security),
                generate_address_field("support-addresses", server_info.support),
            ],
        }
    }
}

/// Generate `Field` for addresses
pub fn generate_address_field<S: Into<String>>(var: S, values: Vec<String>) -> Field {
    Field {
        var: var.into(),
        type_: FieldType::ListMulti,
        label: None,
        required: false,
        options: vec![],
        values,
        media: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_forms::{DataForm, DataFormType, Field, FieldType};

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(ServerInfo, 72);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(ServerInfo, 144);
    }

    #[test]
    fn test_simple() {
        let form = DataForm {
            type_: DataFormType::Result_,
            form_type: Some(String::from(ns::SERVER_INFO)),
            title: None,
            instructions: None,
            fields: vec![
                Field {
                    var: String::from("abuse-addresses"),
                    type_: FieldType::ListMulti,
                    label: None,
                    required: false,
                    options: vec![],
                    values: vec![],
                    media: vec![],
                },
                Field {
                    var: String::from("admin-addresses"),
                    type_: FieldType::ListMulti,
                    label: None,
                    required: false,
                    options: vec![],
                    values: vec![
                        String::from("xmpp:admin@foo.bar"),
                        String::from("https://foo.bar/chat/"),
                        String::from("mailto:admin@foo.bar"),
                    ],
                    media: vec![],
                },
                Field {
                    var: String::from("feedback-addresses"),
                    type_: FieldType::ListMulti,
                    label: None,
                    required: false,
                    options: vec![],
                    values: vec![],
                    media: vec![],
                },
                Field {
                    var: String::from("sales-addresses"),
                    type_: FieldType::ListMulti,
                    label: None,
                    required: false,
                    options: vec![],
                    values: vec![],
                    media: vec![],
                },
                Field {
                    var: String::from("security-addresses"),
                    type_: FieldType::ListMulti,
                    label: None,
                    required: false,
                    options: vec![],
                    values: vec![
                        String::from("xmpp:security@foo.bar"),
                        String::from("mailto:security@foo.bar"),
                    ],
                    media: vec![],
                },
                Field {
                    var: String::from("support-addresses"),
                    type_: FieldType::ListMulti,
                    label: None,
                    required: false,
                    options: vec![],
                    values: vec![String::from("mailto:support@foo.bar")],
                    media: vec![],
                },
            ],
        };

        let server_info = ServerInfo {
            abuse: vec![],
            admin: vec![
                String::from("xmpp:admin@foo.bar"),
                String::from("https://foo.bar/chat/"),
                String::from("mailto:admin@foo.bar"),
            ],
            feedback: vec![],
            sales: vec![],
            security: vec![
                String::from("xmpp:security@foo.bar"),
                String::from("mailto:security@foo.bar"),
            ],
            support: vec![String::from("mailto:support@foo.bar")],
        };

        // assert_eq!(DataForm::from(server_info), form);
        assert_eq!(ServerInfo::try_from(form).unwrap(), server_info);
    }
}
