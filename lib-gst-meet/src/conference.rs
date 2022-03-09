use std::{
  collections::HashMap, convert::TryFrom, fmt, future::Future, pin::Pin, sync::Arc, time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use colibri::{ColibriMessage, JsonMessage};
use futures::stream::StreamExt;
use glib::ObjectExt;
use gstreamer::prelude::{ElementExt, ElementExtManual, GstBinExt};
use jitsi_xmpp_parsers::jingle::{Action, Jingle};
use maplit::hashmap;
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::{
  sync::{mpsc, oneshot, Mutex},
  time,
};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;
pub use xmpp_parsers::disco::Feature;
use xmpp_parsers::{
  caps::{self, Caps},
  disco::{DiscoInfoQuery, DiscoInfoResult, Identity},
  ecaps2::{self, ECaps2},
  hashes::{Algo, Hash},
  iq::{Iq, IqType},
  message::{Message, MessageType},
  muc::{user::Status as MucStatus, Muc, MucUser},
  nick::Nick,
  ns,
  presence::{self, Presence},
  stanza_error::{DefinedCondition, ErrorType, StanzaError},
  BareJid, FullJid, Jid,
};

use crate::{
  colibri::ColibriChannel,
  jingle::JingleSession,
  source::MediaType,
  stanza_filter::StanzaFilter,
  util::generate_id,
  xmpp::{self, connection::Connection},
};

const SEND_STATS_INTERVAL: Duration = Duration::from_secs(10);

const DISCO_NODE: &str = "https://github.com/avstack/gst-meet";

static DISCO_INFO: Lazy<DiscoInfoResult> = Lazy::new(|| DiscoInfoResult {
  node: None,
  identities: vec![Identity::new("client", "bot", "en", "gst-meet")],
  features: vec![
    Feature::new(ns::DISCO_INFO),
    Feature::new(ns::JINGLE_RTP_AUDIO),
    Feature::new(ns::JINGLE_RTP_VIDEO),
    Feature::new(ns::JINGLE_ICE_UDP),
    Feature::new(ns::JINGLE_DTLS),
    Feature::new("urn:ietf:rfc:5888"), // BUNDLE
    Feature::new("urn:ietf:rfc:5761"), // RTCP-MUX
    Feature::new("urn:ietf:rfc:4588"), // RTX
    Feature::new("http://jitsi.org/tcc"),
  ],
  extensions: vec![],
});

static COMPUTED_CAPS_HASH: Lazy<Hash> =
  Lazy::new(|| caps::hash_caps(&caps::compute_disco(&DISCO_INFO), Algo::Sha_1).unwrap());

#[derive(Debug, Clone, Copy)]
enum JitsiConferenceState {
  Discovering,
  JoiningMuc,
  Idle,
}

#[derive(Debug, Clone)]
pub struct JitsiConferenceConfig {
  pub muc: BareJid,
  pub focus: Jid,
  pub nick: String,
  pub region: Option<String>,
  pub video_codec: String,
  pub extra_muc_features: Vec<String>,

  pub start_bitrate: u32,
  pub stereo: bool,

  pub recv_video_scale_width: u16,
  pub recv_video_scale_height: u16,

  pub buffer_size: u32,

  #[cfg(feature = "log-rtp")]
  pub log_rtp: bool,
  #[cfg(feature = "log-rtp")]
  pub log_rtcp: bool,
}

#[derive(Clone)]
pub struct JitsiConference {
  pub(crate) glib_main_context: glib::MainContext,
  pub(crate) jid: FullJid,
  pub(crate) xmpp_tx: mpsc::Sender<xmpp_parsers::Element>,
  pub(crate) config: JitsiConferenceConfig,
  pub(crate) external_services: Vec<xmpp::extdisco::Service>,
  pub(crate) jingle_session: Arc<Mutex<Option<JingleSession>>>,
  pub(crate) inner: Arc<Mutex<JitsiConferenceInner>>,
  pub(crate) tls_insecure: bool,
}

impl fmt::Debug for JitsiConference {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("JitsiConference")
      .field("jid", &self.jid)
      .field("config", &self.config)
      .field("inner", &self.inner)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct Participant {
  pub jid: Option<FullJid>,
  pub muc_jid: FullJid,
  pub nick: Option<String>,
}

type BoxedResultFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

pub(crate) struct JitsiConferenceInner {
  participants: HashMap<String, Participant>,
  audio_sink: Option<gstreamer::Element>,
  video_sink: Option<gstreamer::Element>,
  on_participant:
    Option<Arc<dyn (Fn(JitsiConference, Participant) -> BoxedResultFuture) + Send + Sync>>,
  on_participant_left:
    Option<Arc<dyn (Fn(JitsiConference, Participant) -> BoxedResultFuture) + Send + Sync>>,
  on_colibri_message:
    Option<Arc<dyn (Fn(JitsiConference, ColibriMessage) -> BoxedResultFuture) + Send + Sync>>,
  presence: Vec<xmpp_parsers::Element>,
  state: JitsiConferenceState,
  send_resolution: Option<i32>,
  connected_tx: Option<oneshot::Sender<()>>,
}

impl fmt::Debug for JitsiConferenceInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("JitsiConferenceInner")
      .field("state", &self.state)
      .finish()
  }
}

