// Copyright (c) 2021 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::iq::{IqGetPayload, IqResultPayload, IqSetPayload};
use crate::ns;
use crate::util::error::Error;
use jid::Jid;
use minidom::{Element, Node};
use std::convert::TryFrom;

generate_attribute!(
    /// Notes the default archiving preference for the user.
    DefaultPrefs, "default", {
        /// The default is to always log messages in the archive.
        Always => "always",

        /// The default is to never log messages in the archive.
        Never => "never",

        /// The default is to log messages in the archive only for contacts
        /// present in the user’s [roster](../roster/index.html).
        Roster => "roster",
    }
);

/// Controls the archiving preferences of the user.
#[derive(Debug, Clone)]
pub struct Prefs {
    /// The default preference for JIDs in neither
    /// [always](#structfield.always) or [never](#structfield.never) lists.
    pub default_: DefaultPrefs,

    /// The set of JIDs for which to always store messages in the archive.
    pub always: Vec<Jid>,

    /// The set of JIDs for which to never store messages in the archive.
    pub never: Vec<Jid>,
}

impl IqGetPayload for Prefs {}
impl IqSetPayload for Prefs {}
impl IqResultPayload for Prefs {}

impl TryFrom<Element> for Prefs {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Prefs, Error> {
        check_self!(elem, "prefs", MAM);
        check_no_unknown_attributes!(elem, "prefs", ["default"]);
        let mut always = vec![];
        let mut never = vec![];
        for child in elem.children() {
            if child.is("always", ns::MAM) {
                for jid_elem in child.children() {
                    if !jid_elem.is("jid", ns::MAM) {
                        return Err(Error::ParseError("Invalid jid element in always."));
                    }
                    always.push(jid_elem.text().parse()?);
                }
            } else if child.is("never", ns::MAM) {
                for jid_elem in child.children() {
                    if !jid_elem.is("jid", ns::MAM) {
                        return Err(Error::ParseError("Invalid jid element in never."));
                    }
                    never.push(jid_elem.text().parse()?);
                }
            } else {
                return Err(Error::ParseError("Unknown child in prefs element."));
            }
        }
        let default_ = get_attr!(elem, "default", Required);
        Ok(Prefs {
            default_,
            always,
            never,
        })
    }
}

fn serialise_jid_list(name: &str, jids: Vec<Jid>) -> ::std::option::IntoIter<Node> {
    if jids.is_empty() {
        None.into_iter()
    } else {
        Some(
            Element::builder(name, ns::MAM)
                .append_all(
                    jids.into_iter()
                        .map(|jid| Element::builder("jid", ns::MAM).append(String::from(jid))),
                )
                .into(),
        )
        .into_iter()
    }
}

impl From<Prefs> for Element {
    fn from(prefs: Prefs) -> Element {
        Element::builder("prefs", ns::MAM)
            .attr("default", prefs.default_)
            .append_all(serialise_jid_list("always", prefs.always))
            .append_all(serialise_jid_list("never", prefs.never))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jid::BareJid;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(DefaultPrefs, 1);
        assert_size!(Prefs, 28);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(DefaultPrefs, 1);
        assert_size!(Prefs, 56);
    }

    #[test]
    fn test_prefs_get() {
        let elem: Element = "<prefs xmlns='urn:xmpp:mam:2' default='always'/>"
            .parse()
            .unwrap();
        let prefs = Prefs::try_from(elem).unwrap();
        assert!(prefs.always.is_empty());
        assert!(prefs.never.is_empty());

        let elem: Element = r#"
<prefs xmlns='urn:xmpp:mam:2' default='roster'>
  <always/>
  <never/>
</prefs>
"#
        .parse()
        .unwrap();
        let prefs = Prefs::try_from(elem).unwrap();
        assert!(prefs.always.is_empty());
        assert!(prefs.never.is_empty());
    }

    #[test]
    fn test_prefs_result() {
        let elem: Element = r#"
<prefs xmlns='urn:xmpp:mam:2' default='roster'>
  <always>
    <jid>romeo@montague.lit</jid>
  </always>
  <never>
    <jid>montague@montague.lit</jid>
  </never>
</prefs>
"#
        .parse()
        .unwrap();
        let prefs = Prefs::try_from(elem).unwrap();
        assert_eq!(prefs.always, [BareJid::new("romeo", "montague.lit")]);
        assert_eq!(prefs.never, [BareJid::new("montague", "montague.lit")]);

        let elem2 = Element::from(prefs.clone());
        println!("{:?}", elem2);
        let prefs2 = Prefs::try_from(elem2).unwrap();
        assert_eq!(prefs.default_, prefs2.default_);
        assert_eq!(prefs.always, prefs2.always);
        assert_eq!(prefs.never, prefs2.never);
    }
}
