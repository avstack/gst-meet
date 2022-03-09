use std::{collections::HashMap, fmt, net::SocketAddr};

use anyhow::{anyhow, bail, Context, Result};
use futures::stream::StreamExt;
use glib::{Cast, ObjectExt, ToValue};
use gstreamer::{
  prelude::{ElementExt, ElementExtManual, GObjectExtManualGst, GstBinExt, GstObjectExt, PadExt},
  Bin, Element, GhostPad,
};
#[cfg(feature = "log-rtp")]
use gstreamer_rtp::RTPBuffer;
use gstreamer_rtp::{prelude::RTPHeaderExtensionExt, RTPHeaderExtension};
use itertools::Itertools;
use jitsi_xmpp_parsers::{
  jingle::{Action, Content, Description, Jingle, Transport},
  jingle_dtls_srtp::Fingerprint,
  jingle_ice_udp::Transport as IceUdpTransport,
  jingle_rtp::Description as RtpDescription,
  jingle_ssma,
};
use nice_gst_meet as nice;
use pem::Pem;
use rand::random;
use rcgen::{Certificate, CertificateParams, PKCS_ECDSA_P256_SHA256};
use ring::digest::{digest, SHA256};
use tokio::{net::lookup_host, runtime::Handle, sync::oneshot, task::JoinHandle};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use xmpp_parsers::{
  hashes::Algo,
  iq::Iq,
  jingle::{Creator, Senders},
  jingle_dtls_srtp::Setup,
  jingle_grouping::{self, Content as GroupContent},
  jingle_ice_udp,
  jingle_rtcp_fb::RtcpFb,
  jingle_rtp::{self, PayloadType, RtcpMux},
  jingle_rtp_hdrext::RtpHdrext,
  jingle_ssma::{Parameter, Semantics},
  Jid,
};

use crate::{
  colibri::ColibriChannel,
  conference::JitsiConference,
  source::{MediaType, Source},
  util::generate_id,
};

const RTP_HDREXT_SSRC_AUDIO_LEVEL: &str = "urn:ietf:params:rtp-hdrext:ssrc-audio-level";
const RTP_HDREXT_TRANSPORT_CC: &str =
  "http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01";

const DEFAULT_STUN_PORT: u16 = 3478;
const DEFAULT_TURNS_PORT: u16 = 5349;

#[derive(Clone, PartialEq)]
enum CodecName {
  Opus,
  H264,
  Vp8,
  Vp9,
}

#[derive(Clone)]
struct Codec {
  name: CodecName,
  pt: u8,
  rtx_pt: Option<u8>,
  rtcp_fbs: Vec<RtcpFb>,
}

impl Codec {
  fn is(&self, pt: u8) -> bool {
    self.pt == pt
  }

  fn is_rtx(&self, rtx_pt: u8) -> bool {
    if let Some(pt) = self.rtx_pt {
      pt == rtx_pt
    }
    else {
      false
    }
  }

  fn is_audio(&self) -> bool {
    self.name == CodecName::Opus
  }

  fn is_video(&self) -> bool {
    self.name != CodecName::Opus
  }

  fn is_codec(&self, name: &str) -> bool {
    match name {
      "h264" => self.name == CodecName::H264,
      "vp8" => self.name == CodecName::Vp8,
      "vp9" => self.name == CodecName::Vp9,
      _ => false,
    }
  }

  fn encoding_name(&self) -> &'static str {
    match self.name {
      CodecName::Opus => "opus",
      CodecName::H264 => "H264",
      CodecName::Vp8 => "VP8",
      CodecName::Vp9 => "VP9",
    }
  }

  fn depayloader_name(&self) -> &'static str {
    match self.name {
      CodecName::Opus => "rtpopusdepay",
      CodecName::H264 => "rtph264depay",
      CodecName::Vp8 => "rtpvp8depay",
      CodecName::Vp9 => "rtpvp9depay",
    }
  }

  fn decoder_name(&self) -> &'static str {
    match self.name {
      CodecName::Opus => "opusdec",
      CodecName::H264 => "avdec_h264",
      CodecName::Vp8 => "vp8dec",
      CodecName::Vp9 => "vp9dec",
    }
  }

  fn payloader_name(&self) -> &'static str {
    match self.name {
      CodecName::Opus => "rtpopuspay",
      CodecName::H264 => "rtph264pay",
      CodecName::Vp8 => "rtpvp8pay",
      CodecName::Vp9 => "rtpvp9pay",
    }
  }
}

struct ParsedRtpDescription {
  codecs: Vec<Codec>,
  audio_hdrext_ssrc_audio_level: Option<u16>,
  audio_hdrext_transport_cc: Option<u16>,
  video_hdrext_transport_cc: Option<u16>,
}

pub(crate) struct JingleSession {
  pipeline: gstreamer::Pipeline,
  audio_sink_element: gstreamer::Element,
  video_sink_element: gstreamer::Element,
  pub(crate) remote_ssrc_map: HashMap<u32, Source>,
  _ice_agent: nice::Agent,
  pub(crate) accept_iq_id: Option<String>,
  pub(crate) colibri_url: Option<String>,
  pub(crate) colibri_channel: Option<ColibriChannel>,
  pub(crate) stats_handler_task: Option<JoinHandle<()>>,
  pipeline_state_null_rx: oneshot::Receiver<()>,
}

impl fmt::Debug for JingleSession {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("JingleSession").finish()
  }
}

impl JingleSession {
  pub(crate) fn pipeline(&self) -> gstreamer::Pipeline {
    self.pipeline.clone()
  }

  pub(crate) fn audio_sink_element(&self) -> gstreamer::Element {
    self.audio_sink_element.clone()
  }

  pub(crate) fn video_sink_element(&self) -> gstreamer::Element {
    self.video_sink_element.clone()
  }

  pub(crate) fn pause_all_sinks(&self) {
    if let Some(rtpbin) = self.pipeline.by_name("rtpbin") {
      rtpbin.foreach_src_pad(|_, pad| {
        let pad_name: String = pad.property("name");
        if pad_name.starts_with("recv_rtp_src_0_") {
          if let Some(peer_pad) = pad.peer() {
            if let Some(element) = peer_pad.parent_element() {
              element.set_state(gstreamer::State::Paused).unwrap();
            }
          }
        }
        true
      });
    }
  }

  pub(crate) async fn pipeline_stopped(self) -> Result<()> {
    Ok(self.pipeline_state_null_rx.await?)
  }