impl JitsiConference {
  #[tracing::instrument(level = "debug", err)]
  pub async fn join(
    xmpp_connection: Connection,
    glib_main_context: glib::MainContext,
    config: JitsiConferenceConfig,
  ) -> Result<Self> {
    let conference_stanza = xmpp::jitsi::Conference {
      machine_uid: Uuid::new_v4().to_string(),
      room: config.muc.to_string(),
      properties: hashmap! {
        "stereo".to_string() => config.stereo.to_string(),
        "startBitrate".to_string() => config.start_bitrate.to_string(),
      },
    };

    let (tx, rx) = oneshot::channel();

    let focus = config.focus.clone();

    let ecaps2_hash = ecaps2::hash_ecaps2(&ecaps2::compute_disco(&DISCO_INFO)?, Algo::Sha_256)?;
    let mut presence = vec![
      Muc::new().into(),
      Caps::new(DISCO_NODE, COMPUTED_CAPS_HASH.clone()).into(),
      ECaps2::new(vec![ecaps2_hash]).into(),
      xmpp_parsers::Element::builder("stats-id", ns::DEFAULT_NS)
        .append("gst-meet")
        .build(),
      xmpp_parsers::Element::builder("jitsi_participant_codecType", ns::DEFAULT_NS)
        .append(config.video_codec.as_str())
        .build(),
      xmpp_parsers::Element::builder("audiomuted", ns::DEFAULT_NS)
        .append("false")
        .build(),
      xmpp_parsers::Element::builder("videomuted", ns::DEFAULT_NS)
        .append("false")
        .build(),
      xmpp_parsers::Element::builder("nick", "http://jabber.org/protocol/nick")
        .append(config.nick.as_str())
        .build(),
    ];
    if let Some(region) = &config.region {
      presence.extend([
        xmpp_parsers::Element::builder("jitsi_participant_region", ns::DEFAULT_NS)
          .append(region.as_str())
          .build(),
        xmpp_parsers::Element::builder("region", "http://jitsi.org/jitsi-meet")
          .attr("id", region)
          .build(),
      ]);
    }
    presence.extend(
      config
        .extra_muc_features
        .iter()
        .cloned()
        .map(|var| Feature { var })
        .map(|feature| feature.into()),
    );

    let conference = Self {
      glib_main_context,
      jid: xmpp_connection
        .jid()
        .await
        .context("not connected (no JID)")?,
      xmpp_tx: xmpp_connection.tx.clone(),
      config,
      external_services: xmpp_connection.external_services().await,
      jingle_session: Arc::new(Mutex::new(None)),
      inner: Arc::new(Mutex::new(JitsiConferenceInner {
        state: JitsiConferenceState::Discovering,
        presence,
        participants: HashMap::new(),
        audio_sink: None,
        video_sink: None,
        on_participant: None,
        on_participant_left: None,
        on_colibri_message: None,
        send_resolution: None,
        connected_tx: Some(tx),
      })),
      tls_insecure: xmpp_connection.tls_insecure,
    };

    xmpp_connection.add_stanza_filter(conference.clone()).await;

    let iq = Iq::from_set(generate_id(), conference_stanza).with_to(focus);
    xmpp_connection.tx.send(iq.into()).await?;

    rx.await?;

    Ok(conference)
  }

