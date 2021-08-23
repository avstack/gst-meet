use std::{collections::HashMap, fmt, net::SocketAddr};

use anyhow::{anyhow, bail, Context, Result};
use futures::stream::StreamExt;
use glib::{ObjectExt, ToValue};
use gstreamer::prelude::{ElementExt, GObjectExtManualGst, GstBinExt, PadExt};
use nice_gst_meet as nice;
use pem::Pem;
use rand::random;
use rcgen::{Certificate, CertificateParams, PKCS_ECDSA_P256_SHA256};
use ring::digest::{digest, SHA256};
use tokio::{
  net::lookup_host,
  runtime::Handle,
  sync::oneshot,
};
use tracing::{debug, error, warn};
use uuid::Uuid;
use xmpp_parsers::{
  hashes::Algo,
  iq::Iq,
  jingle::{Action, Content, Creator, Description, Jingle, Senders, Transport},
  jingle_dtls_srtp::{Fingerprint, Setup},
  jingle_ice_udp::{self, Transport as IceUdpTransport},
  jingle_rtp::{Description as RtpDescription, PayloadType, RtcpMux},
  jingle_rtp_hdrext::RtpHdrext,
  jingle_ssma::{self, Parameter},
  Jid,
};

use crate::{
  colibri::ColibriChannel,
  conference::JitsiConference,
  source::{MediaType, Source},
  util::generate_id,
};

const RTP_HDREXT_SSRC_AUDIO_LEVEL: &str = "urn:ietf:params:rtp-hdrext:ssrc-audio-level";
const RTP_HDREXT_ABS_SEND_TIME: &str = "http://www.webrtc.org/experiments/rtp-hdrext/abs-send-time";
const RTP_HDREXT_TRANSPORT_CC: &str = "http://www.ietf.org/id/draft-holmer-rmcat-transport-wide-cc-extensions-01";

const RTX_PAYLOAD_TYPE_VP8: u8 = 96;
const RTX_PAYLOAD_TYPE_VP9: u8 = 97;
const RTX_PAYLOAD_TYPE_H264: u8 = 99;

const DEFAULT_STUN_PORT: u16 = 3478;
const DEFAULT_TURNS_PORT: u16 = 5349;

pub(crate) struct JingleSession {
  pipeline: gstreamer::Pipeline,
  audio_sink_element: gstreamer::Element,
  video_sink_element: gstreamer::Element,
  remote_ssrc_map: HashMap<u32, Source>,
  _ice_agent: nice::Agent,
  pub(crate) accept_iq_id: Option<String>,
  pub(crate) colibri_url: Option<String>,
  pub(crate) colibri_channel: Option<ColibriChannel>,
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
        let pad_name: String = pad.property("name").unwrap().get().unwrap();
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

