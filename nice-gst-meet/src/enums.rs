// Generated by gir (https://github.com/gtk-rs/gir @ 5bbf6cb)
// from ../../gir-files (@ 8e47c67)
// DO NOT EDIT

use std::fmt;

use glib::translate::*;
use nice_sys as ffi;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
#[non_exhaustive]
#[doc(alias = "NiceCandidateTransport")]
pub enum CandidateTransport {
  #[doc(alias = "NICE_CANDIDATE_TRANSPORT_UDP")]
  Udp,
  #[doc(alias = "NICE_CANDIDATE_TRANSPORT_TCP_ACTIVE")]
  TcpActive,
  #[doc(alias = "NICE_CANDIDATE_TRANSPORT_TCP_PASSIVE")]
  TcpPassive,
  #[doc(alias = "NICE_CANDIDATE_TRANSPORT_TCP_SO")]
  TcpSo,
  #[doc(hidden)]
  __Unknown(i32),
}

impl fmt::Display for CandidateTransport {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "CandidateTransport::{}",
      match *self {
        Self::Udp => "Udp",
        Self::TcpActive => "TcpActive",
        Self::TcpPassive => "TcpPassive",
        Self::TcpSo => "TcpSo",
        _ => "Unknown",
      }
    )
  }
}

#[doc(hidden)]
impl IntoGlib for CandidateTransport {
  type GlibType = ffi::NiceCandidateTransport;

  fn into_glib(self) -> ffi::NiceCandidateTransport {
    match self {
      Self::Udp => ffi::NICE_CANDIDATE_TRANSPORT_UDP,
      Self::TcpActive => ffi::NICE_CANDIDATE_TRANSPORT_TCP_ACTIVE,
      Self::TcpPassive => ffi::NICE_CANDIDATE_TRANSPORT_TCP_PASSIVE,
      Self::TcpSo => ffi::NICE_CANDIDATE_TRANSPORT_TCP_SO,
      Self::__Unknown(value) => value,
    }
  }
}

#[doc(hidden)]
impl FromGlib<ffi::NiceCandidateTransport> for CandidateTransport {
  unsafe fn from_glib(value: ffi::NiceCandidateTransport) -> Self {
    match value {
      ffi::NICE_CANDIDATE_TRANSPORT_UDP => Self::Udp,
      ffi::NICE_CANDIDATE_TRANSPORT_TCP_ACTIVE => Self::TcpActive,
      ffi::NICE_CANDIDATE_TRANSPORT_TCP_PASSIVE => Self::TcpPassive,
      ffi::NICE_CANDIDATE_TRANSPORT_TCP_SO => Self::TcpSo,
      value => Self::__Unknown(value),
    }
  }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
#[non_exhaustive]
#[doc(alias = "NiceCandidateType")]
pub enum CandidateType {
  #[doc(alias = "NICE_CANDIDATE_TYPE_HOST")]
  Host,
  #[doc(alias = "NICE_CANDIDATE_TYPE_SERVER_REFLEXIVE")]
  ServerReflexive,
  #[doc(alias = "NICE_CANDIDATE_TYPE_PEER_REFLEXIVE")]
  PeerReflexive,
  #[doc(alias = "NICE_CANDIDATE_TYPE_RELAYED")]
  Relayed,
  #[doc(hidden)]
  __Unknown(i32),
}

impl fmt::Display for CandidateType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "CandidateType::{}",
      match *self {
        Self::Host => "Host",
        Self::ServerReflexive => "ServerReflexive",
        Self::PeerReflexive => "PeerReflexive",
        Self::Relayed => "Relayed",
        _ => "Unknown",
      }
    )
  }
}

#[doc(hidden)]
impl IntoGlib for CandidateType {
  type GlibType = ffi::NiceCandidateType;

  fn into_glib(self) -> ffi::NiceCandidateType {
    match self {
      Self::Host => ffi::NICE_CANDIDATE_TYPE_HOST,
      Self::ServerReflexive => ffi::NICE_CANDIDATE_TYPE_SERVER_REFLEXIVE,
      Self::PeerReflexive => ffi::NICE_CANDIDATE_TYPE_PEER_REFLEXIVE,
      Self::Relayed => ffi::NICE_CANDIDATE_TYPE_RELAYED,
      Self::__Unknown(value) => value,
    }
  }
}

