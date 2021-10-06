// Copyright (c) 2020 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// TODO: validate nicks by applying the “nickname” profile of the PRECIS OpaqueString class, as
// defined in RFC 7700.

use crate::iq::{IqResultPayload, IqSetPayload};
use crate::message::MessagePayload;
use crate::pubsub::{NodeName, PubSubPayload};
use jid::BareJid;

generate_id!(
    /// The identifier a participant receives when joining a channel.
    ParticipantId
);

impl ParticipantId {
    /// Create a new ParticipantId.
    pub fn new<P: Into<String>>(participant: P) -> ParticipantId {
        ParticipantId(participant.into())
    }
}

generate_id!(
    /// A MIX channel identifier.
    ChannelId
);

generate_element!(
    /// Represents a participant in a MIX channel, usually returned on the
    /// urn:xmpp:mix:nodes:participants PubSub node.
    Participant, "participant", MIX_CORE,
    children: [
        /// The nick of this participant.
        nick: Required<String> = ("nick", MIX_CORE) => String,

        /// The bare JID of this participant.
        // TODO: should be a BareJid!
        jid: Required<String> = ("jid", MIX_CORE) => String
    ]
);

impl PubSubPayload for Participant {}

impl Participant {
    /// Create a new MIX participant.
    pub fn new<J: Into<String>, N: Into<String>>(jid: J, nick: N) -> Participant {
        Participant {
            nick: nick.into(),
            jid: jid.into(),
        }
    }
}

generate_element!(
    /// A node to subscribe to.
    Subscribe, "subscribe", MIX_CORE,
    attributes: [
        /// The PubSub node to subscribe to.
        node: Required<NodeName> = "node",
    ]
);

impl Subscribe {
    /// Create a new Subscribe element.
    pub fn new<N: Into<String>>(node: N) -> Subscribe {
        Subscribe {
            node: NodeName(node.into()),
        }
    }
}

generate_element!(
    /// A request from a user’s server to join a MIX channel.
    Join, "join", MIX_CORE,
    attributes: [
        /// The participant identifier returned by the MIX service on successful join.
        id: Option<ParticipantId> = "id",
    ],
    children: [
        /// The nick requested by the user or set by the service.
        nick: Required<String> = ("nick", MIX_CORE) => String,

        /// Which MIX nodes to subscribe to.
        subscribes: Vec<Subscribe> = ("subscribe", MIX_CORE) => Subscribe
    ]
);

impl IqSetPayload for Join {}
impl IqResultPayload for Join {}

impl Join {
    /// Create a new Join element.
    pub fn from_nick_and_nodes<N: Into<String>>(nick: N, nodes: &[&str]) -> Join {
        let subscribes = nodes
            .into_iter()
            .cloned()
            .map(|n| Subscribe::new(n))
            .collect();
        Join {
            id: None,
            nick: nick.into(),
            subscribes,
        }
    }

    /// Sets the JID on this update-subscription.
    pub fn with_id<I: Into<String>>(mut self, id: I) -> Self {
        self.id = Some(ParticipantId(id.into()));
        self
    }
}

generate_element!(
    /// Update a given subscription.
    UpdateSubscription, "update-subscription", MIX_CORE,
    attributes: [
        /// The JID of the user to be affected.
        // TODO: why is it not a participant id instead?
        jid: Option<BareJid> = "jid",
    ],
    children: [
        /// The list of additional nodes to subscribe to.
        // TODO: what happens when we are already subscribed?  Also, how do we unsubscribe from
        // just one?
        subscribes: Vec<Subscribe> = ("subscribe", MIX_CORE) => Subscribe
    ]
);

impl IqSetPayload for UpdateSubscription {}
impl IqResultPayload for UpdateSubscription {}

impl UpdateSubscription {
    /// Create a new UpdateSubscription element.
    pub fn from_nodes(nodes: &[&str]) -> UpdateSubscription {
        let subscribes = nodes
            .into_iter()
            .cloned()
            .map(|n| Subscribe::new(n))
            .collect();
        UpdateSubscription {
            jid: None,
            subscribes,
        }
    }

