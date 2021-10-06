// Copyright (c) 2017 Maxime “pep” Buquet <pep@bouah.net>
// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::ns;
use crate::util::error::Error;
use crate::Element;
use jid::FullJid;
use std::convert::TryFrom;

generate_attribute_enum!(
/// Lists all of the possible status codes used in MUC presences.
Status, "status", MUC_USER, "code", {
    /// Inform user that any occupant is allowed to see the user's full JID
    NonAnonymousRoom => 100,

    /// Inform user that his or her affiliation changed while not in the room
    AffiliationChange => 101,

    /// Inform occupants that room now shows unavailable members
    ConfigShowsUnavailableMembers => 102,

    /// Inform occupants that room now does not show unavailable members
    ConfigHidesUnavailableMembers => 103,

    /// Inform occupants that a non-privacy-related room configuration change has occurred
    ConfigNonPrivacyRelated => 104,

    /// Inform user that presence refers to itself
    SelfPresence => 110,

    /// Inform occupants that room logging is now enabled
    ConfigRoomLoggingEnabled => 170,

    /// Inform occupants that room logging is now disabled
    ConfigRoomLoggingDisabled => 171,

    /// Inform occupants that the room is now non-anonymous
    ConfigRoomNonAnonymous => 172,

    /// Inform occupants that the room is now semi-anonymous
    ConfigRoomSemiAnonymous => 173,

    /// Inform user that a new room has been created
    RoomHasBeenCreated => 201,

    /// Inform user that service has assigned or modified occupant's roomnick
    AssignedNick => 210,

    /// Inform user that he or she has been banned from the room
    Banned => 301,

    /// Inform all occupants of new room nickname
    NewNick => 303,

    /// Inform user that he or she has been kicked from the room
    Kicked => 307,

    /// Inform user that he or she is being removed from the room
    /// because of an affiliation change
    RemovalFromRoom => 321,

    /// Inform user that he or she is being removed from the room
    /// because the room has been changed to members-only and the
    /// user is not a member
    ConfigMembersOnly => 322,

    /// Inform user that he or she is being removed from the room
    /// because the MUC service is being shut down
    ServiceShutdown => 332,
});

/// Optional <actor/> element used in <item/> elements inside presence stanzas of type
/// "unavailable" that are sent to users who are kick or banned, as well as within IQs for tracking
/// purposes. -- CHANGELOG  0.17 (2002-10-23)
///
/// Possesses a 'jid' and a 'nick' attribute, so that an action can be attributed either to a real
/// JID or to a roomnick. -- CHANGELOG  1.25 (2012-02-08)
#[derive(Debug, Clone, PartialEq)]
pub enum Actor {
    /// The full JID associated with this user.
    Jid(FullJid),

    /// The nickname of this user.
    Nick(String),
}

impl TryFrom<Element> for Actor {
    type Error = Error;

    fn try_from(elem: Element) -> Result<Actor, Error> {
        check_self!(elem, "actor", MUC_USER);
        check_no_unknown_attributes!(elem, "actor", ["jid", "nick"]);
        check_no_children!(elem, "actor");
        let jid: Option<FullJid> = get_attr!(elem, "jid", Option);
        let nick = get_attr!(elem, "nick", Option);

        match (jid, nick) {
            (Some(_), Some(_)) | (None, None) => Err(Error::ParseError(
                "Either 'jid' or 'nick' attribute is required.",
            )),
            (Some(jid), _) => Ok(Actor::Jid(jid)),
            (_, Some(nick)) => Ok(Actor::Nick(nick)),
        }
    }
}

impl From<Actor> for Element {
    fn from(actor: Actor) -> Element {
        let elem = Element::builder("actor", ns::MUC_USER);

        (match actor {
            Actor::Jid(jid) => elem.attr("jid", jid),
            Actor::Nick(nick) => elem.attr("nick", nick),
        })
        .build()
    }
}

generate_element!(
    /// Used to continue a one-to-one discussion in a room, with more than one
    /// participant.
    Continue, "continue", MUC_USER,
    attributes: [
        /// The thread to continue in this room.
        thread: Option<String> = "thread",
    ]
);