  pub(crate) async fn initiate(conference: &JitsiConference, jingle: Jingle) -> Result<Self> {
    let initiator = jingle
      .initiator
      .as_ref()
      .ok_or_else(|| anyhow!("session-initiate with no initiator"))?
      .clone();

    debug!("Received Jingle session-initiate from {}", initiator);

    let mut ice_remote_candidates = None;
    let mut ice_remote_ufrag = None;
    let mut ice_remote_pwd = None;
    let mut dtls_fingerprint = None;
    let mut opus_payload_type = None;
    let mut h264_payload_type = None;
    let mut vp8_payload_type = None;
    let mut vp9_payload_type = None;
    let mut audio_hdrext_ssrc_audio_level = None;
    let mut audio_hdrext_transport_cc = None;
    let mut video_hdrext_abs_send_time = None;
    let mut video_hdrext_transport_cc = None;
    let mut colibri_url = None;

    let mut remote_ssrc_map = HashMap::new();

    for content in &jingle.contents {
      if let Some(Description::Rtp(description)) = &content.description {
        if description.media == "audio" {
          opus_payload_type = description
            .payload_types
            .iter()
            .find(|pt| pt.name.as_deref() == Some("opus"))
            .map(|pt| pt.id);
          audio_hdrext_ssrc_audio_level = description
            .hdrexts
            .iter()
            .find(|hdrext| hdrext.uri == RTP_HDREXT_SSRC_AUDIO_LEVEL)
            .map(|hdrext| hdrext.id.parse::<u8>())
            .transpose()?;
          audio_hdrext_transport_cc = description
            .hdrexts
            .iter()
            .find(|hdrext| hdrext.uri == RTP_HDREXT_TRANSPORT_CC)
            .map(|hdrext| hdrext.id.parse::<u8>())
            .transpose()?;
        }
        else if description.media == "video" {
          h264_payload_type = description
            .payload_types
            .iter()
            .find(|pt| pt.name.as_deref() == Some("H264"))
            .map(|pt| pt.id);
          vp8_payload_type = description
            .payload_types
            .iter()
            .find(|pt| pt.name.as_deref() == Some("VP8"))
            .map(|pt| pt.id);
          vp9_payload_type = description
            .payload_types
            .iter()
            .find(|pt| pt.name.as_deref() == Some("VP9"))
            .map(|pt| pt.id);
          video_hdrext_abs_send_time = description
            .hdrexts
            .iter()
            .find(|hdrext| hdrext.uri == RTP_HDREXT_ABS_SEND_TIME)
            .map(|hdrext| hdrext.id.parse::<u8>())
            .transpose()?;
          video_hdrext_transport_cc = description
            .hdrexts
            .iter()
            .find(|hdrext| hdrext.uri == RTP_HDREXT_TRANSPORT_CC)
            .map(|hdrext| hdrext.id.parse::<u8>())
            .transpose()?;
        }
        else {
          continue;
        }

        for ssrc in &description.ssrcs {
          let owner = ssrc
            .info
            .as_ref()
            .context("missing ssrc-info")?
            .owner
            .clone();
          if owner == "jvb" {
            debug!("skipping ssrc (owner = jvb)");
            continue;
          }

          remote_ssrc_map.insert(
            ssrc.id.parse()?,
            Source {
              ssrc: ssrc.id.parse()?,
              participant_id: owner
                .split('/')
                .nth(1)
                .context("invalid ssrc-info owner")?
                .to_owned(),
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

      if let Some(Transport::IceUdp(transport)) = &content.transport {
        if !transport.candidates.is_empty() {
          ice_remote_candidates = Some(transport.candidates.clone());
        }
        if let Some(ufrag) = &transport.ufrag {
          ice_remote_ufrag = Some(ufrag.to_owned());
        }
        if let Some(pwd) = &transport.pwd {
          ice_remote_pwd = Some(pwd.to_owned());
        }
        if let Some(fingerprint) = &transport.fingerprint {
          if fingerprint.hash != Algo::Sha_256 {
            bail!("unsupported fingerprint hash: {:?}", fingerprint.hash);
          }
          dtls_fingerprint = Some(fingerprint.value.clone());
        }
        if let Some(websocket) = &transport.web_socket {
          colibri_url = Some(websocket.url.clone());
        }
      }
    }

    if let Some(remote_fingerprint) = dtls_fingerprint {
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

    debug!("audio SSRC: {}", audio_ssrc);
    debug!("video SSRC: {}", video_ssrc);

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

    let (ice_local_ufrag, ice_local_pwd) = ice_agent
      .local_credentials(ice_stream_id)
      .context("no local ICE credentials")?;

    if let (Some(ufrag), Some(pwd)) = (&ice_remote_ufrag, &ice_remote_pwd) {
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

    if let (Some(ufrag), Some(pwd), Some(remote_candidates)) =
      (&ice_remote_ufrag, &ice_remote_pwd, &ice_remote_candidates)
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

    debug!("building gstreamer pipeline");

    let pipeline = gstreamer::Pipeline::new(None);

    let rtpbin = gstreamer::ElementFactory::make("rtpbin", Some("rtpbin"))?;
    rtpbin.set_property_from_str("rtp-profile", "savpf");
    rtpbin.set_property("do-retransmission", true)?;
    rtpbin.set_property("autoremove", true)?;
    pipeline.add(&rtpbin)?;

    let nicesrc = gstreamer::ElementFactory::make("nicesrc", None)?;
    nicesrc.set_property("stream", ice_stream_id)?;
    nicesrc.set_property("component", ice_component_id)?;
    nicesrc.set_property("agent", &ice_agent)?;
    pipeline.add(&nicesrc)?;

    let nicesink = gstreamer::ElementFactory::make("nicesink", None)?;
    nicesink.set_property("stream", ice_stream_id)?;
    nicesink.set_property("component", ice_component_id)?;
    nicesink.set_property("agent", &ice_agent)?;
    pipeline.add(&nicesink)?;

    let dtls_srtp_connection_id = "gst-meet";

    let dtlssrtpdec = gstreamer::ElementFactory::make("dtlssrtpdec", None)?;
    dtlssrtpdec.set_property("connection-id", dtls_srtp_connection_id)?;
    dtlssrtpdec.set_property(
      "pem",
      format!("{}\n{}", dtls_cert_pem, dtls_private_key_pem),
    )?;
    pipeline.add(&dtlssrtpdec)?;

    let dtlssrtpenc = gstreamer::ElementFactory::make("dtlssrtpenc", None)?;
    dtlssrtpenc.set_property("connection-id", dtls_srtp_connection_id)?;
    dtlssrtpenc.set_property("is-client", true)?;
    pipeline.add(&dtlssrtpenc)?;

    rtpbin.connect("request-pt-map", false, move |values| {
      let f = || {
        debug!("rtpbin request-pt-map {:?}", values);
        let pt = values[2].get::<u32>()? as u8;
        let mut caps = gstreamer::Caps::builder("application/x-rtp");
        if Some(pt) == opus_payload_type {
          caps = caps
            .field("media", "audio")
            .field("encoding-name", "OPUS")
            .field("clock-rate", 48000);
          if let Some(hdrext) = audio_hdrext_ssrc_audio_level {
            caps = caps.field(&format!("extmap-{}", hdrext), RTP_HDREXT_SSRC_AUDIO_LEVEL);
          }
          if let Some(hdrext) = audio_hdrext_transport_cc {
            caps = caps.field(&format!("extmap-{}", hdrext), &RTP_HDREXT_TRANSPORT_CC);
          }
          Ok::<_, anyhow::Error>(Some(caps.build()))
        }
        else if Some(pt) == h264_payload_type || Some(pt) == vp8_payload_type || Some(pt) == vp9_payload_type {
          caps = caps
            .field("media", "video")
            .field("clock-rate", 90000)
            .field("encoding-name", if Some(pt) == h264_payload_type {
              "H264"
            }
            else if Some(pt) == vp8_payload_type {
              "VP8"
            }
            else if Some(pt) == vp9_payload_type {
              "VP9"
            }
            else {
              unreachable!()
            });
          if let Some(hdrext) = video_hdrext_abs_send_time {
            caps = caps.field(&format!("extmap-{}", hdrext), &RTP_HDREXT_ABS_SEND_TIME);
          }
          if let Some(hdrext) = video_hdrext_transport_cc {
            caps = caps.field(&format!("extmap-{}", hdrext), &RTP_HDREXT_TRANSPORT_CC);
          }
          Ok(Some(caps.build()))
        }
        else if pt == RTX_PAYLOAD_TYPE_VP8 || pt == RTX_PAYLOAD_TYPE_VP9 || pt == RTX_PAYLOAD_TYPE_H264 {
          caps = caps
            .field("media", "video")
            .field("clock-rate", 90000)
            .field("encoding-name", "RTX")
            .field("apt", if pt == RTX_PAYLOAD_TYPE_VP8 {
              vp8_payload_type
                .context("missing VP8 payload type")?
            }
            else if pt == RTX_PAYLOAD_TYPE_VP9 {
              vp9_payload_type
                .context("missing VP9 payload type")?
            }
            else if pt == RTX_PAYLOAD_TYPE_H264 {
              h264_payload_type
                .context("missing H264 payload type")?
            }
            else {
              unreachable!()
            });
          Ok(Some(caps.build()))
        }
        else {
          warn!("unknown payload type: {}", pt);
          Ok(None)
        }
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
    })?;

    rtpbin.connect("request-aux-sender", false, move |values| {
      let f = move || {
        let session: u32 = values[1].get()?;
        debug!("creating RTX sender for session {}", session);
        let mut pt_map = gstreamer::Structure::builder("application/x-rtp-pt-map");
        if let Some(pt) = vp8_payload_type {
          pt_map = pt_map.field(&pt.to_string(), &(RTX_PAYLOAD_TYPE_VP8 as u32));
        }
        if let Some(pt) = vp9_payload_type {
          pt_map = pt_map.field(&pt.to_string(), &(RTX_PAYLOAD_TYPE_VP9 as u32));
        }
        if let Some(pt) = h264_payload_type {
          pt_map = pt_map.field(&pt.to_string(), &(RTX_PAYLOAD_TYPE_H264 as u32));
        }
        let bin = gstreamer::Bin::new(None);
        let rtx_sender = gstreamer::ElementFactory::make("rtprtxsend", None)?;
        rtx_sender.set_property("payload-type-map", pt_map.build())?;
        bin.add(&rtx_sender)?;
        bin.add_pad(
          &gstreamer::GhostPad::with_target(
            Some(&format!("src_{}", session)),
            &rtx_sender
              .static_pad("src")
              .context("rtprtxsend has no src pad")?,
          )?,
        )?;
        bin.add_pad(
          &gstreamer::GhostPad::with_target(
            Some(&format!("sink_{}", session)),
            &rtx_sender
              .static_pad("sink")
              .context("rtprtxsend has no sink pad")?,
          )?,
        )?;
        Ok::<_, anyhow::Error>(Some(bin.to_value()))
      };
      match f() {
        Ok(o) => o,
        Err(e) => {
          warn!("request-aux-sender: {:?}", e);
          None
        },
      }
    })?;

    rtpbin.connect("request-aux-receiver", false, move |values| {
      let f = move || {
        let session: u32 = values[1].get()?;
        debug!("creating RTX receiver for session {}", session);
        let mut pt_map = gstreamer::Structure::builder("application/x-rtp-pt-map");
        if let Some(pt) = vp8_payload_type {
          pt_map = pt_map.field(&pt.to_string(), RTX_PAYLOAD_TYPE_VP8 as u32);
        }
        if let Some(pt) = vp9_payload_type {
          pt_map = pt_map.field(&pt.to_string(), RTX_PAYLOAD_TYPE_VP9 as u32);
        }
        if let Some(pt) = h264_payload_type {
          pt_map = pt_map.field(&pt.to_string(), RTX_PAYLOAD_TYPE_H264 as u32);
        }
        let bin = gstreamer::Bin::new(None);
        let rtx_receiver = gstreamer::ElementFactory::make("rtprtxreceive", None)?;
        rtx_receiver.set_property("payload-type-map", pt_map.build())?;
        bin.add(&rtx_receiver)?;
        bin.add_pad(
          &gstreamer::GhostPad::with_target(
            Some(&format!("src_{}", session)),
            &rtx_receiver.static_pad("src")
              .context("rtprtxreceive has no src pad")?,
          )?,
        )?;
        bin.add_pad(
          &gstreamer::GhostPad::with_target(
            Some(&format!("sink_{}", session)),
            &rtx_receiver.static_pad("sink")
              .context("rtprtxreceive has no sink pad")?,
          )?,
        )?;
        Ok::<_, anyhow::Error>(Some(bin.to_value()))
      };
      match f() {
        Ok(o) => o,
        Err(e) => {
          warn!("request-aux-receiver: {:?}", e);
          None
        },
      }
    })?;

    let handle = Handle::current();
    let inner_ = conference.inner.clone();
    let pipeline_ = pipeline.clone();
    let rtpbin_ = rtpbin.clone();
    rtpbin.connect("pad-added", false, move |values| {
      let inner_ = inner_.clone();
      let handle = handle.clone();
      let pipeline_ = pipeline_.clone();
      let rtpbin_ = rtpbin_.clone();
      let f = move || {
        debug!("rtpbin pad-added {:?}", values);
        let pad: gstreamer::Pad = values[1].get()?;
        let pad_name: String = pad.property("name")?.get()?;
        if pad_name.starts_with("recv_rtp_src_0_") {
          let mut parts = pad_name.split('_').skip(4);
          let ssrc: u32 = parts.next().context("malformed pad name")?.parse()?;
          let pt: u8 = parts.next().context("malformed pad name")?.parse()?;
          let source = handle.block_on(async move {
            let locked_inner = inner_.lock().await;
            let jingle_session = locked_inner
              .jingle_session
              .as_ref()
              .context("not connected (no jingle session)")?;
            Ok::<_, anyhow::Error>(
              jingle_session
                .remote_ssrc_map
                .get(&ssrc)
                .context(format!("unknown ssrc: {}", ssrc))?
                .clone(),
            )
          })?;

          debug!("pad added for remote source: {:?}", source);

          let element_name = match source.media_type {
            MediaType::Audio => {
              if Some(pt) == opus_payload_type {
                "rtpopusdepay"
              }
              else {
                bail!("received audio with unsupported PT {}", pt);
              }
            },
            MediaType::Video => {
              if Some(pt) == h264_payload_type {
                "rtph264depay"
              }
              else if Some(pt) == vp8_payload_type {
                "rtpvp8depay"
              }
              else if Some(pt) == vp9_payload_type {
                "rtpvp9depay"
              }
              else {
                bail!("received video with unsupported PT {}", pt);
              }
            },
          };

          let source_element = gstreamer::ElementFactory::make(element_name, None)?;
          if source_element.has_property("auto-header-extension", None) {
            source_element.set_property("auto-header-extension", true)?;
          }
          if source_element.has_property("request-keyframe", None) {
            source_element.set_property("request-keyframe", true)?;
          }
          pipeline_
            .add(&source_element)
            .context(format!("failed to add {} to pipeline", element_name))?;
          source_element.sync_state_with_parent()?;
          debug!("created {} element", element_name);
          rtpbin_
            .link_pads(Some(&pad_name), &source_element, None)
            .context(format!(
              "failed to link rtpbin.{} to {}",
              pad_name, element_name
            ))?;
          debug!("linked rtpbin.{} to {}", pad_name, element_name);

          let src_pad = source_element
            .static_pad("src")
            .context("depayloader has no src pad")?;

          if let Some(participant_bin) =
            pipeline_.by_name(&format!("participant_{}", source.participant_id))
          {
            let sink_pad_name = match source.media_type {
              MediaType::Audio => "audio",
              MediaType::Video => "video",
            };
            if let Some(sink_pad) = participant_bin.static_pad(sink_pad_name) {
              debug!("linking depayloader to participant bin");
              src_pad.link(&sink_pad)?;
            }
            else {
              warn!(
                "no {} sink pad in {} participant bin",
                sink_pad_name, source.participant_id
              );
            }
          }
          else {
            debug!("no participant bin for {}", source.participant_id);
          }

          if !src_pad.is_linked() {
            debug!("nothing linked to {}, adding fakesink", element_name);
            let fakesink = gstreamer::ElementFactory::make("fakesink", None)?;
            pipeline_.add(&fakesink)?;
            fakesink.sync_state_with_parent()?;
            source_element.link(&fakesink)?;
          }

          gstreamer::debug_bin_to_dot_file(
            &pipeline_,
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
    })?;

    let audio_sink_element = gstreamer::ElementFactory::make("rtpopuspay", None)?;
    audio_sink_element.set_property(
      "pt",
      opus_payload_type.context("no opus payload type in jingle session-initiate")? as u32,
    )?;
    audio_sink_element.set_property("min-ptime", 10i64 * 1000 * 1000)?;
    audio_sink_element.set_property("ssrc", audio_ssrc)?;
    let audio_hdrext_supported = if audio_sink_element.has_property("auto-header-extension", None) {
      audio_sink_element.set_property("auto-header-extension", true)?;
      true
    }
    else {
      false
    };
    pipeline.add(&audio_sink_element)?;

    let video_sink_element = match conference.config.video_codec.as_str() {
      "h264" => {
        let element = gstreamer::ElementFactory::make("rtph264pay", None)?;
        element.set_property(
          "pt",
          h264_payload_type.context("no h264 payload type in jingle session-initiate")? as u32,
        )?;
        element.set_property_from_str("aggregate-mode", "zero-latency");
        element
      },
      "vp8" => {
        let element = gstreamer::ElementFactory::make("rtpvp8pay", None)?;
        element.set_property(
          "pt",
          vp8_payload_type.context("no vp8 payload type in jingle session-initiate")? as u32,
        )?;
        element.set_property_from_str("picture-id-mode", "15-bit");
        element
      },
      "vp9" => {
        let element = gstreamer::ElementFactory::make("rtpvp9pay", None)?;
        element.set_property(
          "pt",
          vp9_payload_type.context("no vp9 payload type in jingle session-initiate")? as u32,
        )?;
        element.set_property_from_str("picture-id-mode", "15-bit");
        element
      },
      other => bail!("unsupported video codec: {}", other),
    };
    video_sink_element.set_property("ssrc", video_ssrc)?;
    let video_hdrext_supported = if video_sink_element.has_property("auto-header-extension", None) {
      video_sink_element.set_property("auto-header-extension", true)?;
      true
    }
    else {
      false
    };
    pipeline.add(&video_sink_element)?;

    let mut audio_caps = gstreamer::Caps::builder("application/x-rtp");
    // TODO: fails to negotiate
    // if let Some(hdrext) = audio_hdrext_ssrc_audio_level {
    //   audio_caps = audio_caps.field(&format!("extmap-{}", hdrext), RTP_HDREXT_SSRC_AUDIO_LEVEL);
    // }
    if let Some(hdrext) = audio_hdrext_transport_cc {
      audio_caps = audio_caps.field(&format!("extmap-{}", hdrext), RTP_HDREXT_TRANSPORT_CC);
    }
    let audio_capsfilter = gstreamer::ElementFactory::make("capsfilter", None)?;
    audio_capsfilter.set_property("caps", audio_caps.build())?;
    pipeline.add(&audio_capsfilter)?;

    let mut video_caps = gstreamer::Caps::builder("application/x-rtp");
    if let Some(hdrext) = video_hdrext_abs_send_time {
      video_caps = video_caps.field(&format!("extmap-{}", hdrext), &RTP_HDREXT_ABS_SEND_TIME);
    }
    if let Some(hdrext) = video_hdrext_transport_cc {
      video_caps = video_caps.field(&format!("extmap-{}", hdrext), &RTP_HDREXT_TRANSPORT_CC);
    }
    let video_capsfilter = gstreamer::ElementFactory::make("capsfilter", None)?;
    video_capsfilter.set_property("caps", video_caps.build())?;
    pipeline.add(&video_capsfilter)?;

    debug!("linking audio payloader -> rtpbin");
    audio_sink_element.link(&audio_capsfilter)?;
    audio_capsfilter.link_pads(None, &rtpbin, Some("send_rtp_sink_0"))?;

    debug!("linking video payloader -> rtpbin");
    video_sink_element.link(&video_capsfilter)?;
    video_capsfilter.link_pads(None, &rtpbin, Some("send_rtp_sink_1"))?;

    debug!("linking ICE <-> DTLS-SRTP");
    nicesrc.link(&dtlssrtpdec)?;
    dtlssrtpenc.link(&nicesink)?;

    debug!("linking rtpbin -> DTLS-SRTP encoder");
    rtpbin.link_pads(Some("send_rtp_src_0"), &dtlssrtpenc, Some("rtp_sink_0"))?;
    rtpbin.link_pads(Some("send_rtcp_src_0"), &dtlssrtpenc, Some("rtcp_sink_0"))?;
    rtpbin.link_pads(Some("send_rtp_src_1"), &dtlssrtpenc, Some("rtp_sink_1"))?;
    rtpbin.link_pads(Some("send_rtcp_src_1"), &dtlssrtpenc, Some("rtcp_sink_1"))?;

    debug!("linking DTLS-SRTP decoder -> rtpbin");
    dtlssrtpdec.link_pads(Some("rtp_src"), &rtpbin, Some("recv_rtp_sink_0"))?;
    dtlssrtpdec.link_pads(Some("rtcp_src"), &rtpbin, Some("recv_rtcp_sink_0"))?;

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
          }
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
        vec![PayloadType::new(
          opus_payload_type.context("no opus payload type in jingle session-initiate")?,
          "opus".to_owned(),
          48000,
          2,
        )]
      }
      else {
        match conference.config.video_codec.as_str() {
          "h264" => vec![PayloadType::new(
            h264_payload_type.context("no h264 payload type in jingle session-initiate")?,
            "H264".to_owned(),
            90000,
            1,
          )],
          "vp8" => vec![PayloadType::new(
            vp8_payload_type.context("no vp8 payload type in jingle session-initiate")?,
            "VP8".to_owned(),
            90000,
            1,
          )],
          "vp9" => vec![PayloadType::new(
            vp9_payload_type.context("no vp9 payload type in jingle session-initiate")?,
            "VP9".to_owned(),
            90000,
            1,
          )],
          other => bail!("unsupported video codec: {}", other),
        }
      };

      description.rtcp_mux = Some(RtcpMux);

      let mslabel = Uuid::new_v4().to_string();
      let label = Uuid::new_v4().to_string();
      let cname = Uuid::new_v4().to_string();

      description.ssrcs = if initiate_content.name.0 == "audio" {
        vec![jingle_ssma::Source::new(audio_ssrc.to_string())]
      }
      else {
        vec![jingle_ssma::Source::new(video_ssrc.to_string())]
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
        ssrc.parameters.push(Parameter {
          name: "mslabel".to_owned(),
          value: Some(mslabel.clone()),
        });
        ssrc.parameters.push(Parameter {
          name: "label".to_owned(),
          value: Some(label.clone()),
        });
      }

      if initiate_content.name.0 == "audio" {
        // TODO: fails to negotiate
        // if let Some(hdrext) = audio_hdrext_ssrc_audio_level {
        //   if audio_hdrext_supported {
        //     description.hdrexts.push(RtpHdrext::new(hdrext.to_string(), RTP_HDREXT_SSRC_AUDIO_LEVEL.to_owned()));
        //   }
        //   else {
        //     debug!("ssrc-audio-level hdrext requested but not supported");
        //   }
        // }
        if let Some(hdrext) = audio_hdrext_transport_cc {
          if audio_hdrext_supported {
            description.hdrexts.push(RtpHdrext::new(hdrext.to_string(), RTP_HDREXT_TRANSPORT_CC.to_owned()));
          }
          else {
            debug!("transport-cc hdrext requested, but hdrext not supported by audio payloader");
          }
        }
      }
      else if initiate_content.name.0 == "video" {
        if let Some(hdrext) = video_hdrext_abs_send_time {
          if video_hdrext_supported {
            description.hdrexts.push(RtpHdrext::new(hdrext.to_string(), RTP_HDREXT_ABS_SEND_TIME.to_owned()));
          }
          else {
            debug!("abs-send-time hdrext requested, but hdrext not supported by video payloader");
          }
        }
        if let Some(hdrext) = video_hdrext_transport_cc {
          if video_hdrext_supported {
            description.hdrexts.push(RtpHdrext::new(hdrext.to_string(), RTP_HDREXT_TRANSPORT_CC.to_owned()));
          }
          else {
            debug!("transport-cc hdrext requested, but hdrext not supported by video payloader");
          }
        }
      }

      let mut transport = IceUdpTransport::new().with_fingerprint(Fingerprint {
        hash: Algo::Sha_256,
        setup: Some(Setup::Active),
        value: fingerprint.clone(),
        required: Some(true.to_string()),
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
      colibri_url,
      colibri_channel: None,
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
          if owner == "jvb" {
            debug!("skipping ssrc (owner = jvb)");
            continue;
          }

          debug!("adding ssrc to remote_ssrc_map: {:?}", ssrc);
          self.remote_ssrc_map.insert(
            ssrc.id.parse()?,
            Source {
              ssrc: ssrc.id.parse()?,
              participant_id: owner
                .split('/')
                .nth(1)
                .context("invalid ssrc-info owner")?
                .to_owned(),
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