#[doc(hidden)]
impl FromGlib<ffi::NiceCandidateType> for CandidateType {
  unsafe fn from_glib(value: ffi::NiceCandidateType) -> Self {
    match value {
      ffi::NICE_CANDIDATE_TYPE_HOST => Self::Host,
      ffi::NICE_CANDIDATE_TYPE_SERVER_REFLEXIVE => Self::ServerReflexive,
      ffi::NICE_CANDIDATE_TYPE_PEER_REFLEXIVE => Self::PeerReflexive,
      ffi::NICE_CANDIDATE_TYPE_RELAYED => Self::Relayed,
      value => Self::__Unknown(value),
    }
  }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
#[non_exhaustive]
#[doc(alias = "NiceCompatibility")]
pub enum Compatibility {
  #[doc(alias = "NICE_COMPATIBILITY_RFC5245")]
  Rfc5245,
  #[doc(alias = "NICE_COMPATIBILITY_GOOGLE")]
  Google,
  #[doc(alias = "NICE_COMPATIBILITY_MSN")]
  Msn,
  #[doc(alias = "NICE_COMPATIBILITY_WLM2009")]
  Wlm2009,
  #[doc(alias = "NICE_COMPATIBILITY_OC2007")]
  Oc2007,
  #[doc(alias = "NICE_COMPATIBILITY_OC2007R2")]
  Oc2007r2,
  #[doc(hidden)]
  __Unknown(i32),
}

impl fmt::Display for Compatibility {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "Compatibility::{}",
      match *self {
        Self::Rfc5245 => "Rfc5245",
        Self::Google => "Google",
        Self::Msn => "Msn",
        Self::Wlm2009 => "Wlm2009",
        Self::Oc2007 => "Oc2007",
        Self::Oc2007r2 => "Oc2007r2",
        _ => "Unknown",
      }
    )
  }
}

#[doc(hidden)]
impl IntoGlib for Compatibility {
  type GlibType = ffi::NiceCompatibility;

  fn into_glib(self) -> ffi::NiceCompatibility {
    match self {
      Self::Rfc5245 => ffi::NICE_COMPATIBILITY_RFC5245,
      Self::Google => ffi::NICE_COMPATIBILITY_GOOGLE,
      Self::Msn => ffi::NICE_COMPATIBILITY_MSN,
      Self::Wlm2009 => ffi::NICE_COMPATIBILITY_WLM2009,
      Self::Oc2007 => ffi::NICE_COMPATIBILITY_OC2007,
      Self::Oc2007r2 => ffi::NICE_COMPATIBILITY_OC2007R2,
      Self::__Unknown(value) => value,
    }
  }
}

#[doc(hidden)]
impl FromGlib<ffi::NiceCompatibility> for Compatibility {
  unsafe fn from_glib(value: ffi::NiceCompatibility) -> Self {
    match value {
      ffi::NICE_COMPATIBILITY_RFC5245 => Self::Rfc5245,
      ffi::NICE_COMPATIBILITY_GOOGLE => Self::Google,
      ffi::NICE_COMPATIBILITY_MSN => Self::Msn,
      ffi::NICE_COMPATIBILITY_WLM2009 => Self::Wlm2009,
      ffi::NICE_COMPATIBILITY_OC2007 => Self::Oc2007,
      ffi::NICE_COMPATIBILITY_OC2007R2 => Self::Oc2007r2,
      value => Self::__Unknown(value),
    }
  }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
#[non_exhaustive]
#[doc(alias = "NiceComponentState")]
pub enum ComponentState {
  #[doc(alias = "NICE_COMPONENT_STATE_DISCONNECTED")]
  Disconnected,
  #[doc(alias = "NICE_COMPONENT_STATE_GATHERING")]
  Gathering,
  #[doc(alias = "NICE_COMPONENT_STATE_CONNECTING")]
  Connecting,
  #[doc(alias = "NICE_COMPONENT_STATE_CONNECTED")]
  Connected,
  #[doc(alias = "NICE_COMPONENT_STATE_READY")]
  Ready,
  #[doc(alias = "NICE_COMPONENT_STATE_FAILED")]
  Failed,
  #[doc(alias = "NICE_COMPONENT_STATE_LAST")]
  Last,
  #[doc(hidden)]
  __Unknown(i32),
}

impl fmt::Display for ComponentState {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "ComponentState::{}",
      match *self {
        Self::Disconnected => "Disconnected",
        Self::Gathering => "Gathering",
        Self::Connecting => "Connecting",
        Self::Connected => "Connected",
        Self::Ready => "Ready",
        Self::Failed => "Failed",
        Self::Last => "Last",
        _ => "Unknown",
      }
    )
  }
}

