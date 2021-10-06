// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ns;
use crate::util::error::Error;
use crate::Element;
use std::convert::TryFrom;

/// Requests paging through a potentially big set of items (represented by an
/// UID).
#[derive(Debug, Clone, PartialEq)]
pub struct SetQuery {
    /// Limit the number of items, or use the recipient’s defaults if None.
    pub max: Option<usize>,

    /// The UID after which to give results, or if None it is the element
    /// “before” the first item, effectively an index of negative one.
    pub after: Option<String>,

    /// The UID before which to give results, or if None it starts with the
    /// last page of the full set.
    pub before: Option<String>,

    /// Numerical index of the page (deprecated).
    pub index: Option<usize>,
}

impl TryFrom<Element> for SetQuery {
    type Error = Error;

    fn try_from(elem: Element) -> Result<SetQuery, Error> {
        check_self!(elem, "set", RSM, "RSM set");
        let mut set = SetQuery {
            max: None,
            after: None,
            before: None,
            index: None,
        };
        for child in elem.children() {
            if child.is("max", ns::RSM) {
                if set.max.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one max."));
                }
                set.max = Some(child.text().parse()?);
            } else if child.is("after", ns::RSM) {
                if set.after.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one after."));
                }
                set.after = Some(child.text());
            } else if child.is("before", ns::RSM) {
                if set.before.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one before."));
                }
                set.before = Some(child.text());
            } else if child.is("index", ns::RSM) {
                if set.index.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one index."));
                }
                set.index = Some(child.text().parse()?);
            } else {
                return Err(Error::ParseError("Unknown child in set element."));
            }
        }
        Ok(set)
    }
}

impl From<SetQuery> for Element {
    fn from(set: SetQuery) -> Element {
        Element::builder("set", ns::RSM)
            .append_all(
                set.max
                    .map(|max| Element::builder("max", ns::RSM).append(format!("{}", max))),
            )
            .append_all(
                set.after
                    .map(|after| Element::builder("after", ns::RSM).append(after)),
            )
            .append_all(
                set.before
                    .map(|before| Element::builder("before", ns::RSM).append(before)),
            )
            .append_all(
                set.index
                    .map(|index| Element::builder("index", ns::RSM).append(format!("{}", index))),
            )
            .build()
    }
}

/// Describes the paging result of a [query](struct.SetQuery.html).
#[derive(Debug, Clone, PartialEq)]
pub struct SetResult {
    /// The UID of the first item of the page.
    pub first: Option<String>,

    /// The position of the [first item](#structfield.first) in the full set
    /// (which may be approximate).
    pub first_index: Option<usize>,

    /// The UID of the last item of the page.
    pub last: Option<String>,

    /// How many items there are in the full set (which may be approximate).
    pub count: Option<usize>,
}

impl TryFrom<Element> for SetResult {
    type Error = Error;

    fn try_from(elem: Element) -> Result<SetResult, Error> {
        check_self!(elem, "set", RSM, "RSM set");
        let mut set = SetResult {
            first: None,
            first_index: None,
            last: None,
            count: None,
        };
        for child in elem.children() {
            if child.is("first", ns::RSM) {
                if set.first.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one first."));
                }
                set.first_index = get_attr!(child, "index", Option);
                set.first = Some(child.text());
            } else if child.is("last", ns::RSM) {
                if set.last.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one last."));
                }
                set.last = Some(child.text());
            } else if child.is("count", ns::RSM) {
                if set.count.is_some() {
                    return Err(Error::ParseError("Set can’t have more than one count."));
                }
                set.count = Some(child.text().parse()?);
            } else {
                return Err(Error::ParseError("Unknown child in set element."));
            }
        }
        Ok(set)
    }
}

impl From<SetResult> for Element {
    fn from(set: SetResult) -> Element {
        let first = set.first.clone().map(|first| {
            Element::builder("first", ns::RSM)
                .attr("index", set.first_index)
                .append(first)
        });
        Element::builder("set", ns::RSM)
            .append_all(first)
            .append_all(
                set.last
                    .map(|last| Element::builder("last", ns::RSM).append(last)),
            )
            .append_all(
                set.count
                    .map(|count| Element::builder("count", ns::RSM).append(format!("{}", count))),
            )
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(SetQuery, 40);
        assert_size!(SetResult, 40);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(SetQuery, 80);
        assert_size!(SetResult, 80);
    }

    #[test]
    fn test_simple() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'/>"
            .parse()
            .unwrap();
        let set = SetQuery::try_from(elem).unwrap();
        assert_eq!(set.max, None);
        assert_eq!(set.after, None);
        assert_eq!(set.before, None);
        assert_eq!(set.index, None);

        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'/>"
            .parse()
            .unwrap();
        let set = SetResult::try_from(elem).unwrap();
        match set.first {
            Some(_) => panic!(),
            None => (),
        }
        assert_eq!(set.last, None);
        assert_eq!(set.count, None);
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>"
            .parse()
            .unwrap();
        let error = SetQuery::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a RSM set element.");

        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>"
            .parse()
            .unwrap();
        let error = SetResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not a RSM set element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'><coucou/></set>"
            .parse()
            .unwrap();
        let error = SetQuery::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in set element.");

        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'><coucou/></set>"
            .parse()
            .unwrap();
        let error = SetResult::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in set element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'/>"
            .parse()
            .unwrap();
        let rsm = SetQuery {
            max: None,
            after: None,
            before: None,
            index: None,
        };
        let elem2 = rsm.into();
        assert_eq!(elem, elem2);

        let elem: Element = "<set xmlns='http://jabber.org/protocol/rsm'/>"
            .parse()
            .unwrap();
        let rsm = SetResult {
            first: None,
            first_index: None,
            last: None,
            count: None,
        };
        let elem2 = rsm.into();
        assert_eq!(elem, elem2);
    }

    #[test]
    fn test_first_index() {
        let elem: Element =
            "<set xmlns='http://jabber.org/protocol/rsm'><first index='4'>coucou</first></set>"
                .parse()
                .unwrap();
        let elem1 = elem.clone();
        let set = SetResult::try_from(elem).unwrap();
        assert_eq!(set.first, Some(String::from("coucou")));
        assert_eq!(set.first_index, Some(4));

        let set2 = SetResult {
            first: Some(String::from("coucou")),
            first_index: Some(4),
            last: None,
            count: None,
        };
        let elem2 = set2.into();
        assert_eq!(elem1, elem2);
    }
}
