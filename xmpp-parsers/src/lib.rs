//! A crate parsing common XMPP elements into Rust structures.
//!
//! Each module implements the `TryFrom<Element>` trait, which takes a
//! minidom [`Element`] and returns a `Result` whose value is `Ok` if the
//! element parsed correctly, `Err(error::Error)` otherwise.
//!
//! The returned structure can be manipuled as any Rust structure, with each
//! field being public.  You can also create the same structure manually, with
//! some having `new()` and `with_*()` helper methods to create them.
//!
//! Once you are happy with your structure, you can serialise it back to an
//! [`Element`], using either `From` or `Into<Element>`, which give you what
//! you want to be sending on the wire.
//!
//! [`Element`]: ../minidom/element/struct.Element.html

// Copyright (c) 2017-2019 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
// Copyright (c) 2017-2019 Maxime “pep” Buquet <pep@bouah.net>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![deny(missing_docs)]

pub use crate::util::error::Error;
pub use jid::{BareJid, FullJid, Jid, JidParseError};
pub use minidom::Element;

/// XML namespace definitions used through XMPP.
pub mod ns;

#[macro_use]
mod util;

/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod bind;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod iq;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod message;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod presence;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod sasl;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod stanza_error;
/// RFC 6120: Extensible Messaging and Presence Protocol (XMPP): Core
pub mod stream;

/// RFC 6121: Extensible Messaging and Presence Protocol (XMPP): Instant Messaging and Presence
pub mod roster;

/// RFC 7395: An Extensible Messaging and Presence Protocol (XMPP) Subprotocol for WebSocket
pub mod websocket;

/// XEP-0004: Data Forms
pub mod data_forms;

/// XEP-0030: Service Discovery
pub mod disco;

/// XEP-0045: Multi-User Chat
pub mod muc;

/// XEP-0047: In-Band Bytestreams
pub mod ibb;

/// XEP-0048: Bookmarks
pub mod bookmarks;

/// XEP-0059: Result Set Management
pub mod rsm;

/// XEP-0060: Publish-Subscribe
pub mod pubsub;

/// XEP-0071: XHTML-IM
pub mod xhtml;

/// XEP-0077: In-Band Registration
pub mod ibr;

/// XEP-0082: XMPP Date and Time Profiles
pub mod date;

/// XEP-0084: User Avatar
pub mod avatar;

/// XEP-0085: Chat State Notifications
pub mod chatstates;

/// XEP-0092: Software Version
pub mod version;

/// XEP-0107: User Mood
pub mod mood;

/// XEP-0114: Jabber Component Protocol
pub mod component;

/// XEP-0115: Entity Capabilities
pub mod caps;

/// XEP-0118: User Tune
pub mod tune;

/// XEP-0157: Contact Addresses for XMPP Services
pub mod server_info;

/// XEP-0166: Jingle
pub mod jingle;

/// XEP-0167: Jingle RTP Sessions
pub mod jingle_rtp;

/// XEP-0172: User Nickname
pub mod nick;

/// XEP-0176: Jingle ICE-UDP Transport Method
pub mod jingle_ice_udp;

/// XEP-0177: Jingle Raw UDP Transport Method
pub mod jingle_raw_udp;

/// XEP-0184: Message Delivery Receipts
pub mod receipts;

/// XEP-0191: Blocking Command
pub mod blocking;

/// XEP-0198: Stream Management
pub mod sm;

/// XEP-0199: XMPP Ping
pub mod ping;

/// XEP-0202: Entity Time
pub mod time;

/// XEP-0203: Delayed Delivery
pub mod delay;

/// XEP-0221: Data Forms Media Element
pub mod media_element;

/// XEP-0224: Attention
pub mod attention;

/// XEP-0231: Bits of Binary
pub mod bob;

/// XEP-0234: Jingle File Transfer
pub mod jingle_ft;

/// XEP-0257: Client Certificate Management for SASL EXTERNAL
pub mod cert_management;

/// XEP-0260: Jingle SOCKS5 Bytestreams Transport Method
pub mod jingle_s5b;

/// XEP-0261: Jingle In-Band Bytestreams Transport Method
pub mod jingle_ibb;

/// XEP-0280: Message Carbons
pub mod carbons;

/// XEP-0293: Jingle RTP Feedback Negotiation
pub mod jingle_rtcp_fb;

/// XEP-0294: Jingle RTP Header Extensions Negociation
pub mod jingle_rtp_hdrext;

/// XEP-0297: Stanza Forwarding
pub mod forwarding;

/// XEP-0300: Use of Cryptographic Hash Functions in XMPP
pub mod hashes;

/// XEP-0308: Last Message Correction
pub mod message_correct;

/// XEP-0313: Message Archive Management
pub mod mam;

/// XEP-0319: Last User Interaction in Presence
pub mod idle;

/// XEP-0320: Use of DTLS-SRTP in Jingle Sessions
pub mod jingle_dtls_srtp;

/// XEP-0328: JID Prep
pub mod jid_prep;

/// XEP-0338: Jingle Grouping Framework
pub mod jingle_grouping;

/// XEP-0339: Source-Specific Media Attributes in Jingle
pub mod jingle_ssma;

/// XEP-0352: Client State Indication
pub mod csi;

/// XEP-0353: Jingle Message Initiation
pub mod jingle_message;

/// XEP-0359: Unique and Stable Stanza IDs
pub mod stanza_id;

/// XEP-0369: Mediated Information eXchange (MIX)
pub mod mix;

/// XEP-0373: OpenPGP for XMPP
pub mod openpgp;

/// XEP-0380: Explicit Message Encryption
pub mod eme;

/// XEP-0390: Entity Capabilities 2.0
pub mod ecaps2;

/// XEP-0402: Bookmarks 2 (This Time it's Serious)
pub mod bookmarks2;

/// XEP-0421: Anonymous unique occupant identifiers for MUCs
pub mod occupant_id;

/// XEP-0441: Message Archive Management Preferences
pub mod mam_prefs;