  #[tracing::instrument(level = "debug", err)]
  pub async fn leave(self) -> Result<()> {
    if let Some(jingle_session) = self.jingle_session.lock().await.take() {
      debug!("pausing all sinks");
      jingle_session.pause_all_sinks();

      debug!("setting pipeline state to NULL");
      if let Err(e) = jingle_session.pipeline().set_state(gstreamer::State::Null) {
        warn!("failed to set pipeline state to NULL: {:?}", e);
      }

      debug!("waiting for state change to complete");
      let _ = jingle_session.pipeline_stopped().await;
    }

    // should leave the XMPP muc gracefully, instead of just disconnecting

    Ok(())
  }

  fn endpoint_id(&self) -> Result<&str> {
    self
      .jid
      .node
      .as_ref()
      .ok_or_else(|| anyhow!("invalid jid"))?
      .split('-')
      .next()
      .ok_or_else(|| anyhow!("invalid jid"))
  }

  fn jid_in_muc(&self) -> Result<FullJid> {
    Ok(self.config.muc.clone().with_resource(self.endpoint_id()?))
  }

  pub(crate) fn focus_jid_in_muc(&self) -> Result<FullJid> {
    Ok(self.config.muc.clone().with_resource("focus"))
  }

  #[tracing::instrument(level = "debug", err)]
  async fn send_presence(&self, payloads: &[xmpp_parsers::Element]) -> Result<()> {
    let mut presence = Presence::new(presence::Type::None).with_to(self.jid_in_muc()?);
    presence.payloads = payloads.to_owned();
    self.xmpp_tx.send(presence.into()).await?;
    Ok(())
  }

  #[tracing::instrument(level = "debug", err)]
  pub async fn set_muted(&self, media_type: MediaType, muted: bool) -> Result<()> {
    let mut locked_inner = self.inner.lock().await;
    let element = xmpp_parsers::Element::builder(
      media_type.jitsi_muted_presence_element_name(),
      ns::DEFAULT_NS,
    )
    .append(muted.to_string())
    .build();
    locked_inner
      .presence
      .retain(|el| el.name() != media_type.jitsi_muted_presence_element_name());
    locked_inner.presence.push(element);
    self.send_presence(&locked_inner.presence).await
  }

  pub async fn pipeline(&self) -> Result<gstreamer::Pipeline> {
    Ok(
      self
        .jingle_session
        .lock()
        .await
        .as_ref()
        .context("not connected (no jingle session)")?
        .pipeline(),
    )
  }

  #[tracing::instrument(level = "debug", err)]
  pub async fn add_bin(&self, bin: &gstreamer::Bin) -> Result<()> {
    let pipeline = self.pipeline().await?;
    pipeline.add(bin)?;
    bin.sync_state_with_parent()?;
    Ok(())
  }

  #[tracing::instrument(level = "debug", err)]
  pub async fn set_pipeline_state(&self, state: gstreamer::State) -> Result<()> {
    self.pipeline().await?.call_async(move |p| {
      if let Err(e) = p.set_state(state) {
        error!("pipeline set_state: {:?}", e);
      }
    });
    Ok(())
  }

  pub async fn remote_participant_audio_sink_element(&self) -> Option<gstreamer::Element> {
    self.inner.lock().await.audio_sink.as_ref().cloned()
  }

  pub async fn set_remote_participant_audio_sink_element(&self, sink: Option<gstreamer::Element>) {
    self.inner.lock().await.audio_sink = sink;
  }

  pub async fn remote_participant_video_sink_element(&self) -> Option<gstreamer::Element> {
    self.inner.lock().await.video_sink.as_ref().cloned()
  }

  pub async fn set_remote_participant_video_sink_element(&self, sink: Option<gstreamer::Element>) {
    self.inner.lock().await.video_sink = sink;
  }

  pub async fn audio_sink_element(&self) -> Result<gstreamer::Element> {
    Ok(
      self
        .jingle_session
        .lock()
        .await
        .as_ref()
        .context("not connected (no jingle session)")?
        .audio_sink_element(),
    )
  }