  fn parse_rtp_description(
    description: &RtpDescription,
    remote_ssrc_map: &mut HashMap<u32, Source>,
  ) -> Result<Option<ParsedRtpDescription>> {
    let mut opus = None;
    let mut h264 = None;
    let mut vp8 = None;
    let mut vp9 = None;
    let mut audio_hdrext_ssrc_audio_level = None;
    let mut audio_hdrext_transport_cc = None;
    let mut video_hdrext_transport_cc = None;

    if description.media == "audio" {
      for pt in description.payload_types.iter() {
        // We don’t support any static codec, so name MUST be set.
        if pt.name.as_deref() == Some("opus") {
          opus = Some(Codec {
            name: CodecName::Opus,
            pt: pt.id,
            rtx_pt: None,
            rtcp_fbs: pt.rtcp_fbs.clone(),
          });
        }
      }
      for hdrext in description.hdrexts.iter() {
        if hdrext.uri == RTP_HDREXT_SSRC_AUDIO_LEVEL {
          audio_hdrext_ssrc_audio_level = Some(hdrext.id);
        }
        else if hdrext.uri == RTP_HDREXT_TRANSPORT_CC {
          audio_hdrext_transport_cc = Some(hdrext.id);
        }
      }
    }
    else if description.media == "video" {
      for pt in description.payload_types.iter() {
        // We don’t support any static codec, so name MUST be set.
        if let Some(name) = &pt.name {
          match name.as_str() {
            "H264" => {
              h264 = Some(Codec {
                name: CodecName::H264,
                pt: pt.id,
                rtx_pt: None,
                rtcp_fbs: pt.rtcp_fbs.clone(),
              });
            },
            "VP8" => {
              vp8 = Some(Codec {
                name: CodecName::Vp8,
                pt: pt.id,
                rtx_pt: None,
                rtcp_fbs: pt.rtcp_fbs.clone(),
              });
            },
            "VP9" => {
              vp9 = Some(Codec {
                name: CodecName::Vp9,
                pt: pt.id,
                rtx_pt: None,
                rtcp_fbs: pt.rtcp_fbs.clone(),
              });
            },
            _ => (),
          }
        }
      }
      for pt in description.payload_types.iter() {
        if let Some(name) = &pt.name {
          if name == "rtx" {
            for param in pt.parameters.iter() {
              if param.name == "apt" {
                let apt_pt: u8 = param.value.parse()?;
                if let Some(h264) = &mut h264 {
                  if apt_pt == h264.pt {
                    h264.rtx_pt = Some(pt.id);
                  }
                }
                if let Some(vp8) = &mut vp8 {
                  if apt_pt == vp8.pt {
                    vp8.rtx_pt = Some(pt.id);
                  }
                }
                if let Some(vp9) = &mut vp9 {
                  if apt_pt == vp9.pt {
                    vp9.rtx_pt = Some(pt.id);
                  }
                }
              }
            }
          }
        }
      }
      for hdrext in description.hdrexts.iter() {
        if hdrext.uri == RTP_HDREXT_TRANSPORT_CC {
          video_hdrext_transport_cc = Some(hdrext.id);
        }
      }
    }
    else {
      debug!("skipping media: {}", description.media);
      return Ok(None);
    }

    let codecs = [opus, h264, vp8, vp9].iter().flatten().cloned().collect();

    for ssrc in &description.ssrcs {
      let owner = ssrc
        .info
        .as_ref()
        .context("missing ssrc-info")?
        .owner
        .clone();

      debug!("adding ssrc to remote_ssrc_map: {:?}", ssrc);
      remote_ssrc_map.insert(
        ssrc.id,
        Source {
          ssrc: ssrc.id,
          participant_id: if owner == "jvb" {
            None
          }
          else {
            Some(
              owner
                .split('/')
                .nth(1)
                .context("invalid ssrc-info owner")?
                .to_owned(),
            )
          },
          media_type: if description.media == "audio" {
            MediaType::Audio
          }
          else {
            MediaType::Video
          },
        },
      );
    }
    Ok(Some(ParsedRtpDescription {
      codecs,
      audio_hdrext_ssrc_audio_level,
      audio_hdrext_transport_cc,
      video_hdrext_transport_cc,
    }))
  }