    /// Sets the JID on this update-subscription.
    pub fn with_jid(mut self, jid: BareJid) -> Self {
        self.jid = Some(jid);
        self
    }
}

generate_empty_element!(
    /// Request to leave a given MIX channel.  It will automatically unsubscribe the user from all
    /// nodes on this channel.
    Leave,
    "leave",
    MIX_CORE
);

impl IqSetPayload for Leave {}
impl IqResultPayload for Leave {}

generate_element!(
    /// A request to change the user’s nick.
    SetNick, "setnick", MIX_CORE,
    children: [
        /// The new requested nick.
        nick: Required<String> = ("nick", MIX_CORE) => String
    ]
);

impl IqSetPayload for SetNick {}
impl IqResultPayload for SetNick {}

impl SetNick {
    /// Create a new SetNick element.
    pub fn new<N: Into<String>>(nick: N) -> SetNick {
        SetNick { nick: nick.into() }
    }
}

generate_element!(
    /// Message payload describing who actually sent the message, since unlike in MUC, all messages
    /// are sent from the channel’s JID.
    Mix, "mix", MIX_CORE,
    children: [
        /// The nick of the user who said something.
        nick: Required<String> = ("nick", MIX_CORE) => String,

        /// The JID of the user who said something.
        // TODO: should be a BareJid!
        jid: Required<String> = ("jid", MIX_CORE) => String
    ]
);

impl MessagePayload for Mix {}

impl Mix {
    /// Create a new Mix element.
    pub fn new<N: Into<String>, J: Into<String>>(nick: N, jid: J) -> Mix {
        Mix {
            nick: nick.into(),
            jid: jid.into(),
        }
    }
}

generate_element!(
    /// Create a new MIX channel.
    Create, "create", MIX_CORE,
    attributes: [
        /// The requested channel identifier.
        channel: Option<ChannelId> = "channel",
    ]
);

impl IqSetPayload for Create {}
impl IqResultPayload for Create {}

impl Create {
    /// Create a new ad-hoc Create element.
    pub fn new() -> Create {
        Create { channel: None }
    }

    /// Create a new Create element with a channel identifier.
    pub fn from_channel_id<C: Into<String>>(channel: C) -> Create {
        Create {
            channel: Some(ChannelId(channel.into())),
        }
    }
}

generate_element!(
    /// Destroy a given MIX channel.
    Destroy, "destroy", MIX_CORE,
    attributes: [
        /// The channel identifier to be destroyed.
        channel: Required<ChannelId> = "channel",
    ]
);

// TODO: section 7.3.4, example 33, doesn’t mirror the <destroy/> in the iq result unlike every
// other section so far.
impl IqSetPayload for Destroy {}