generate_elem_id!(
    /// A reason for inviting, declining, etc. a request.
    Reason,
    "reason",
    MUC_USER
);

generate_attribute!(
    /// The affiliation of an entity with a room, which isn’t tied to its
    /// presence in it.
    Affiliation, "affiliation", {
        /// The user who created the room, or who got appointed by its creator
        /// to be their equal.
        Owner => "owner",

        /// A user who has been empowered by an owner to do administrative
        /// operations.
        Admin => "admin",

        /// A user who is whitelisted to speak in moderated rooms, or to join a
        /// member-only room.
        Member => "member",

        /// A user who has been banned from this room.
        Outcast => "outcast",

        /// A normal participant.
        None => "none",
    }, Default = None
);

generate_attribute!(
    /// The current role of an entity in a room, it can be changed by an owner
    /// or an administrator but will be lost once they leave the room.
    Role, "role", {
        /// This user can kick other participants, as well as grant and revoke
        /// them voice.
        Moderator => "moderator",

        /// A user who can speak in this room.
        Participant => "participant",

        /// A user who cannot speak in this room, and must request voice before
        /// doing so.
        Visitor => "visitor",

        /// A user who is absent from the room.
        None => "none",
    }, Default = None
);

generate_element!(
    /// An item representing a user in a room.
    Item, "item", MUC_USER, attributes: [
        /// The affiliation of this user with the room.
        affiliation: Required<Affiliation> = "affiliation",

        /// The real JID of this user, if you are allowed to see it.
        jid: Option<FullJid> = "jid",

        /// The current nickname of this user.
        nick: Option<String> = "nick",

        /// The current role of this user.
        role: Required<Role> = "role",
    ], children: [
        /// The actor affected by this item.
        actor: Option<Actor> = ("actor", MUC_USER) => Actor,

        /// Whether this continues a one-to-one discussion.
        continue_: Option<Continue> = ("continue", MUC_USER) => Continue,

        /// A reason for this item.
        reason: Option<Reason> = ("reason", MUC_USER) => Reason
    ]
);

impl Item {
    /// Creates a new item with the given affiliation and role.
    pub fn new(affiliation: Affiliation, role: Role) -> Item {
        Item {
            affiliation,
            role,
            jid: None,
            nick: None,
            actor: None,
            continue_: None,
            reason: None,
        }
    }
}