  async fn setup_ice(
    conference: &JitsiConference,
    transport: &IceUdpTransport,
  ) -> Result<(nice::Agent, u32, u32)> {
    let ice_agent = nice::Agent::new(&conference.glib_main_context, nice::Compatibility::Rfc5245);
    ice_agent.set_ice_tcp(false);
    ice_agent.set_upnp(false);
    let ice_stream_id = ice_agent.add_stream(1);
    let ice_component_id = 1;

    let maybe_stun = conference
      .external_services
      .iter()
      .find(|svc| svc.r#type == "stun");

    let stun_addr = if let Some(stun) = maybe_stun {
      lookup_host(format!(
        "{}:{}",
        stun.host,
        stun.port.unwrap_or(DEFAULT_STUN_PORT)
      ))
      .await?
      .next()
    }
    else {
      None
    };
    debug!("STUN address: {:?}", stun_addr);

    if let Some((stun_addr, stun_port)) = stun_addr.map(|sa| (sa.ip().to_string(), sa.port())) {
      ice_agent.set_stun_server(Some(&stun_addr));
      ice_agent.set_stun_server_port(stun_port as u32);
    }

    let maybe_turn = conference
      .external_services
      .iter()
      .find(|svc| svc.r#type == "turns");

    if let Some(turn_server) = maybe_turn {
      let maybe_addr = lookup_host(format!(
        "{}:{}",
        turn_server.host,
        turn_server.port.unwrap_or(DEFAULT_TURNS_PORT)
      ))
      .await?
      .next();

      if let Some(addr) = maybe_addr {
        debug!("TURN address: {:?}", addr);
        ice_agent.set_relay_info(
          ice_stream_id,
          ice_component_id,
          &addr.ip().to_string(),
          addr.port() as u32,
          turn_server.username.as_deref().unwrap_or_default(),
          turn_server.password.as_deref().unwrap_or_default(),
          nice::RelayType::Tls,
        );
      }
    }

    if !ice_agent.attach_recv(
      ice_stream_id,
      ice_component_id,
      &conference.glib_main_context,
      |_, _, _, s| debug!("ICE nice_agent_attach_recv cb: {}", s),
    ) {
      warn!("nice_agent_attach_recv failed");
    }

    debug!("ice_agent={:?}", ice_agent);
    debug!("ice_stream_id={}", ice_stream_id);
    debug!("ice_component_id={}", ice_component_id);

    if let (Some(ufrag), Some(pwd)) = (&transport.ufrag, &transport.pwd) {
      debug!("setting ICE remote credentials");
      if !ice_agent.set_remote_credentials(ice_stream_id, ufrag, pwd) {
        warn!("nice_agent_set_remote_candidates failed");
      }
    }

    ice_agent.connect_candidate_gathering_done(move |_agent, candidates| {
      debug!("ICE candidate-gathering-done {:?}", candidates);
    });

    debug!("gathering ICE candidates");
    if !ice_agent.gather_candidates(ice_stream_id) {
      warn!("nice_agent_gather_candidates failed");
    }

    if let (Some(ufrag), Some(pwd), remote_candidates) =
      (&transport.ufrag, &transport.pwd, &transport.candidates)
    {
      debug!("setting ICE remote candidates: {:?}", remote_candidates);
      let remote_candidates: Vec<_> = remote_candidates
        .iter()
        .map(|c| {
          let mut candidate = nice::Candidate::new(match c.type_ {
            jingle_ice_udp::Type::Host => nice::CandidateType::Host,
            jingle_ice_udp::Type::Prflx => nice::CandidateType::PeerReflexive,
            jingle_ice_udp::Type::Srflx => nice::CandidateType::ServerReflexive,
            jingle_ice_udp::Type::Relay => nice::CandidateType::Relayed,
          });
          candidate.set_stream_id(ice_stream_id);
          candidate.set_component_id(c.component as u32);
          candidate.set_foundation(&c.foundation);
          candidate.set_addr(SocketAddr::new(c.ip, c.port));
          candidate.set_priority(c.priority);
          candidate.set_username(ufrag);
          candidate.set_password(pwd);
          debug!("candidate: {:?}", candidate);
          candidate
        })
        .collect();
      let candidate_refs: Vec<_> = remote_candidates.iter().collect();
      let res = ice_agent.set_remote_candidates(ice_stream_id, ice_component_id, &candidate_refs);
      if res < remote_candidates.len() as i32 {
        warn!("some remote candidates failed to add: {}", res);
      }
    }

    Ok((ice_agent, ice_stream_id, ice_component_id))
  }

  pub(crate) async fn initiate(conference: &JitsiConference, jingle: Jingle) -> Result<Self> {
    let initiator = jingle
      .initiator
      .as_ref()
      .ok_or_else(|| anyhow!("session-initiate with no initiator"))?
      .clone();

    debug!("Received Jingle session-initiate from {}", initiator);

    let mut ice_transport = None;
    let mut codecs = vec![];
    let mut audio_hdrext_ssrc_audio_level = None;
    let mut audio_hdrext_transport_cc = None;
    let mut video_hdrext_transport_cc = None;

    let mut remote_ssrc_map = HashMap::new();

    for content in &jingle.contents {
      if let Some(Description::Rtp(description)) = &content.description {
        if let Some(description) =
          JingleSession::parse_rtp_description(description, &mut remote_ssrc_map)?
        {
          codecs.extend(description.codecs);
          audio_hdrext_ssrc_audio_level =
            audio_hdrext_ssrc_audio_level.or(description.audio_hdrext_ssrc_audio_level);
          audio_hdrext_transport_cc =
            audio_hdrext_transport_cc.or(description.audio_hdrext_transport_cc);
          video_hdrext_transport_cc =
            video_hdrext_transport_cc.or(description.video_hdrext_transport_cc);
        }
      }

      if let Some(Transport::IceUdp(transport)) = &content.transport {
        if let Some(fingerprint) = &transport.fingerprint {
          if fingerprint.hash != Algo::Sha_256 {
            bail!("unsupported fingerprint hash: {:?}", fingerprint.hash);
          }
        }
        ice_transport = Some(transport);
      }
    }

    let ice_transport = ice_transport.context("missing ICE transport")?;

    if let Some(remote_fingerprint) = &ice_transport.fingerprint {
      warn!(
        "Remote DTLS fingerprint (verification not implemented yet): {:?}",
        remote_fingerprint
      );
    }

    let mut dtls_cert_params = CertificateParams::new(vec!["gst-meet".to_owned()]);
    dtls_cert_params.alg = &PKCS_ECDSA_P256_SHA256;
    let dtls_cert = Certificate::from_params(dtls_cert_params)?;
    let dtls_cert_der = dtls_cert.serialize_der()?;
    let fingerprint = digest(&SHA256, &dtls_cert_der).as_ref().to_vec();
    let fingerprint_str =
      itertools::join(fingerprint.iter().map(|byte| format!("{:X}", byte)), ":");
    let dtls_cert_pem = pem::encode(&Pem {
      tag: "CERTIFICATE".to_string(),
      contents: dtls_cert_der,
    });
    let dtls_private_key_pem = pem::encode(&Pem {
      tag: "PRIVATE KEY".to_string(),
      contents: dtls_cert.serialize_private_key_der(),
    });
    debug!("Local DTLS certificate:\n{}", dtls_cert_pem);
    debug!("Local DTLS fingerprint: {}", fingerprint_str);

    let audio_ssrc: u32 = random();
    let video_ssrc: u32 = random();
    let video_rtx_ssrc: u32 = random();

    debug!("audio SSRC: {}", audio_ssrc);
    debug!("video SSRC: {}", video_ssrc);
    debug!("video RTX SSRC: {}", video_rtx_ssrc);

    let (ice_agent, ice_stream_id, ice_component_id) =
      JingleSession::setup_ice(conference, ice_transport).await?;

    let (ice_local_ufrag, ice_local_pwd) = ice_agent
      .local_credentials(ice_stream_id)
      .context("no local ICE credentials")?;

    debug!("building gstreamer pipeline");

    let pipeline = gstreamer::Pipeline::new(None);

    let rtpbin = gstreamer::ElementFactory::make("rtpbin", Some("rtpbin"))?;
    rtpbin.set_property_from_str("rtp-profile", "savpf");
    rtpbin.set_property("autoremove", true);
    rtpbin.set_property("do-lost", true);
    rtpbin.set_property("do-sync-event", true);
    pipeline.add(&rtpbin)?;

    let nicesrc = gstreamer::ElementFactory::make("nicesrc", None)?;
    nicesrc.set_property("stream", ice_stream_id);
    nicesrc.set_property("component", ice_component_id);
    nicesrc.set_property("agent", &ice_agent);
    pipeline.add(&nicesrc)?;

    let nicesink = gstreamer::ElementFactory::make("nicesink", None)?;
    nicesink.set_property("stream", ice_stream_id);
    nicesink.set_property("component", ice_component_id);
    nicesink.set_property("agent", &ice_agent);
    pipeline.add(&nicesink)?;

    let dtls_srtp_connection_id = "gst-meet";

    let dtlssrtpenc = gstreamer::ElementFactory::make("dtlssrtpenc", None)?;
    dtlssrtpenc.set_property("connection-id", dtls_srtp_connection_id);
    dtlssrtpenc.set_property("is-client", true);
    pipeline.add(&dtlssrtpenc)?;

    let dtlssrtpdec = gstreamer::ElementFactory::make("dtlssrtpdec", None)?;
    dtlssrtpdec.set_property("connection-id", dtls_srtp_connection_id);
    dtlssrtpdec.set_property(
      "pem",
      format!("{}\n{}", dtls_cert_pem, dtls_private_key_pem),
    );
    pipeline.add(&dtlssrtpdec)?;

    {
      let codecs = codecs.clone();
      rtpbin.connect("request-pt-map", false, move |values| {
        let f = || {
          debug!("rtpbin request-pt-map {:?}", values);
          let pt = values[2].get::<u32>()? as u8;
          let mut caps = gstreamer::Caps::builder("application/x-rtp").field("payload", pt as i32);
          for codec in codecs.iter() {
            if codec.is(pt) {
              if codec.is_audio() {
                caps = caps
                  .field("media", "audio")
                  .field("encoding-name", "OPUS")
                  .field("clock-rate", 48000);
                if let Some(hdrext) = audio_hdrext_ssrc_audio_level {
                  caps = caps.field(&format!("extmap-{}", hdrext), RTP_HDREXT_SSRC_AUDIO_LEVEL);
                }
                if let Some(hdrext) = audio_hdrext_transport_cc {
                  caps = caps.field(&format!("extmap-{}", hdrext), RTP_HDREXT_TRANSPORT_CC);
                }
              }
              else {
                // A video codec, as the only audio codec we support is Opus.
                caps = caps
                  .field("media", "video")
                  .field("clock-rate", 90000)
                  .field("encoding-name", codec.encoding_name())
                  .field("rtcp-fb-nack-pli", true);
                if let Some(hdrext) = video_hdrext_transport_cc {
                  caps = caps.field(&format!("extmap-{}", hdrext), RTP_HDREXT_TRANSPORT_CC);
                }
              }
              return Ok::<_, anyhow::Error>(Some(caps.build()));
            }
            else if codec.is_rtx(pt) {
              caps = caps
                .field("media", "video")
                .field("clock-rate", 90000)
                .field("encoding-name", "RTX")
                .field("apt", codec.pt);
              return Ok(Some(caps.build()));
            }
          }

          warn!("unknown payload type: {}", pt);
          Ok(None)
        };
        match f() {
          Ok(Some(caps)) => {
            debug!("mapped pt to caps: {:?}", caps);
            Some(caps.to_value())
          },
          Ok(None) => None,
          Err(e) => {
            error!("handling request-pt-map: {:?}", e);
            None
          },
        }
      });
    }

    let handle = Handle::current();
    let jingle_session = conference.jingle_session.clone();
    let buffer_size = conference.config.buffer_size;
    rtpbin.connect("new-jitterbuffer", false, move |values| {
      let handle = handle.clone();
      let jingle_session = jingle_session.clone();
      let f = move || {
        let rtpjitterbuffer: gstreamer::Element = values[1].get()?;
        let session: u32 = values[2].get()?;
        let ssrc: u32 = values[3].get()?;
        debug!(
          "new jitterbuffer created for session {} ssrc {}",
          session, ssrc
        );

        let source = handle.block_on(async move {
          Ok::<_, anyhow::Error>(
            jingle_session
              .lock()
              .await
              .as_ref()
              .context("not connected (no jingle session)")?
              .remote_ssrc_map
              .get(&ssrc)
              .context(format!("unknown ssrc: {}", ssrc))?
              .clone(),
          )
        })?;
        debug!("jitterbuffer is for remote source: {:?}", source);
        if source.media_type == MediaType::Video && source.participant_id.is_some() {
          debug!("enabling RTX for ssrc {}", ssrc);
          rtpjitterbuffer.set_property("do-retransmission", true);
          rtpjitterbuffer.set_property("drop-on-latency", true);
          rtpjitterbuffer.set_property("latency", buffer_size);
        }
        Ok::<_, anyhow::Error>(())
      };
      if let Err(e) = f() {
        warn!("new-jitterbuffer: {:?}", e);
      }
      None
    });

    let pts: Vec<(String, u32)> = codecs
      .iter()
      .filter(|codec| codec.is_video())
      .flat_map(|codec| {
        codec
          .rtx_pt
          .map(|rtx_pt| (codec.pt.to_string(), rtx_pt as u32))
      })
      .collect();
    {
      let pts = pts.clone();
      rtpbin.connect("request-aux-sender", false, move |values| {
        let f = || {
          let session: u32 = values[1].get()?;
          debug!("creating RTX sender for session {}", session);
          let mut pt_map = gstreamer::Structure::builder("application/x-rtp-pt-map");
          let mut ssrc_map = gstreamer::Structure::builder("application/x-rtp-ssrc-map");
          for (pt, rtx_pt) in pts.iter() {
            pt_map = pt_map.field(pt, rtx_pt);
          }
          ssrc_map = ssrc_map.field(&video_ssrc.to_string(), &(video_rtx_ssrc as u32));
          let bin = gstreamer::Bin::new(None);
          let rtx_sender = gstreamer::ElementFactory::make("rtprtxsend", None)?;
          rtx_sender.set_property("payload-type-map", pt_map.build());
          rtx_sender.set_property("ssrc-map", ssrc_map.build());
          bin.add(&rtx_sender)?;
          bin.add_pad(&gstreamer::GhostPad::with_target(
            Some(&format!("src_{}", session)),
            &rtx_sender
              .static_pad("src")
              .context("rtprtxsend has no src pad")?,
          )?)?;
          bin.add_pad(&gstreamer::GhostPad::with_target(
            Some(&format!("sink_{}", session)),
            &rtx_sender
              .static_pad("sink")
              .context("rtprtxsend has no sink pad")?,
          )?)?;
          Ok::<_, anyhow::Error>(Some(bin.to_value()))
        };
        match f() {
          Ok(o) => o,
          Err(e) => {
            warn!("request-aux-sender: {:?}", e);
            None
          },
        }
      });
    }

    rtpbin.connect("request-aux-receiver", false, move |values| {
      let f = || {
        let session: u32 = values[1].get()?;
        debug!("creating RTX receiver for session {}", session);
        let mut pt_map = gstreamer::Structure::builder("application/x-rtp-pt-map");
        for (pt, rtx_pt) in pts.iter() {
          pt_map = pt_map.field(pt, rtx_pt);
        }
        let bin = gstreamer::Bin::new(None);
        let rtx_receiver = gstreamer::ElementFactory::make("rtprtxreceive", None)?;
        rtx_receiver.set_property("payload-type-map", pt_map.build());
        bin.add(&rtx_receiver)?;
        bin.add_pad(&gstreamer::GhostPad::with_target(
          Some(&format!("src_{}", session)),
          &rtx_receiver
            .static_pad("src")
            .context("rtprtxreceive has no src pad")?,
        )?)?;
        bin.add_pad(&gstreamer::GhostPad::with_target(
          Some(&format!("sink_{}", session)),
          &rtx_receiver
            .static_pad("sink")
            .context("rtprtxreceive has no sink pad")?,
        )?)?;
        Ok::<_, anyhow::Error>(Some(bin.to_value()))
      };
      match f() {
        Ok(o) => o,
        Err(e) => {
          warn!("request-aux-receiver: {:?}", e);
          None
        },
      }
    });

    {
      let handle = Handle::current();
      let conference = conference.clone();
      let pipeline = pipeline.clone();
      let rtpbin_ = rtpbin.clone();
      let codecs = codecs.clone();
      rtpbin.connect("pad-added", false, move |values| {
        let rtpbin = &rtpbin_;
        let f = || {
          debug!("rtpbin pad-added {:?}", values);
          let pad: gstreamer::Pad = values[1].get()?;
          let pad_name: String = pad.property("name");
          if pad_name.starts_with("recv_rtp_src_0_") {
            let mut parts = pad_name.split('_').skip(4);
            let ssrc: u32 = parts.next().context("malformed pad name")?.parse()?;
            let pt: u8 = parts.next().context("malformed pad name")?.parse()?;
            let source = handle.block_on(async {
              Ok::<_, anyhow::Error>(
                conference
                  .jingle_session
                  .lock()
                  .await
                  .as_ref()
                  .context("not connected (no jingle session)")?
                  .remote_ssrc_map
                  .get(&ssrc)
                  .context(format!("unknown ssrc: {}", ssrc))?
                  .clone(),
              )
            })?;

            debug!("pad added for remote source: {:?}", source);

            let depayloader = match source.media_type {
              MediaType::Audio => {
                let codec = codecs
                  .iter()
                  .filter(|codec| codec.is_audio())
                  .find(|codec| codec.is(pt));
                if let Some(codec) = codec {
                  gstreamer::ElementFactory::make(codec.depayloader_name(), None)?
                }
                else {
                  bail!("received audio with unsupported PT {}", pt);
                }
              },
              MediaType::Video => {
                let codec = codecs
                  .iter()
                  .filter(|codec| codec.is_video())
                  .find(|codec| codec.is(pt));
                if let Some(codec) = codec {
                  gstreamer::ElementFactory::make(codec.depayloader_name(), None)?
                }
                else {
                  bail!("received video with unsupported PT {}", pt);
                }
              },
            };

            depayloader.set_property("auto-header-extension", false);
            depayloader.connect("request-extension", false, move |values| {
              let f = || {
                let ext_id: u32 = values[1].get()?;
                let ext_uri: String = values[2].get()?;
                debug!("depayloader requested extension: {} {}", ext_id, ext_uri);
                let hdrext = RTPHeaderExtension::create_from_uri(&ext_uri)
                  .context("failed to create hdrext")?;
                hdrext.set_id(ext_id);
                Ok::<_, anyhow::Error>(hdrext)
              };
              match f() {
                Ok(hdrext) => Some(hdrext.to_value()),
                Err(e) => {
                  warn!("request-extension: {:?}", e);
                  None
                },
              }
            });

            pipeline
              .add(&depayloader)
              .context("failed to add depayloader to pipeline")?;
            depayloader.sync_state_with_parent()?;
            debug!("created depayloader");
            rtpbin
              .link_pads(Some(&pad_name), &depayloader, None)
              .context(format!("failed to link rtpbin.{} to depayloader", pad_name))?;

            debug!("linked rtpbin.{} to depayloader", pad_name);

            debug!("rtpbin pads:\n{}", dump_pads(&rtpbin));

            let queue = gstreamer::ElementFactory::make("queue", None)?;
            pipeline
              .add(&queue)
              .context("failed to add queue to pipeline")?;
            queue.sync_state_with_parent()?;
            depayloader
              .link(&queue)
              .context("failed to link depayloader to queue")?;

            let decoder = match source.media_type {
              MediaType::Audio => {
                let codec = codecs
                  .iter()
                  .filter(|codec| codec.is_audio())
                  .find(|codec| codec.is(pt));
                if let Some(codec) = codec {
                  gstreamer::ElementFactory::make(codec.decoder_name(), None)?
                  // TODO: fec
                }
                else {
                  bail!("received audio with unsupported PT {}", pt);
                }
              },
              MediaType::Video => {
                let codec = codecs
                  .iter()
                  .filter(|codec| codec.is_video())
                  .find(|codec| codec.is(pt));
                if let Some(codec) = codec {
                  let decoder = gstreamer::ElementFactory::make(codec.decoder_name(), None)?;
                  decoder.set_property("automatic-request-sync-points", true);
                  decoder.set_property_from_str(
                    "automatic-request-sync-point-flags",
                    "GST_VIDEO_DECODER_REQUEST_SYNC_POINT_CORRUPT_OUTPUT",
                  );
                  decoder
                }
                else {
                  bail!("received video with unsupported PT {}", pt);
                }
              },
            };

            pipeline
              .add(&decoder)
              .context("failed to add decoder to pipeline")?;
            decoder.sync_state_with_parent()?;
            queue
              .link(&decoder)
              .context("failed to link queue to decoder")?;

            let src_pad = match source.media_type {
              MediaType::Audio => decoder
                .static_pad("src")
                .context("decoder has no src pad")?,
              MediaType::Video => {
                let videoscale = gstreamer::ElementFactory::make("videoscale", None)?;
                pipeline
                  .add(&videoscale)
                  .context("failed to add videoscale to pipeline")?;
                videoscale.sync_state_with_parent()?;
                decoder
                  .link(&videoscale)
                  .context("failed to link decoder to videoscale")?;

                let capsfilter = gstreamer::ElementFactory::make("capsfilter", None)?;
                capsfilter.set_property_from_str(
                  "caps",
                  &format!(
                    "video/x-raw, width={}, height={}",
                    conference.config.recv_video_scale_width,
                    conference.config.recv_video_scale_height
                  ),
                );
                pipeline
                  .add(&capsfilter)
                  .context("failed to add capsfilter to pipeline")?;
                capsfilter.sync_state_with_parent()?;
                videoscale
                  .link(&capsfilter)
                  .context("failed to link videoscale to capsfilter")?;

                let videoconvert = gstreamer::ElementFactory::make("videoconvert", None)?;
                pipeline
                  .add(&videoconvert)
                  .context("failed to add videoconvert to pipeline")?;
                videoconvert.sync_state_with_parent()?;
                capsfilter
                  .link(&videoconvert)
                  .context("failed to link capsfilter to videoconvert")?;

                videoconvert
                  .static_pad("src")
                  .context("videoconvert has no src pad")?
              },
            };

            if let Some(participant_id) = source.participant_id {
              handle.block_on(conference.ensure_participant(&participant_id))?;
              let maybe_sink_element = match source.media_type {
                MediaType::Audio => {
                  handle.block_on(conference.remote_participant_audio_sink_element())
                },
                MediaType::Video => {
                  handle.block_on(conference.remote_participant_video_sink_element())
                },
              };
              if let Some(sink_element) = maybe_sink_element {
                let sink_pad = sink_element
                  .request_pad_simple("sink_%u")
                  .context("no suitable sink pad provided by sink element in recv pipeline")?;
                let ghost_pad = GhostPad::with_target(
                  Some(&format!(
                    "participant_{}_{:?}",
                    participant_id, source.media_type
                  )),
                  &sink_pad,
                )?;
                let bin: Bin = sink_element
                  .parent()
                  .context("sink element has no parent")?
                  .downcast()
                  .map_err(|_| anyhow!("sink element's parent is not a bin"))?;
                bin.add_pad(&ghost_pad)?;

                src_pad
                  .link(&ghost_pad)
                  .context("failed to link decode chain to participant bin from recv pipeline")?;
                info!(
                  "linked {}/{:?} to new pad in recv pipeline",
                  participant_id, source.media_type
                );
              }
              else if let Some(participant_bin) =
                pipeline.by_name(&format!("participant_{}", participant_id))
              {
                let sink_pad_name = match source.media_type {
                  MediaType::Audio => "audio",
                  MediaType::Video => "video",
                };
                if let Some(sink_pad) = participant_bin.static_pad(sink_pad_name) {
                  src_pad.link(&sink_pad).context(
                    "failed to link decode chain to participant bin from recv participant pipeline",
                  )?;
                  info!(
                    "linked {}/{:?} to recv participant pipeline",
                    participant_id, source.media_type
                  );
                }
                else {
                  warn!(
                    "no {} sink pad on {} participant bin in recv participant pipeline",
                    sink_pad_name, participant_id
                  );
                }
              }
              else {
                warn!("no pipeline handled new participant: {}", participant_id);
              }
            }
            else {
              debug!("not looking for participant bin, source is owned by JVB");
            }

            if !src_pad.is_linked() {
              debug!("nothing linked to decoder, adding fakesink");
              let fakesink = gstreamer::ElementFactory::make("fakesink", None)?;
              pipeline.add(&fakesink)?;
              fakesink.sync_state_with_parent()?;
              let sink_pad = fakesink
                .static_pad("sink")
                .context("fakesink has no sink pad")?;
              src_pad.link(&sink_pad)?;
            }

            gstreamer::debug_bin_to_dot_file(
              &pipeline,
              gstreamer::DebugGraphDetails::ALL,
              &format!("ssrc-added-{}", ssrc),
            );

            Ok::<_, anyhow::Error>(())
          }
          else {
            Ok(())
          }
        };
        if let Err(e) = f() {
          error!("handling pad-added: {:?}", e);
        }
        None
      });
    }

    let opus = codecs.iter().find(|codec| codec.name == CodecName::Opus);
    let audio_sink_element = if let Some(opus) = opus {
      let audio_sink_element = gstreamer::ElementFactory::make(opus.payloader_name(), None)?;
      audio_sink_element.set_property("pt", opus.pt as u32);
      audio_sink_element
    }
    else {
      bail!("no opus payload type in jingle session-initiate");
    };
    audio_sink_element.set_property("min-ptime", 10i64 * 1000 * 1000);
    audio_sink_element.set_property("ssrc", audio_ssrc);
    if audio_sink_element.has_property("auto-header-extension", None) {
      audio_sink_element.set_property("auto-header-extension", false);
      audio_sink_element.connect("request-extension", false, move |values| {
        let f = || {
          let ext_id: u32 = values[1].get()?;
          let ext_uri: String = values[2].get()?;
          debug!(
            "audio payloader requested extension: {} {}",
            ext_id, ext_uri
          );
          let hdrext =
            RTPHeaderExtension::create_from_uri(&ext_uri).context("failed to create hdrext")?;
          hdrext.set_id(ext_id);
          Ok::<_, anyhow::Error>(hdrext)
        };
        match f() {
          Ok(hdrext) => Some(hdrext.to_value()),
          Err(e) => {
            warn!("request-extension: {:?}", e);
            None
          },
        }
      });
    }
    else {
      debug!("audio payloader: no rtp header extension support");
    }
    pipeline.add(&audio_sink_element)?;

    let codec_name = conference.config.video_codec.as_str();
    let codec = codecs.iter().find(|codec| codec.is_codec(codec_name));
    let video_sink_element = if let Some(codec) = codec {
      let element = gstreamer::ElementFactory::make(codec.payloader_name(), None)?;
      element.set_property("pt", codec.pt as u32);
      if codec.name == CodecName::H264 {
        element.set_property_from_str("aggregate-mode", "zero-latency");
      }
      else {
        element.set_property_from_str("picture-id-mode", "15-bit");
      }
      element
    }
    else {
      bail!("unsupported video codec: {}", codec_name);
    };
    video_sink_element.set_property("ssrc", video_ssrc);
    if video_sink_element.has_property("auto-header-extension", None) {
      video_sink_element.set_property("auto-header-extension", false);
      video_sink_element.connect("request-extension", false, move |values| {
        let f = || {
          let ext_id: u32 = values[1].get()?;
          let ext_uri: String = values[2].get()?;
          debug!(
            "video payloader requested extension: {} {}",
            ext_id, ext_uri
          );
          let hdrext =
            RTPHeaderExtension::create_from_uri(&ext_uri).context("failed to create hdrext")?;
          hdrext.set_id(ext_id);
          Ok::<_, anyhow::Error>(hdrext)
        };
        match f() {
          Ok(hdrext) => Some(hdrext.to_value()),
          Err(e) => {
            warn!("request-extension: {:?}", e);
            None
          },
        }
      });
    }
    else {
      debug!("video payloader: no rtp header extension support");
    }
    pipeline.add(&video_sink_element)?;

    let rtpfunnel = gstreamer::ElementFactory::make("rtpfunnel", None)?;
    pipeline.add(&rtpfunnel)?;

    debug!("linking video payloader -> rtpfunnel");
    video_sink_element.link(&rtpfunnel)?;

    debug!("linking audio payloader -> rtpfunnel");
    audio_sink_element.link(&rtpfunnel)?;

    debug!("linking rtpfunnel -> rtpbin");
    rtpfunnel.link_pads(None, &rtpbin, Some("send_rtp_sink_0"))?;

    let rtp_recv_identity = gstreamer::ElementFactory::make("identity", None)?;
    pipeline.add(&rtp_recv_identity)?;
    let rtcp_recv_identity = gstreamer::ElementFactory::make("identity", None)?;
    pipeline.add(&rtcp_recv_identity)?;
    let rtp_send_identity = gstreamer::ElementFactory::make("identity", None)?;
    pipeline.add(&rtp_send_identity)?;
    let rtcp_send_identity = gstreamer::ElementFactory::make("identity", None)?;
    pipeline.add(&rtcp_send_identity)?;

    #[cfg(feature = "log-rtp")]
    {
      if conference.config.log_rtp {
        debug!("setting up RTP packet logging");

        let make_rtp_logger = |direction: &'static str| {
          move |values: &[glib::Value]| -> Option<glib::Value> {
            let f = || {
              let buffer: gstreamer::Buffer = values[1].get()?;
              let rtp_buffer = RTPBuffer::from_buffer_readable(&buffer)?;
              debug!(
                ssrc = rtp_buffer.ssrc(),
                pt = rtp_buffer.payload_type(),
                seq = rtp_buffer.seq(),
                ts = rtp_buffer.timestamp(),
                marker = rtp_buffer.is_marker(),
                extension = rtp_buffer.is_extension(),
                payload_size = rtp_buffer.payload_size(),
                "RTP {}",
                direction,
              );
              Ok::<_, anyhow::Error>(())
            };
            if let Err(e) = f() {
              warn!("RTP {}: {:?}", direction, e);
            }
            None
          }
        };

        rtp_recv_identity.connect("handoff", false, make_rtp_logger("RECV"));
        rtp_send_identity.connect("handoff", false, make_rtp_logger("SEND"));
      }

      if conference.config.log_rtcp {
        debug!("setting up RTCP packet logging");

        let make_rtcp_logger = |direction: &'static str| {
          move |values: &[glib::Value]| -> Option<glib::Value> {
            let f = || {
              let buffer: gstreamer::Buffer = values[1].get()?;
              let mut buf = [0u8; 1500];
              buffer
                .copy_to_slice(0, &mut buf[..buffer.size()])
                .map_err(|_| anyhow!("invalid RTCP packet size"))?;
              let decoded = rtcp::packet::unmarshal(&mut &buf[..buffer.size()])?;
              debug!("RTCP {} size={}\n{:#?}", direction, buffer.size(), decoded,);
              Ok::<_, anyhow::Error>(())
            };
            if let Err(e) = f() {
              warn!("RTCP {}: {:?}", direction, e);
            }
            None
          }
        };

        rtcp_recv_identity.connect("handoff", false, make_rtcp_logger("RECV"));
        rtcp_send_identity.connect("handoff", false, make_rtcp_logger("SEND"));
      }
    }

    debug!("link dtlssrtpdec -> rtpbin");
    dtlssrtpdec.link_pads(Some("rtp_src"), &rtp_recv_identity, None)?;
    rtp_recv_identity.link_pads(None, &rtpbin, Some("recv_rtp_sink_0"))?;
    dtlssrtpdec.link_pads(Some("rtcp_src"), &rtcp_recv_identity, None)?;
    rtcp_recv_identity.link_pads(None, &rtpbin, Some("recv_rtcp_sink_0"))?;

    debug!("linking rtpbin -> dtlssrtpenc");
    rtpbin.link_pads(Some("send_rtp_src_0"), &rtp_send_identity, None)?;
    rtp_send_identity.link_pads(None, &dtlssrtpenc, Some("rtp_sink_0"))?;
    rtpbin.link_pads(Some("send_rtcp_src_0"), &rtcp_send_identity, None)?;
    rtcp_send_identity.link_pads(None, &dtlssrtpenc, Some("rtcp_sink_0"))?;

    debug!("rtpbin pads:\n{}", dump_pads(&rtpbin));

    debug!("linking ice src -> dtlssrtpdec");
    nicesrc.link(&dtlssrtpdec)?;

    debug!("linking dtlssrtpenc -> ice sink");
    dtlssrtpenc.link_pads(Some("src"), &nicesink, Some("sink"))?;

    let bus = pipeline.bus().context("failed to get pipeline bus")?;

    let (pipeline_state_null_tx, pipeline_state_null_rx) = oneshot::channel();
    tokio::spawn(async move {
      let mut stream = bus.stream();
      while let Some(msg) = stream.next().await {
        match msg.view() {
          gstreamer::MessageView::Error(e) => {
            if let Some(d) = e.debug() {
              error!("{}", d);
            }
          },
          gstreamer::MessageView::Warning(e) => {
            if let Some(d) = e.debug() {
              warn!("{}", d);
            }
          },
          gstreamer::MessageView::StateChanged(state)
            if state.current() == gstreamer::State::Null =>
          {
            debug!("pipeline state is null");
            pipeline_state_null_tx.send(()).unwrap();
            break;
          },
          _ => {},
        }
      }
    });

    gstreamer::debug_bin_to_dot_file(
      &pipeline,
      gstreamer::DebugGraphDetails::ALL,
      "session-initiate",
    );

    let local_candidates = ice_agent.local_candidates(ice_stream_id, ice_component_id);
    debug!("local candidates: {:?}", local_candidates);

    debug!("building Jingle session-accept");
    let mut jingle_accept = Jingle::new(Action::SessionAccept, jingle.sid.clone())
      .with_initiator(
        jingle
          .initiator
          .as_ref()
          .context("jingle session-initiate with no initiator")?
          .clone(),
      )
      .with_responder(Jid::Full(conference.jid.clone()));

    for initiate_content in &jingle.contents {
      let mut description = RtpDescription::new(initiate_content.name.0.clone());

      description.payload_types = if initiate_content.name.0 == "audio" {
        let codec = codecs.iter().find(|codec| codec.name == CodecName::Opus);
        if let Some(codec) = codec {
          let mut pt = PayloadType::new(codec.pt, "opus".to_owned(), 48000, 2);
          pt.rtcp_fbs = codec.rtcp_fbs.clone();
          vec![pt]
        }
        else {
          bail!("no opus payload type in jingle session-initiate");
        }
      }
      else {
        let mut pts = vec![];
        let codec_name = conference.config.video_codec.as_str();
        let codec = codecs.iter().find(|codec| codec.is_codec(codec_name));
        if let Some(codec) = codec {
          let mut pt = PayloadType::new(codec.pt, codec.encoding_name().to_owned(), 90000, 1);
          pt.rtcp_fbs = codec.rtcp_fbs.clone();
          pts.push(pt);
          if let Some(rtx_pt) = codec.rtx_pt {
            let mut rtx_pt = PayloadType::new(rtx_pt, "rtx".to_owned(), 90000, 1);
            rtx_pt.parameters = vec![jingle_rtp::Parameter {
              name: "apt".to_owned(),
              value: codec.pt.to_string(),
            }];
            rtx_pt.rtcp_fbs = codec
              .rtcp_fbs
              .clone()
              .into_iter()
              .filter(|fb| fb.type_ != "transport-cc")
              .collect();
            pts.push(rtx_pt);
          }
        }
        else {
          bail!("unsupported video codec: {}", codec_name);
        }
        pts
      };

      description.rtcp_mux = Some(RtcpMux);

      let mslabel = Uuid::new_v4().to_string();
      let label = Uuid::new_v4().to_string();
      let cname = Uuid::new_v4().to_string();

      description.ssrc = Some(if initiate_content.name.0 == "audio" {
        audio_ssrc.to_string()
      }
      else {
        video_ssrc.to_string()
      });

      description.ssrcs = if initiate_content.name.0 == "audio" {
        vec![jingle_ssma::Source::new(audio_ssrc)]
      }
      else {
        vec![
          jingle_ssma::Source::new(video_ssrc),
          jingle_ssma::Source::new(video_rtx_ssrc),
        ]
      };

      for ssrc in description.ssrcs.iter_mut() {
        ssrc.parameters.push(Parameter {
          name: "cname".to_owned(),
          value: Some(cname.clone()),
        });
        ssrc.parameters.push(Parameter {
          name: "msid".to_owned(),
          value: Some(format!("{} {}", mslabel, label)),
        });
      }

      description.ssrc_groups = if initiate_content.name.0 == "audio" {
        vec![]
      }
      else {
        vec![jingle_ssma::Group {
          semantics: Semantics::Fid,
          sources: vec![
            jingle_ssma::Source::new(video_ssrc),
            jingle_ssma::Source::new(video_rtx_ssrc),
          ],
        }]
      };

      if initiate_content.name.0 == "audio" {
        if let Some(hdrext) = audio_hdrext_ssrc_audio_level {
          description.hdrexts.push(RtpHdrext::new(
            hdrext,
            RTP_HDREXT_SSRC_AUDIO_LEVEL.to_owned(),
          ));
        }
        if let Some(hdrext) = audio_hdrext_transport_cc {
          description
            .hdrexts
            .push(RtpHdrext::new(hdrext, RTP_HDREXT_TRANSPORT_CC.to_owned()));
        }
      }
      else if initiate_content.name.0 == "video" {
        if let Some(hdrext) = video_hdrext_transport_cc {
          description
            .hdrexts
            .push(RtpHdrext::new(hdrext, RTP_HDREXT_TRANSPORT_CC.to_owned()));
        }
      }

      let mut transport = IceUdpTransport::new().with_fingerprint(Fingerprint {
        hash: Algo::Sha_256,
        setup: Some(Setup::Active),
        value: fingerprint.clone(),
        // required: Some(true.to_string()),
      });
      transport.ufrag = Some(ice_local_ufrag.clone());
      transport.pwd = Some(ice_local_pwd.clone());
      transport.candidates = vec![];
      for c in &local_candidates {
        let addr = c.addr();
        let foundation = c.foundation()?;
        transport.candidates.push(jingle_ice_udp::Candidate {
          component: c.component_id() as u8,
          foundation: foundation.to_owned(),
          generation: 0,
          id: Uuid::new_v4().to_string(),
          ip: addr.ip(),
          port: addr.port(),
          priority: c.priority(),
          protocol: "udp".to_owned(),
          type_: match c.type_() {
            nice::CandidateType::Host => jingle_ice_udp::Type::Host,
            nice::CandidateType::PeerReflexive => jingle_ice_udp::Type::Prflx,
            nice::CandidateType::ServerReflexive => jingle_ice_udp::Type::Srflx,
            nice::CandidateType::Relayed => jingle_ice_udp::Type::Relay,
            other => bail!("unsupported candidate type: {:?}", other),
          },
          rel_addr: None,
          rel_port: None,
          network: None,
        });
      }

      jingle_accept = jingle_accept.add_content(
        Content::new(Creator::Responder, initiate_content.name.clone())
          .with_senders(Senders::Both)
          .with_description(description)
          .with_transport(transport),
      );
    }

    jingle_accept = jingle_accept.set_group(jingle_grouping::Group {
      semantics: jingle_grouping::Semantics::Bundle,
      contents: vec![GroupContent::new("video"), GroupContent::new("audio")],
    });

    let accept_iq_id = generate_id();
    let session_accept_iq = Iq::from_set(accept_iq_id.clone(), jingle_accept)
      .with_to(Jid::Full(conference.focus_jid_in_muc()?))
      .with_from(Jid::Full(conference.jid.clone()));
    conference.xmpp_tx.send(session_accept_iq.into()).await?;

    Ok(Self {
      pipeline,
      audio_sink_element,
      video_sink_element,
      remote_ssrc_map,
      _ice_agent: ice_agent,
      accept_iq_id: Some(accept_iq_id),
      colibri_url: ice_transport.web_socket.clone().map(|ws| ws.url),
      colibri_channel: None,
      stats_handler_task: None,
      pipeline_state_null_rx,
    })
  }

