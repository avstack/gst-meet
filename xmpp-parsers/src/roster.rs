// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::iq::{IqGetPayload, IqResultPayload, IqSetPayload};
use jid::BareJid;

generate_elem_id!(
    /// Represents a group a contact is part of.
    Group,
    "group",
    ROSTER
);

generate_attribute!(
    /// The state of your mutual subscription with a contact.
    Subscription, "subscription", {
        /// The user doesn’t have any subscription to this contact’s presence,
        /// and neither does this contact.
        None => "none",

        /// Only this contact has a subscription with you, not the opposite.
        From => "from",

        /// Only you have a subscription with this contact, not the opposite.
        To => "to",

        /// Both you and your contact are subscribed to each other’s presence.
        Both => "both",

        /// In a roster set, this asks the server to remove this contact item
        /// from your roster.
        Remove => "remove",
    }, Default = None
);

generate_attribute!(
    /// The sub-state of subscription with a contact.
    Ask, "ask", (
        /// Pending sub-state of the 'none' subscription state.
        Subscribe => "subscribe"
    )
);

generate_element!(
    /// Contact from the user’s contact list.
    Item, "item", ROSTER,
    attributes: [
        /// JID of this contact.
        jid: Required<BareJid> = "jid",

        /// Name of this contact.
        name: OptionEmpty<String> = "name",

        /// Subscription status of this contact.
        subscription: Default<Subscription> = "subscription",

        /// Indicates “Pending Out” sub-states for this contact.
        ask: Default<Ask> = "ask",
    ],

    children: [
        /// Groups this contact is part of.
        groups: Vec<Group> = ("group", ROSTER) => Group
    ]
);

generate_element!(
    /// The contact list of the user.
    Roster, "query", ROSTER,
    attributes: [
        /// Version of the contact list.
        ///
        /// This is an opaque string that should only be sent back to the server on
        /// a new connection, if this client is storing the contact list between
        /// connections.
        ver: Option<String> = "ver"
    ],
    children: [
        /// List of the contacts of the user.
        items: Vec<Item> = ("item", ROSTER) => Item
    ]
);