  pub async fn video_sink_element(&self) -> Result<gstreamer::Element> {
    Ok(
      self
        .jingle_session
        .lock()
        .await
        .as_ref()
        .context("not connected (no jingle session)")?
        .video_sink_element(),
    )
  }

  /// Set the max resolution that we are currently sending.
  ///
  /// Setting this is required for browser clients in the same conference to display
  /// the stats that we broadcast.
  ///
  /// Note that lib-gst-meet does not encode video (that is the responsibility of your
  /// GStreamer pipeline), so this is purely informational.
  pub async fn set_send_resolution(&self, height: i32) {
    self.inner.lock().await.send_resolution = Some(height);
  }

  pub async fn send_colibri_message(&self, message: ColibriMessage) -> Result<()> {
    self
      .jingle_session
      .lock()
      .await
      .as_ref()
      .context("not connected (no jingle session)")?
      .colibri_channel
      .as_ref()
      .context("no colibri channel")?
      .send(message)
      .await
  }

  pub async fn send_json_message<T: Serialize>(&self, payload: &T) -> Result<()> {
    let message = Message {
      from: Some(Jid::Full(self.jid.clone())),
      to: Some(Jid::Bare(self.config.muc.clone())),
      id: Some(Uuid::new_v4().to_string()),
      type_: MessageType::Groupchat,
      bodies: Default::default(),
      subjects: Default::default(),
      thread: None,
      payloads: vec![xmpp_parsers::Element::try_from(xmpp::jitsi::JsonMessage {
        payload: serde_json::to_value(payload)?,
      })?],
    };
    self.xmpp_tx.send(message.into()).await?;
    Ok(())
  }

  pub(crate) async fn ensure_participant(&self, id: &str) -> Result<()> {
    if !self.inner.lock().await.participants.contains_key(id) {
      let participant = Participant {
        jid: None,
        muc_jid: self.config.muc.clone().with_resource(id),
        nick: None,
      };
      self
        .inner
        .lock()
        .await
        .participants
        .insert(id.to_owned(), participant.clone());
      if let Some(f) = self.inner.lock().await.on_participant.as_ref().cloned() {
        if let Err(e) = f(self.clone(), participant.clone()).await {
          warn!("on_participant failed: {:?}", e);
        }
        else if let Ok(pipeline) = self.pipeline().await {
          gstreamer::debug_bin_to_dot_file(
            &pipeline,
            gstreamer::DebugGraphDetails::ALL,
            &format!("participant-added-{}", participant.muc_jid.resource),
          );
        }
      }
    }
    Ok(())
  }

  #[tracing::instrument(level = "trace", skip(f))]
  pub async fn on_participant(
    &self,
    f: impl (Fn(JitsiConference, Participant) -> BoxedResultFuture) + Send + Sync + 'static,
  ) {
    let f = Arc::new(f);
    let f2 = f.clone();
    let existing_participants: Vec<_> = {
      let mut locked_inner = self.inner.lock().await;
      locked_inner.on_participant = Some(f2);
      locked_inner.participants.values().cloned().collect()
    };
    for participant in existing_participants {
      debug!(
        "calling on_participant with existing participant: {:?}",
        participant
      );
      if let Err(e) = f(self.clone(), participant.clone()).await {
        warn!("on_participant failed: {:?}", e);
      }
      else if let Ok(pipeline) = self.pipeline().await {
        gstreamer::debug_bin_to_dot_file(
          &pipeline,
          gstreamer::DebugGraphDetails::ALL,
          &format!("participant-added-{}", participant.muc_jid.resource),
        );
      }
    }
  }

  #[tracing::instrument(level = "trace", skip(f))]
  pub async fn on_participant_left(
    &self,
    f: impl (Fn(JitsiConference, Participant) -> BoxedResultFuture) + Send + Sync + 'static,
  ) {
    self.inner.lock().await.on_participant_left = Some(Arc::new(f));
  }

  #[tracing::instrument(level = "trace", skip(f))]
  pub async fn on_colibri_message(
    &self,
    f: impl (Fn(JitsiConference, ColibriMessage) -> BoxedResultFuture) + Send + Sync + 'static,
  ) {
    self.inner.lock().await.on_colibri_message = Some(Arc::new(f));
  }
}

