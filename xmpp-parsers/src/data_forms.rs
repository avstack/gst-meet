// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::media_element::MediaElement;
use crate::ns;
use crate::util::error::Error;
use crate::Element;
use std::convert::TryFrom;

generate_element!(
    /// Represents one of the possible values for a list- field.
    Option_, "option", DATA_FORMS,
    attributes: [
        /// The optional label to be displayed to the user for this option.
        label: Option<String> = "label"
    ],
    children: [
        /// The value returned to the server when selecting this option.
        value: Required<String> = ("value", DATA_FORMS) => String
    ]
);

generate_attribute!(
    /// The type of a [field](struct.Field.html) element.
    FieldType, "type", {
        /// This field can only take the values "0" or "false" for a false
        /// value, and "1" or "true" for a true value.
        Boolean => "boolean",

        /// This field describes data, it must not be sent back to the
        /// requester.
        Fixed => "fixed",

        /// This field is hidden, it should not be displayed to the user but
        /// should be sent back to the requester.
        Hidden => "hidden",

        /// This field accepts one or more [JIDs](../../jid/struct.Jid.html).
        /// A client may want to let the user autocomplete them based on their
        /// contacts list for instance.
        JidMulti => "jid-multi",

        /// This field accepts one [JID](../../jid/struct.Jid.html).  A client
        /// may want to let the user autocomplete it based on their contacts
        /// list for instance.
        JidSingle => "jid-single",

        /// This field accepts one or more values from the list provided as
        /// [options](struct.Option_.html).
        ListMulti => "list-multi",

        /// This field accepts one value from the list provided as
        /// [options](struct.Option_.html).
        ListSingle => "list-single",

        /// This field accepts one or more free form text lines.
        TextMulti => "text-multi",

        /// This field accepts one free form password, a client should hide it
        /// in its user interface.
        TextPrivate => "text-private",

        /// This field accepts one free form text line.
        TextSingle => "text-single",
    }, Default = TextSingle
);

/// Represents a field in a [data form](struct.DataForm.html).
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// The unique identifier for this field, in the form.
    pub var: String,

    /// The type of this field.
    pub type_: FieldType,

    /// The label to be possibly displayed to the user for this field.
    pub label: Option<String>,

    /// The form will be rejected if this field isn’t present.
    pub required: bool,

    /// A list of allowed values.
    pub options: Vec<Option_>,

    /// The values provided for this field.
    pub values: Vec<String>,

    /// A list of media related to this field.
    pub media: Vec<MediaElement>,
}

impl Field {
    fn is_list(&self) -> bool {
        self.type_ == FieldType::ListSingle || self.type_ == FieldType::ListMulti
    }
}

impl TryFrom<Element> for Field {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Field, Error> {
        check_self!(elem, "field", DATA_FORMS);
        check_no_unknown_attributes!(elem, "field", ["label", "type", "var"]);
        let mut field = Field {
            var: get_attr!(elem, "var", Required),
            type_: get_attr!(elem, "type", Default),
            label: get_attr!(elem, "label", Option),
            required: false,
            options: vec![],
            values: vec![],
            media: vec![],
        };
        for element in elem.children() {
            if element.is("value", ns::DATA_FORMS) {
                check_no_children!(element, "value");
                check_no_attributes!(element, "value");
                field.values.push(element.text());
            } else if element.is("required", ns::DATA_FORMS) {
                if field.required {
                    return Err(Error::ParseError("More than one required element."));
                }
                check_no_children!(element, "required");
                check_no_attributes!(element, "required");
                field.required = true;
            } else if element.is("option", ns::DATA_FORMS) {
                if !field.is_list() {
                    return Err(Error::ParseError("Option element found in non-list field."));
                }
                let option = Option_::try_from(element.clone())?;
                field.options.push(option);
            } else if element.is("media", ns::MEDIA_ELEMENT) {
                let media_element = MediaElement::try_from(element.clone())?;
                field.media.push(media_element);
            } else {
                return Err(Error::ParseError(
                    "Field child isn’t a value, option or media element.",
                ));
            }
        }
        Ok(field)
    }
}

impl From<Field> for Element {
    fn from(field: Field) -> Element {
        Element::builder("field", ns::DATA_FORMS)
            .attr("var", field.var)
            .attr("type", field.type_)
            .attr("label", field.label)
            .append_all(if field.required {
                Some(Element::builder("required", ns::DATA_FORMS))
            } else {
                None
            })
            .append_all(field.options.iter().cloned().map(Element::from))
            .append_all(
                field
                    .values
                    .into_iter()
                    .map(|value| Element::builder("value", ns::DATA_FORMS).append(value)),
            )
            .append_all(field.media.iter().cloned().map(Element::from))
            .build()
    }
}

generate_attribute!(
    /// Represents the type of a [data form](struct.DataForm.html).
    DataFormType, "type", {
        /// This is a cancel request for a prior type="form" data form.
        Cancel => "cancel",

        /// This is a request for the recipient to fill this form and send it
        /// back as type="submit".
        Form => "form",

        /// This is a result form, which contains what the requester asked for.
        Result_ => "result",

        /// This is a complete response to a form received before.
        Submit => "submit",
    }
);