impl Destroy {
    /// Create a new Destroy element.
    pub fn new<C: Into<String>>(channel: C) -> Destroy {
        Destroy {
            channel: ChannelId(channel.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;
    use std::convert::TryFrom;

    #[test]
    fn participant() {
        let elem: Element = "<participant xmlns='urn:xmpp:mix:core:1'><jid>foo@bar</jid><nick>coucou</nick></participant>"
            .parse()
            .unwrap();
        let participant = Participant::try_from(elem).unwrap();
        assert_eq!(participant.nick, "coucou");
        assert_eq!(participant.jid, "foo@bar");
    }

    #[test]
    fn join() {
        let elem: Element = "<join xmlns='urn:xmpp:mix:core:1'><subscribe node='urn:xmpp:mix:nodes:messages'/><subscribe node='urn:xmpp:mix:nodes:info'/><nick>coucou</nick></join>"
            .parse()
            .unwrap();
        let join = Join::try_from(elem).unwrap();
        assert_eq!(join.nick, "coucou");
        assert_eq!(join.id, None);
        assert_eq!(join.subscribes.len(), 2);
        assert_eq!(join.subscribes[0].node.0, "urn:xmpp:mix:nodes:messages");
        assert_eq!(join.subscribes[1].node.0, "urn:xmpp:mix:nodes:info");
    }

    #[test]
    fn update_subscription() {
        let elem: Element = "<update-subscription xmlns='urn:xmpp:mix:core:1'><subscribe node='urn:xmpp:mix:nodes:participants'/></update-subscription>"
            .parse()
            .unwrap();
        let update_subscription = UpdateSubscription::try_from(elem).unwrap();
        assert_eq!(update_subscription.jid, None);
        assert_eq!(update_subscription.subscribes.len(), 1);
        assert_eq!(
            update_subscription.subscribes[0].node.0,
            "urn:xmpp:mix:nodes:participants"
        );
    }

    #[test]
    fn leave() {
        let elem: Element = "<leave xmlns='urn:xmpp:mix:core:1'/>".parse().unwrap();
        Leave::try_from(elem).unwrap();
    }

    #[test]
    fn setnick() {
        let elem: Element = "<setnick xmlns='urn:xmpp:mix:core:1'><nick>coucou</nick></setnick>"
            .parse()
            .unwrap();
        let setnick = SetNick::try_from(elem).unwrap();
        assert_eq!(setnick.nick, "coucou");
    }

    #[test]
    fn message_mix() {
        let elem: Element =
            "<mix xmlns='urn:xmpp:mix:core:1'><jid>foo@bar</jid><nick>coucou</nick></mix>"
                .parse()
                .unwrap();
        let mix = Mix::try_from(elem).unwrap();
        assert_eq!(mix.nick, "coucou");
        assert_eq!(mix.jid, "foo@bar");
    }

    #[test]
    fn create() {
        let elem: Element = "<create xmlns='urn:xmpp:mix:core:1' channel='coucou'/>"
            .parse()
            .unwrap();
        let create = Create::try_from(elem).unwrap();
        assert_eq!(create.channel.unwrap().0, "coucou");

        let elem: Element = "<create xmlns='urn:xmpp:mix:core:1'/>".parse().unwrap();
        let create = Create::try_from(elem).unwrap();
        assert_eq!(create.channel, None);
    }

    #[test]
    fn destroy() {
        let elem: Element = "<destroy xmlns='urn:xmpp:mix:core:1' channel='coucou'/>"
            .parse()
            .unwrap();
        let destroy = Destroy::try_from(elem).unwrap();
        assert_eq!(destroy.channel.0, "coucou");
    }

    #[test]
    fn serialise() {
        let elem: Element = Join::from_nick_and_nodes("coucou", &["foo", "bar"]).into();
        let xml = String::from(&elem);
        assert_eq!(xml, "<join xmlns=\"urn:xmpp:mix:core:1\"><nick>coucou</nick><subscribe node=\"foo\"/><subscribe node=\"bar\"/></join>");

        let elem: Element = UpdateSubscription::from_nodes(&["foo", "bar"]).into();
        let xml = String::from(&elem);
        assert_eq!(xml, "<update-subscription xmlns=\"urn:xmpp:mix:core:1\"><subscribe node=\"foo\"/><subscribe node=\"bar\"/></update-subscription>");

        let elem: Element = Leave.into();
        let xml = String::from(&elem);
        assert_eq!(xml, "<leave xmlns=\"urn:xmpp:mix:core:1\"/>");

        let elem: Element = SetNick::new("coucou").into();
        let xml = String::from(&elem);
        assert_eq!(
            xml,
            "<setnick xmlns=\"urn:xmpp:mix:core:1\"><nick>coucou</nick></setnick>"
        );

        let elem: Element = Mix::new("coucou", "coucou@example").into();
        let xml = String::from(&elem);
        assert_eq!(
            xml,
            "<mix xmlns=\"urn:xmpp:mix:core:1\"><nick>coucou</nick><jid>coucou@example</jid></mix>"
        );

        let elem: Element = Create::new().into();
        let xml = String::from(&elem);
        assert_eq!(xml, "<create xmlns=\"urn:xmpp:mix:core:1\"/>");

        let elem: Element = Create::from_channel_id("coucou").into();
        let xml = String::from(&elem);
        assert_eq!(
            xml,
            "<create xmlns=\"urn:xmpp:mix:core:1\" channel=\"coucou\"/>"
        );

        let elem: Element = Destroy::new("coucou").into();
        let xml = String::from(&elem);
        assert_eq!(
            xml,
            "<destroy xmlns=\"urn:xmpp:mix:core:1\" channel=\"coucou\"/>"
        );
    }
}