#[async_trait]
impl StanzaFilter for JitsiConference {
  #[tracing::instrument(level = "trace")]
  fn filter(&self, element: &xmpp_parsers::Element) -> bool {
    element.attr("from") == Some(self.config.focus.to_string().as_str())
      && element.is("iq", ns::DEFAULT_NS)
      || element
        .attr("from")
        .and_then(|from| from.parse::<BareJid>().ok())
        .map(|jid| jid == self.config.muc)
        .unwrap_or_default()
        && (element.is("presence", ns::DEFAULT_NS) || element.is("iq", ns::DEFAULT_NS))
  }

  #[tracing::instrument(level = "trace", err)]
  async fn take(&self, element: xmpp_parsers::Element) -> Result<()> {
    use JitsiConferenceState::*;
    let state = self.inner.lock().await.state;
    match state {
      Discovering => {
        let iq = Iq::try_from(element)?;
        if let IqType::Result(Some(element)) = iq.payload {
          let ready: bool = element
            .attr("ready")
            .ok_or_else(|| anyhow!("missing ready attribute on conference IQ"))?
            .parse()?;
          if !ready {
            bail!("focus reports room not ready");
          }
        }
        else {
          bail!("focus IQ failed");
        };

        let mut locked_inner = self.inner.lock().await;
        self.send_presence(&locked_inner.presence).await?;
        locked_inner.state = JoiningMuc;
      },
      JoiningMuc => {
        let presence = Presence::try_from(element)?;
        if let Some(payload) = presence
          .payloads
          .iter()
          .find(|payload| payload.is("x", ns::MUC_USER))
        {
          let muc_user = MucUser::try_from(payload.clone())?;
          if muc_user.status.contains(&MucStatus::SelfPresence) {
            debug!("Joined MUC: {}", self.config.muc);
            self.inner.lock().await.state = Idle;
          }
        }
      },
      Idle => {
        if let Ok(iq) = Iq::try_from(element.clone()) {
          match iq.payload {
            IqType::Get(element) => {
              if let Ok(query) = DiscoInfoQuery::try_from(element) {
                debug!(
                  "Received disco info query from {} for node {:?}",
                  iq.from.as_ref().unwrap(),
                  query.node
                );
                if let Some(node) = query.node {
                  match node.splitn(2, '#').collect::<Vec<_>>().as_slice() {
                    // TODO: also support ecaps2, as we send it in our presence.
                    [uri, hash]
                      if *uri == DISCO_NODE && *hash == COMPUTED_CAPS_HASH.to_base64() =>
                    {
                      let mut disco_info = DISCO_INFO.clone();
                      disco_info.node = Some(node);
                      let iq = Iq::from_result(iq.id, Some(disco_info))
                        .with_from(Jid::Full(self.jid.clone()))
                        .with_to(iq.from.unwrap());
                      self.xmpp_tx.send(iq.into()).await?;
                    },
                    _ => {
                      let error = StanzaError::new(
                        ErrorType::Cancel,
                        DefinedCondition::ItemNotFound,
                        "en",
                        format!("Unknown disco#info node: {}", node),
                      );
                      let iq = Iq::from_error(iq.id, error)
                        .with_from(Jid::Full(self.jid.clone()))
                        .with_to(iq.from.unwrap());
                      self.xmpp_tx.send(iq.into()).await?;
                    },
                  }
                }
                else {
                  let iq = Iq::from_result(iq.id, Some(DISCO_INFO.clone()))
                    .with_from(Jid::Full(self.jid.clone()))
                    .with_to(iq.from.unwrap());
                  self.xmpp_tx.send(iq.into()).await?;
                }
              }
            },
            IqType::Set(element) => match Jingle::try_from(element) {
              Ok(jingle) => {
                if let Some(Jid::Full(from_jid)) = iq.from {
                  if jingle.action == Action::SessionInitiate {
                    if from_jid.resource == "focus" {
                      // Acknowledge the IQ
                      let result_iq = Iq::empty_result(Jid::Full(from_jid.clone()), iq.id.clone())
                        .with_from(Jid::Full(self.jid.clone()));
                      self.xmpp_tx.send(result_iq.into()).await?;

                      *self.jingle_session.lock().await =
                        Some(JingleSession::initiate(self, jingle).await?);
                    }
                    else {
                      debug!("Ignored Jingle session-initiate from {}", from_jid);
                    }
                  }
                  else if jingle.action == Action::SourceAdd {
                    debug!("Received Jingle source-add");

                    // Acknowledge the IQ
                    let result_iq = Iq::empty_result(Jid::Full(from_jid.clone()), iq.id.clone())
                      .with_from(Jid::Full(self.jid.clone()));
                    self.xmpp_tx.send(result_iq.into()).await?;

                    self
                      .jingle_session
                      .lock()
                      .await
                      .as_mut()
                      .context("not connected (no jingle session")?
                      .source_add(jingle)
                      .await?;
                  }
                }
                else {
                  debug!("Received Jingle IQ from invalid JID: {:?}", iq.from);
                }
              },
              Err(e) => debug!("IQ did not successfully parse as Jingle: {:?}", e),
            },
            IqType::Result(_) => {
              if let Some(jingle_session) = self.jingle_session.lock().await.as_mut() {
                if Some(iq.id) == jingle_session.accept_iq_id {
                  let colibri_url = jingle_session.colibri_url.clone();

                  jingle_session.accept_iq_id = None;

                  debug!("Focus acknowledged session-accept");

                  if let Some(colibri_url) = colibri_url {
                    info!("Connecting Colibri WebSocket to {}", colibri_url);
                    let colibri_channel =
                      ColibriChannel::new(&colibri_url, self.tls_insecure).await?;
                    let (tx, rx) = mpsc::channel(8);
                    colibri_channel.subscribe(tx).await;
                    jingle_session.colibri_channel = Some(colibri_channel.clone());

                    let my_endpoint_id = self.endpoint_id()?.to_owned();

                    {
                      let my_endpoint_id = my_endpoint_id.clone();
                      let colibri_channel = colibri_channel.clone();
                      let self_ = self.clone();
                      jingle_session.stats_handler_task = Some(tokio::spawn(async move {
                        let mut interval = time::interval(SEND_STATS_INTERVAL);
                        loop {
                          let maybe_remote_ssrc_map = self_
                            .jingle_session
                            .lock()
                            .await
                            .as_ref()
                            .map(|sess| sess.remote_ssrc_map.clone());
                          let maybe_source_stats: Option<Vec<gstreamer::Structure>> = self_
                            .pipeline()
                            .await
                            .ok()
                            .and_then(|pipeline| pipeline.by_name("rtpbin"))
                            .and_then(|rtpbin| {
                              rtpbin.try_emit_by_name("get-session", &[&0u32]).ok()
                            })
                            .and_then(|rtpsession: gstreamer::Element| {
                              rtpsession.try_property("stats").ok()
                            })
                            .and_then(|stats: gstreamer::Structure| stats.get("source-stats").ok())
                            .and_then(|stats: glib::ValueArray| {
                              stats
                                .into_iter()
                                .map(|v| v.get())
                                .collect::<Result<_, _>>()
                                .ok()
                            });

                          if let (Some(remote_ssrc_map), Some(source_stats)) =
                            (maybe_remote_ssrc_map, maybe_source_stats)
                          {
                            debug!("source stats: {:#?}", source_stats);

                            let audio_recv_bitrate: u64 = source_stats
                              .iter()
                              .filter(|stat| {
                                stat
                                  .get("ssrc")
                                  .ok()
                                  .and_then(|ssrc: u32| remote_ssrc_map.get(&ssrc))
                                  .map(|source| {
                                    source.media_type == MediaType::Audio
                                      && source
                                        .participant_id
                                        .as_ref()
                                        .map(|id| id != &my_endpoint_id)
                                        .unwrap_or_default()
                                  })
                                  .unwrap_or_default()
                              })
                              .filter_map(|stat| stat.get::<u64>("bitrate").ok())
                              .sum();

                            let video_recv_bitrate: u64 = source_stats
                              .iter()
                              .filter(|stat| {
                                stat
                                  .get("ssrc")
                                  .ok()
                                  .and_then(|ssrc: u32| remote_ssrc_map.get(&ssrc))
                                  .map(|source| {
                                    source.media_type == MediaType::Video
                                      && source
                                        .participant_id
                                        .as_ref()
                                        .map(|id| id != &my_endpoint_id)
                                        .unwrap_or_default()
                                  })
                                  .unwrap_or_default()
                              })
                              .filter_map(|stat| stat.get::<u64>("bitrate").ok())
                              .sum();

                            let audio_send_bitrate: u64 = source_stats
                              .iter()
                              .find(|stat| {
                                stat
                                  .get("ssrc")
                                  .ok()
                                  .and_then(|ssrc: u32| remote_ssrc_map.get(&ssrc))
                                  .map(|source| {
                                    source.media_type == MediaType::Audio
                                      && source
                                        .participant_id
                                        .as_ref()
                                        .map(|id| id == &my_endpoint_id)
                                        .unwrap_or_default()
                                  })
                                  .unwrap_or_default()
                              })
                              .and_then(|stat| stat.get("bitrate").ok())
                              .unwrap_or_default();
                            let video_send_bitrate: u64 = source_stats
                              .iter()
                              .find(|stat| {
                                stat
                                  .get("ssrc")
                                  .ok()
                                  .and_then(|ssrc: u32| remote_ssrc_map.get(&ssrc))
                                  .map(|source| {
                                    source.media_type == MediaType::Video
                                      && source
                                        .participant_id
                                        .as_ref()
                                        .map(|id| id == &my_endpoint_id)
                                        .unwrap_or_default()
                                  })
                                  .unwrap_or_default()
                              })
                              .and_then(|stat| stat.get("bitrate").ok())
                              .unwrap_or_default();

                            let recv_packets: u64 = source_stats
                              .iter()
                              .filter(|stat| {
                                stat
                                  .get("ssrc")
                                  .ok()
                                  .and_then(|ssrc: u32| remote_ssrc_map.get(&ssrc))
                                  .map(|source| {
                                    source
                                      .participant_id
                                      .as_ref()
                                      .map(|id| id != &my_endpoint_id)
                                      .unwrap_or_default()
                                  })
                                  .unwrap_or_default()
                              })
                              .filter_map(|stat| stat.get::<u64>("packets-received").ok())
                              .sum();
                            let recv_lost: u64 = source_stats
                              .iter()
                              .filter(|stat| {
                                stat
                                  .get("ssrc")
                                  .ok()
                                  .and_then(|ssrc: u32| remote_ssrc_map.get(&ssrc))
                                  .map(|source| source.participant_id.as_ref().map(|id| id != &my_endpoint_id).unwrap_or_default())
                                  .unwrap_or_default()
                              })
                              .filter_map(|stat| stat.get::<i32>("packets-lost").ok())
                              .sum::<i32>()
                              // Loss can be negative because of duplicate packets. Clamp it to zero.
                              .try_into()
                              .unwrap_or_default();
                            let recv_loss =
                              recv_lost as f64 / (recv_packets as f64 + recv_lost as f64);

                            let stats = ColibriMessage::EndpointStats {
                              from: None,
                              bitrate: colibri::Bitrates {
                                audio: colibri::Bitrate {
                                  upload: audio_send_bitrate / 1024,
                                  download: audio_recv_bitrate / 1024,
                                },
                                video: colibri::Bitrate {
                                  upload: video_send_bitrate / 1024,
                                  download: video_recv_bitrate / 1024,
                                },
                                total: colibri::Bitrate {
                                  upload: (audio_send_bitrate + video_send_bitrate) / 1024,
                                  download: (audio_recv_bitrate + video_recv_bitrate) / 1024,
                                },
                              },
                              packet_loss: colibri::PacketLoss {
                                total: (recv_loss * 100.) as u64,
                                download: (recv_loss * 100.) as u64,
                                upload: 0, // TODO
                              },
                              connection_quality: 100.0,
                              jvb_rtt: Some(0), // TODO
                              server_region: self_.config.region.clone(),
                              max_enabled_resolution: self_.inner.lock().await.send_resolution,
                            };
                            if let Err(e) = colibri_channel.send(stats).await {
                              warn!("failed to send stats: {:?}", e);
                            }
                          }
                          else {
                            warn!("unable to get stats from pipeline");
                          }
                          interval.tick().await;
                        }
                      }));
                    }

                    {
                      let self_ = self.clone();
                      tokio::spawn(async move {
                        let mut stream = ReceiverStream::new(rx);
                        while let Some(msg) = stream.next().await {
                          // Some message types are handled internally rather than passed to the on_colibri_message handler.
                          let handled = match &msg {
                            ColibriMessage::EndpointMessage {
                              to: Some(to),
                              from,
                              msg_payload,
                            } if to == &my_endpoint_id => {
                              match serde_json::from_value::<JsonMessage>(msg_payload.clone()) {
                                Ok(JsonMessage::E2ePingRequest { id }) => {
                                  if let Err(e) = colibri_channel
                                    .send(ColibriMessage::EndpointMessage {
                                      from: None,
                                      to: from.clone(),
                                      msg_payload: serde_json::to_value(
                                        JsonMessage::E2ePingResponse { id },
                                      )
                                      .unwrap(),
                                    })
                                    .await
                                  {
                                    warn!("failed to send e2e ping response: {:?}", e);
                                  }
                                  true
                                },
                                _ => false,
                              }
                            },
                            _ => false,
                          };

                          if handled {
                            continue;
                          }

                          let locked_inner = self_.inner.lock().await;
                          if let Some(f) = &locked_inner.on_colibri_message {
                            if let Err(e) = f(self_.clone(), msg).await {
                              warn!("on_colibri_message failed: {:?}", e);
                            }
                          }
                        }
                        Ok::<_, anyhow::Error>(())
                      });
                    }
                  }

                  if let Some(connected_tx) = self.inner.lock().await.connected_tx.take() {
                    connected_tx.send(()).unwrap();
                  }
                }
              }
            },
            _ => {},
          }
        }
        else if let Ok(presence) = Presence::try_from(element) {
          if let Jid::Full(from) = presence
            .from
            .as_ref()
            .context("missing from in presence")?
            .clone()
          {
            let bare_from: BareJid = from.clone().into();
            if bare_from == self.config.muc && from.resource != "focus" {
              trace!("received MUC presence from {}", from.resource);
              let nick_payload = presence
                .payloads
                .iter()
                .find(|e| e.is("nick", ns::NICK))
                .map(|e| Nick::try_from(e.clone()))
                .transpose()?;
              if let Some(muc_user_payload) = presence
                .payloads
                .into_iter()
                .find(|e| e.is("x", ns::MUC_USER))
              {
                let muc_user = MucUser::try_from(muc_user_payload)?;
                for item in muc_user.items {
                  if let Some(jid) = &item.jid {
                    if jid == &self.jid {
                      continue;
                    }
                    let participant = Participant {
                      jid: Some(jid.clone()),
                      muc_jid: from.clone(),
                      nick: item
                        .nick
                        .or_else(|| nick_payload.as_ref().map(|nick| nick.0.clone())),
                    };
                    if presence.type_ == presence::Type::Unavailable
                      && self
                        .inner
                        .lock()
                        .await
                        .participants
                        .remove(&from.resource.clone())
                        .is_some()
                    {
                      debug!("participant left: {:?}", jid);
                      if let Some(f) = &self
                        .inner
                        .lock()
                        .await
                        .on_participant_left
                        .as_ref()
                        .cloned()
                      {
                        debug!("calling on_participant_left with old participant");
                        if let Err(e) = f(self.clone(), participant).await {
                          warn!("on_participant_left failed: {:?}", e);
                        }
                      }
                    }
                    else if self
                      .inner
                      .lock()
                      .await
                      .participants
                      .insert(from.resource.clone(), participant.clone())
                      .is_none()
                    {
                      debug!("new participant: {:?}", jid);
                      if let Some(f) = &self.inner.lock().await.on_participant.as_ref().cloned() {
                        debug!("calling on_participant with new participant");
                        if let Err(e) = f(self.clone(), participant.clone()).await {
                          warn!("on_participant failed: {:?}", e);
                        }
                        else if let Some(jingle_session) =
                          self.jingle_session.lock().await.as_ref()
                        {
                          gstreamer::debug_bin_to_dot_file(
                            &jingle_session.pipeline(),
                            gstreamer::DebugGraphDetails::ALL,
                            &format!("participant-added-{}", participant.muc_jid.resource),
                          );
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      },
    }
    Ok(())
  }
}