/// This is a form to be sent to another entity for filling.
#[derive(Debug, Clone, PartialEq)]
pub struct DataForm {
    /// The type of this form, telling the other party which action to execute.
    pub type_: DataFormType,

    /// An easy accessor for the FORM_TYPE of this form, see
    /// [XEP-0068](https://xmpp.org/extensions/xep-0068.html) for more
    /// information.
    pub form_type: Option<String>,

    /// The title of this form.
    pub title: Option<String>,

    /// The instructions given with this form.
    pub instructions: Option<String>,

    /// A list of fields comprising this form.
    pub fields: Vec<Field>,
}

impl TryFrom<Element> for DataForm {
    type Error = Error;

    fn try_from(elem: Element) -> Result<DataForm, Error> {
        check_self!(elem, "x", DATA_FORMS);
        check_no_unknown_attributes!(elem, "x", ["type"]);
        let type_ = get_attr!(elem, "type", Required);
        let mut form = DataForm {
            type_,
            form_type: None,
            title: None,
            instructions: None,
            fields: vec![],
        };
        for child in elem.children() {
            if child.is("title", ns::DATA_FORMS) {
                if form.title.is_some() {
                    return Err(Error::ParseError("More than one title in form element."));
                }
                check_no_children!(child, "title");
                check_no_attributes!(child, "title");
                form.title = Some(child.text());
            } else if child.is("instructions", ns::DATA_FORMS) {
                if form.instructions.is_some() {
                    return Err(Error::ParseError(
                        "More than one instructions in form element.",
                    ));
                }
                check_no_children!(child, "instructions");
                check_no_attributes!(child, "instructions");
                form.instructions = Some(child.text());
            } else if child.is("field", ns::DATA_FORMS) {
                let field = Field::try_from(child.clone())?;
                if field.var == "FORM_TYPE" {
                    let mut field = field;
                    if form.form_type.is_some() {
                        return Err(Error::ParseError("More than one FORM_TYPE in a data form."));
                    }
                    if field.type_ != FieldType::Hidden {
                        return Err(Error::ParseError("Invalid field type for FORM_TYPE."));
                    }
                    if field.values.len() != 1 {
                        return Err(Error::ParseError("Wrong number of values in FORM_TYPE."));
                    }
                    form.form_type = field.values.pop();
                } else {
                    form.fields.push(field);
                }
            } else {
                return Err(Error::ParseError("Unknown child in data form element."));
            }
        }
        Ok(form)
    }
}

impl From<DataForm> for Element {
    fn from(form: DataForm) -> Element {
        Element::builder("x", ns::DATA_FORMS)
            .attr("type", form.type_)
            .append_all(
                form.title
                    .map(|title| Element::builder("title", ns::DATA_FORMS).append(title)),
            )
            .append_all(
                form.instructions
                    .map(|text| Element::builder("instructions", ns::DATA_FORMS).append(text)),
            )
            .append_all(form.form_type.map(|form_type| {
                Element::builder("field", ns::DATA_FORMS)
                    .attr("var", "FORM_TYPE")
                    .attr("type", "hidden")
                    .append(Element::builder("value", ns::DATA_FORMS).append(form_type))
            }))
            .append_all(form.fields.iter().cloned().map(Element::from))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Option_, 24);
        assert_size!(FieldType, 1);
        assert_size!(Field, 64);
        assert_size!(DataFormType, 1);
        assert_size!(DataForm, 52);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Option_, 48);
        assert_size!(FieldType, 1);
        assert_size!(Field, 128);
        assert_size!(DataFormType, 1);
        assert_size!(DataForm, 104);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<x xmlns='jabber:x:data' type='result'/>".parse().unwrap();
        let form = DataForm::try_from(elem).unwrap();
        assert_eq!(form.type_, DataFormType::Result_);
        assert!(form.form_type.is_none());
        assert!(form.fields.is_empty());
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<x xmlns='jabber:x:data'/>".parse().unwrap();
        let error = DataForm::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'type' missing.");

        let elem: Element = "<x xmlns='jabber:x:data' type='coucou'/>".parse().unwrap();
        let error = DataForm::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'type' attribute.");
    }

    #[test]
    fn test_wrong_child() {
        let elem: Element = "<x xmlns='jabber:x:data' type='cancel'><coucou/></x>"
            .parse()
            .unwrap();
        let error = DataForm::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in data form element.");
    }

    #[test]
    fn option() {
        let elem: Element =
            "<option xmlns='jabber:x:data' label='Coucou !'><value>coucou</value></option>"
                .parse()
                .unwrap();
        let option = Option_::try_from(elem).unwrap();
        assert_eq!(&option.label.unwrap(), "Coucou !");
        assert_eq!(&option.value, "coucou");

        let elem: Element = "<option xmlns='jabber:x:data' label='Coucou !'/>"
            .parse()
            .unwrap();
        let error = Option_::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Missing child value in option element.");

        let elem: Element = "<option xmlns='jabber:x:data' label='Coucou !'><value>coucou</value><value>error</value></option>".parse().unwrap();
        let error = Option_::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(
            message,
            "Element option must not have more than one value child."
        );
    }
}