#[doc(hidden)]
impl IntoGlib for ComponentState {
  type GlibType = ffi::NiceComponentState;

  fn into_glib(self) -> ffi::NiceComponentState {
    match self {
      Self::Disconnected => ffi::NICE_COMPONENT_STATE_DISCONNECTED,
      Self::Gathering => ffi::NICE_COMPONENT_STATE_GATHERING,
      Self::Connecting => ffi::NICE_COMPONENT_STATE_CONNECTING,
      Self::Connected => ffi::NICE_COMPONENT_STATE_CONNECTED,
      Self::Ready => ffi::NICE_COMPONENT_STATE_READY,
      Self::Failed => ffi::NICE_COMPONENT_STATE_FAILED,
      Self::Last => ffi::NICE_COMPONENT_STATE_LAST,
      Self::__Unknown(value) => value,
    }
  }
}

#[doc(hidden)]
impl FromGlib<ffi::NiceComponentState> for ComponentState {
  unsafe fn from_glib(value: ffi::NiceComponentState) -> Self {
    match value {
      ffi::NICE_COMPONENT_STATE_DISCONNECTED => Self::Disconnected,
      ffi::NICE_COMPONENT_STATE_GATHERING => Self::Gathering,
      ffi::NICE_COMPONENT_STATE_CONNECTING => Self::Connecting,
      ffi::NICE_COMPONENT_STATE_CONNECTED => Self::Connected,
      ffi::NICE_COMPONENT_STATE_READY => Self::Ready,
      ffi::NICE_COMPONENT_STATE_FAILED => Self::Failed,
      ffi::NICE_COMPONENT_STATE_LAST => Self::Last,
      value => Self::__Unknown(value),
    }
  }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
#[non_exhaustive]
#[doc(alias = "NiceRelayType")]
pub enum RelayType {
  #[doc(alias = "NICE_RELAY_TYPE_TURN_UDP")]
  Udp,
  #[doc(alias = "NICE_RELAY_TYPE_TURN_TCP")]
  Tcp,
  #[doc(alias = "NICE_RELAY_TYPE_TURN_TLS")]
  Tls,
  #[doc(hidden)]
  __Unknown(i32),
}

impl fmt::Display for RelayType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "RelayType::{}",
      match *self {
        Self::Udp => "Udp",
        Self::Tcp => "Tcp",
        Self::Tls => "Tls",
        _ => "Unknown",
      }
    )
  }
}

#[doc(hidden)]
impl IntoGlib for RelayType {
  type GlibType = ffi::NiceRelayType;

  fn into_glib(self) -> ffi::NiceRelayType {
    match self {
      Self::Udp => ffi::NICE_RELAY_TYPE_TURN_UDP,
      Self::Tcp => ffi::NICE_RELAY_TYPE_TURN_TCP,
      Self::Tls => ffi::NICE_RELAY_TYPE_TURN_TLS,
      Self::__Unknown(value) => value,
    }
  }
}

#[doc(hidden)]
impl FromGlib<ffi::NiceRelayType> for RelayType {
  unsafe fn from_glib(value: ffi::NiceRelayType) -> Self {
    match value {
      ffi::NICE_RELAY_TYPE_TURN_UDP => Self::Udp,
      ffi::NICE_RELAY_TYPE_TURN_TCP => Self::Tcp,
      ffi::NICE_RELAY_TYPE_TURN_TLS => Self::Tls,
      value => Self::__Unknown(value),
    }
  }
}