generate_element!(
    /// The main muc#user element.
    MucUser, "x", MUC_USER, children: [
        /// List of statuses applying to this item.
        status: Vec<Status> = ("status", MUC_USER) => Status,

        /// List of items.
        items: Vec<Item> = ("item", MUC_USER) => Item
    ]
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc#user'/>
        "
        .parse()
        .unwrap();
        MucUser::try_from(elem).unwrap();
    }

    #[test]
    fn statuses_and_items() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc#user'>
                <status code='101'/>
                <status code='102'/>
                <item affiliation='member' role='moderator'/>
            </x>
        "
        .parse()
        .unwrap();
        let muc_user = MucUser::try_from(elem).unwrap();
        assert_eq!(muc_user.status.len(), 2);
        assert_eq!(muc_user.status[0], Status::AffiliationChange);
        assert_eq!(muc_user.status[1], Status::ConfigShowsUnavailableMembers);
        assert_eq!(muc_user.items.len(), 1);
        assert_eq!(muc_user.items[0].affiliation, Affiliation::Member);
        assert_eq!(muc_user.items[0].role, Role::Moderator);
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc#user'>
                <coucou/>
            </x>
        "
        .parse()
        .unwrap();
        let error = MucUser::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in x element.");
    }

    #[test]
    fn test_serialise() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc#user'/>
        "
        .parse()
        .unwrap();
        let muc = MucUser {
            status: vec![],
            items: vec![],
        };
        let elem2 = muc.into();
        assert_eq!(elem, elem2);
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_invalid_attribute() {
        let elem: Element = "
            <x xmlns='http://jabber.org/protocol/muc#user' coucou=''/>
        "
        .parse()
        .unwrap();
        let error = MucUser::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in x element.");
    }

    #[test]
    fn test_status_simple() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='110'/>
        "
        .parse()
        .unwrap();
        Status::try_from(elem).unwrap();
    }

    #[test]
    fn test_status_invalid() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user'/>
        "
        .parse()
        .unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'code' missing.");
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_status_invalid_child() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='110'>
                <foo/>
            </status>
        "
        .parse()
        .unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in status element.");
    }

    #[test]
    fn test_status_simple_code() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='307'/>
        "
        .parse()
        .unwrap();
        let status = Status::try_from(elem).unwrap();
        assert_eq!(status, Status::Kicked);
    }

    #[test]
    fn test_status_invalid_code() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='666'/>
        "
        .parse()
        .unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Invalid status code value.");
    }

    #[test]
    fn test_status_invalid_code2() {
        let elem: Element = "
            <status xmlns='http://jabber.org/protocol/muc#user' code='coucou'/>
        "
        .parse()
        .unwrap();
        let error = Status::try_from(elem).unwrap_err();
        let error = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(error.to_string(), "invalid digit found in string");
    }

    #[test]
    fn test_actor_required_attributes() {
        let elem: Element = "
            <actor xmlns='http://jabber.org/protocol/muc#user'/>
        "
        .parse()
        .unwrap();
        let error = Actor::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Either 'jid' or 'nick' attribute is required.");
    }

    #[test]
    fn test_actor_required_attributes2() {
        let elem: Element = "
            <actor xmlns='http://jabber.org/protocol/muc#user'
                   jid='foo@bar/baz'
                   nick='baz'/>
        "
        .parse()
        .unwrap();
        let error = Actor::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Either 'jid' or 'nick' attribute is required.");
    }

    #[test]
    fn test_actor_jid() {
        let elem: Element = "
            <actor xmlns='http://jabber.org/protocol/muc#user'
                   jid='foo@bar/baz'/>
        "
        .parse()
        .unwrap();
        let actor = Actor::try_from(elem).unwrap();
        let jid = match actor {
            Actor::Jid(jid) => jid,
            _ => panic!(),
        };
        assert_eq!(jid, "foo@bar/baz".parse::<FullJid>().unwrap());
    }

    #[test]
    fn test_actor_nick() {
        let elem: Element = "
            <actor xmlns='http://jabber.org/protocol/muc#user' nick='baz'/>
        "
        .parse()
        .unwrap();
        let actor = Actor::try_from(elem).unwrap();
        let nick = match actor {
            Actor::Nick(nick) => nick,
            _ => panic!(),
        };
        assert_eq!(nick, "baz".to_owned());
    }

    #[test]
    fn test_continue_simple() {
        let elem: Element = "
            <continue xmlns='http://jabber.org/protocol/muc#user'/>
        "
        .parse()
        .unwrap();
        Continue::try_from(elem).unwrap();
    }

    #[test]
    fn test_continue_thread_attribute() {
        let elem: Element = "
            <continue xmlns='http://jabber.org/protocol/muc#user'
                      thread='foo'/>
        "
        .parse()
        .unwrap();
        let continue_ = Continue::try_from(elem).unwrap();
        assert_eq!(continue_.thread, Some("foo".to_owned()));
    }

    #[test]
    fn test_continue_invalid() {
        let elem: Element = "
            <continue xmlns='http://jabber.org/protocol/muc#user'>
                <foobar/>
            </continue>
        "
        .parse()
        .unwrap();
        let continue_ = Continue::try_from(elem).unwrap_err();
        let message = match continue_ {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in continue element.".to_owned());
    }

    #[test]
    fn test_reason_simple() {
        let elem: Element = "
            <reason xmlns='http://jabber.org/protocol/muc#user'>Reason</reason>"
            .parse()
            .unwrap();
        let elem2 = elem.clone();
        let reason = Reason::try_from(elem).unwrap();
        assert_eq!(reason.0, "Reason".to_owned());

        let elem3 = reason.into();
        assert_eq!(elem2, elem3);
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_reason_invalid_attribute() {
        let elem: Element = "
            <reason xmlns='http://jabber.org/protocol/muc#user' foo='bar'/>
        "
        .parse()
        .unwrap();
        let error = Reason::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in reason element.".to_owned());
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_reason_invalid() {
        let elem: Element = "
            <reason xmlns='http://jabber.org/protocol/muc#user'>
                <foobar/>
            </reason>
        "
        .parse()
        .unwrap();
        let error = Reason::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in reason element.".to_owned());
    }

    #[cfg(not(feature = "disable-validation"))]
    #[test]
    fn test_item_invalid_attr() {
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  foo='bar'/>
        "
        .parse()
        .unwrap();
        let error = Item::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown attribute in item element.".to_owned());
    }

    #[test]
    fn test_item_affiliation_role_attr() {
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'/>
        "
        .parse()
        .unwrap();
        Item::try_from(elem).unwrap();
    }

    #[test]
    fn test_item_affiliation_role_invalid_attr() {
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'/>
        "
        .parse()
        .unwrap();
        let error = Item::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'role' missing.".to_owned());
    }

    #[test]
    fn test_item_nick_attr() {
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'
                  nick='foobar'/>
        "
        .parse()
        .unwrap();
        let item = Item::try_from(elem).unwrap();
        match item {
            Item { nick, .. } => assert_eq!(nick, Some("foobar".to_owned())),
        }
    }

    #[test]
    fn test_item_affiliation_role_invalid_attr2() {
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  role='moderator'/>
        "
        .parse()
        .unwrap();
        let error = Item::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(
            message,
            "Required attribute 'affiliation' missing.".to_owned()
        );
    }

    #[test]
    fn test_item_role_actor_child() {
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'>
                <actor nick='foobar'/>
            </item>
        "
        .parse()
        .unwrap();
        let item = Item::try_from(elem).unwrap();
        match item {
            Item { actor, .. } => assert_eq!(actor, Some(Actor::Nick("foobar".to_owned()))),
        }
    }

    #[test]
    fn test_item_role_continue_child() {
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'>
                <continue thread='foobar'/>
            </item>
        "
        .parse()
        .unwrap();
        let item = Item::try_from(elem).unwrap();
        let continue_1 = Continue {
            thread: Some("foobar".to_owned()),
        };
        match item {
            Item {
                continue_: Some(continue_2),
                ..
            } => assert_eq!(continue_2.thread, continue_1.thread),
            _ => panic!(),
        }
    }

    #[test]
    fn test_item_role_reason_child() {
        let elem: Element = "
            <item xmlns='http://jabber.org/protocol/muc#user'
                  affiliation='member'
                  role='moderator'>
                <reason>foobar</reason>
            </item>
        "
        .parse()
        .unwrap();
        let item = Item::try_from(elem).unwrap();
        match item {
            Item { reason, .. } => assert_eq!(reason, Some(Reason("foobar".to_owned()))),
        }
    }

    #[test]
    fn test_serialize_item() {
        let reference: Element = "<item xmlns='http://jabber.org/protocol/muc#user' affiliation='member' role='moderator'><actor nick='foobar'/><continue thread='foobar'/><reason>foobar</reason></item>"
        .parse()
        .unwrap();

        let elem: Element = "<actor xmlns='http://jabber.org/protocol/muc#user' nick='foobar'/>"
            .parse()
            .unwrap();
        let actor = Actor::try_from(elem).unwrap();

        let elem: Element =
            "<continue xmlns='http://jabber.org/protocol/muc#user' thread='foobar'/>"
                .parse()
                .unwrap();
        let continue_ = Continue::try_from(elem).unwrap();

        let elem: Element = "<reason xmlns='http://jabber.org/protocol/muc#user'>foobar</reason>"
            .parse()
            .unwrap();
        let reason = Reason::try_from(elem).unwrap();

        let item = Item {
            affiliation: Affiliation::Member,
            role: Role::Moderator,
            jid: None,
            nick: None,
            actor: Some(actor),
            reason: Some(reason),
            continue_: Some(continue_),
        };

        let serialized: Element = item.into();
        assert_eq!(serialized, reference);
    }
}