impl IqGetPayload for Roster {}
impl IqSetPayload for Roster {}
impl IqResultPayload for Roster {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::error::Error;
    use crate::Element;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_size() {
        assert_size!(Group, 12);
        assert_size!(Subscription, 1);
        assert_size!(Ask, 1);
        assert_size!(Item, 52);
        assert_size!(Roster, 24);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_size() {
        assert_size!(Group, 24);
        assert_size!(Subscription, 1);
        assert_size!(Ask, 1);
        assert_size!(Item, 104);
        assert_size!(Roster, 48);
    }

    #[test]
    fn test_get() {
        let elem: Element = "<query xmlns='jabber:iq:roster'/>".parse().unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert!(roster.items.is_empty());
    }

    #[test]
    fn test_result() {
        let elem: Element = "<query xmlns='jabber:iq:roster' ver='ver7'><item jid='nurse@example.com'/><item jid='romeo@example.net'/></query>".parse().unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert_eq!(roster.ver, Some(String::from("ver7")));
        assert_eq!(roster.items.len(), 2);

        let elem2: Element = "<query xmlns='jabber:iq:roster' ver='ver7'><item jid='nurse@example.com'/><item jid='romeo@example.net' name=''/></query>".parse().unwrap();
        let roster2 = Roster::try_from(elem2).unwrap();
        assert_eq!(roster.items, roster2.items);

        let elem: Element = "<query xmlns='jabber:iq:roster' ver='ver9'/>"
            .parse()
            .unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert_eq!(roster.ver, Some(String::from("ver9")));
        assert!(roster.items.is_empty());

        let elem: Element = r#"
<query xmlns='jabber:iq:roster' ver='ver11'>
  <item jid='romeo@example.net'
        name='Romeo'
        subscription='both'>
    <group>Friends</group>
  </item>
  <item jid='mercutio@example.com'
        name='Mercutio'
        subscription='from'/>
  <item jid='benvolio@example.net'
        name='Benvolio'
        subscription='both'/>
  <item jid='contact@example.org'
        subscription='none'
        ask='subscribe'
        name='MyContact'>
      <group>MyBuddies</group>
  </item>
</query>
"#
        .parse()
        .unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert_eq!(roster.ver, Some(String::from("ver11")));
        assert_eq!(roster.items.len(), 4);
        assert_eq!(roster.items[0].jid, BareJid::new("romeo", "example.net"));
        assert_eq!(roster.items[0].name, Some(String::from("Romeo")));
        assert_eq!(roster.items[0].subscription, Subscription::Both);
        assert_eq!(roster.items[0].ask, Ask::None);
        assert_eq!(
            roster.items[0].groups,
            vec!(Group::from_str("Friends").unwrap())
        );

        assert_eq!(roster.items[3].jid, BareJid::new("contact", "example.org"));
        assert_eq!(roster.items[3].name, Some(String::from("MyContact")));
        assert_eq!(roster.items[3].subscription, Subscription::None);
        assert_eq!(roster.items[3].ask, Ask::Subscribe);
        assert_eq!(
            roster.items[3].groups,
            vec!(Group::from_str("MyBuddies").unwrap())
        );
    }

    #[test]
    fn test_multiple_groups() {
        let elem: Element = "<query xmlns='jabber:iq:roster'><item jid='test@example.org'><group>A</group><group>B</group></item></query>"
        .parse()
        .unwrap();
        let elem1 = elem.clone();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert_eq!(roster.items.len(), 1);
        assert_eq!(roster.items[0].jid, BareJid::new("test", "example.org"));
        assert_eq!(roster.items[0].name, None);
        assert_eq!(roster.items[0].groups.len(), 2);
        assert_eq!(roster.items[0].groups[0], Group::from_str("A").unwrap());
        assert_eq!(roster.items[0].groups[1], Group::from_str("B").unwrap());
        let elem2 = roster.into();
        assert_eq!(elem1, elem2);
    }

    #[test]
    fn test_set() {
        let elem: Element =
            "<query xmlns='jabber:iq:roster'><item jid='nurse@example.com'/></query>"
                .parse()
                .unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert_eq!(roster.items.len(), 1);

        let elem: Element = r#"
<query xmlns='jabber:iq:roster'>
  <item jid='nurse@example.com'
        name='Nurse'>
    <group>Servants</group>
  </item>
</query>
"#
        .parse()
        .unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert_eq!(roster.items.len(), 1);
        assert_eq!(roster.items[0].jid, BareJid::new("nurse", "example.com"));
        assert_eq!(roster.items[0].name, Some(String::from("Nurse")));
        assert_eq!(roster.items[0].groups.len(), 1);
        assert_eq!(
            roster.items[0].groups[0],
            Group::from_str("Servants").unwrap()
        );

        let elem: Element = r#"
<query xmlns='jabber:iq:roster'>
  <item jid='nurse@example.com'
        subscription='remove'/>
</query>
"#
        .parse()
        .unwrap();
        let roster = Roster::try_from(elem).unwrap();
        assert!(roster.ver.is_none());
        assert_eq!(roster.items.len(), 1);
        assert_eq!(roster.items[0].jid, BareJid::new("nurse", "example.com"));
        assert!(roster.items[0].name.is_none());
        assert!(roster.items[0].groups.is_empty());
        assert_eq!(roster.items[0].subscription, Subscription::Remove);
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid() {
        let elem: Element = "<query xmlns='jabber:iq:roster'><coucou/></query>"
            .parse()
            .unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in query element.");

        let elem: Element = "<query xmlns='jabber:iq:roster' coucou=''/>"
            .parse()
            .unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in query element.");
    }

    #[test]
    fn test_invalid_item() {
        let elem: Element = "<query xmlns='jabber:iq:roster'><item/></query>"
            .parse()
            .unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'jid' missing.");

        /*
        let elem: Element = "<query xmlns='jabber:iq:roster'><item jid=''/></query>".parse().unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let error = match error {
            Error::JidParseError(error) => error,
            _ => panic!(),
        };
        assert_eq!(error.description(), "Invalid JID, I guess?");
        */

        let elem: Element =
            "<query xmlns='jabber:iq:roster'><item jid='coucou'><coucou/></item></query>"
                .parse()
                .unwrap();
        let error = Roster::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in item element.");
    }
}