  pub(crate) async fn source_add(&mut self, jingle: Jingle) -> Result<()> {
    for content in &jingle.contents {
      if let Some(Description::Rtp(description)) = &content.description {
        for ssrc in &description.ssrcs {
          let owner = ssrc
            .info
            .as_ref()
            .context("missing ssrc-info")?
            .owner
            .clone();

          debug!("adding ssrc to remote_ssrc_map: {:?}", ssrc);
          self.remote_ssrc_map.insert(
            ssrc.id,
            Source {
              ssrc: ssrc.id,
              participant_id: if owner == "jvb" {
                None
              }
              else {
                Some(
                  owner
                    .split('/')
                    .nth(1)
                    .context("invalid ssrc-info owner")?
                    .to_owned(),
                )
              },
              media_type: if description.media == "audio" {
                MediaType::Audio
              }
              else {
                MediaType::Video
              },
            },
          );
        }
      }
    }
    Ok(())
  }
}

fn dump_pads(element: &Element) -> String {
  element
    .pads()
    .into_iter()
    .map(|pad| {
      format!(
        "  {}, peer={}.{}, caps=\"{}\"",
        pad.name(),
        pad
          .peer()
          .and_then(|peer| peer.parent_element())
          .map(|element| element.name().to_string())
          .unwrap_or_default(),
        pad
          .peer()
          .map(|peer| peer.name().to_string())
          .unwrap_or_default(),
        pad.caps().map(|caps| caps.to_string()).unwrap_or_default(),
      )
    })
    .join("\n")
}
