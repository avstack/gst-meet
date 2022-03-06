use std::{collections::HashMap, convert::TryFrom, fmt, future::Future, pin::Pin, sync::Arc};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use colibri::ColibriMessage;
use futures::stream::StreamExt;
use gstreamer::prelude::{ElementExt, ElementExtManual, GstBinExt};
use jitsi_xmpp_parsers::jingle::{Action, Jingle};
use maplit::hashmap;
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;
pub use xmpp_parsers::disco::Feature;
use xmpp_parsers::{
  disco::{DiscoInfoQuery, DiscoInfoResult, Identity},
  caps::{self, Caps},
  ecaps2::{self, ECaps2},
  hashes::{Algo, Hash},
  iq::{Iq, IqType},
  message::{Message, MessageType},
  muc::{Muc, MucUser, user::Status as MucStatus},
  nick::Nick,
  ns,
  presence::{self, Presence},
  stanza_error::{DefinedCondition, ErrorType, StanzaError},
  BareJid, Element, FullJid, Jid,
};

use crate::{
  colibri::ColibriChannel,
  jingle::JingleSession,
  source::MediaType,
  stanza_filter::StanzaFilter,
  util::generate_id,
  xmpp::{self, connection::Connection},
};

const DISCO_NODE: &str = "https://github.com/avstack/gst-meet";

static DISCO_INFO: Lazy<DiscoInfoResult> = Lazy::new(|| DiscoInfoResult {
  node: None,
  identities: vec![
    Identity::new("client", "bot", "en", "gst-meet"),
  ],
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

static COMPUTED_CAPS_HASH: Lazy<Hash> = Lazy::new(|| {
  caps::hash_caps(&caps::compute_disco(&DISCO_INFO), Algo::Sha_1).unwrap()
});

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
}

#[derive(Clone)]
pub struct JitsiConference {
  pub(crate) glib_main_context: glib::MainContext,
  pub(crate) jid: FullJid,
  pub(crate) xmpp_tx: mpsc::Sender<Element>,
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
  on_participant:
    Option<Arc<dyn (Fn(JitsiConference, Participant) -> BoxedResultFuture) + Send + Sync>>,
  on_participant_left:
    Option<Arc<dyn (Fn(JitsiConference, Participant) -> BoxedResultFuture) + Send + Sync>>,
  on_colibri_message:
    Option<Arc<dyn (Fn(JitsiConference, ColibriMessage) -> BoxedResultFuture) + Send + Sync>>,
  presence: Vec<Element>,
  state: JitsiConferenceState,
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
        // Disable voice processing
        // TODO put this in config
        "stereo".to_string() => "true".to_string(),
        "startBitrate".to_string() => "800".to_string(),
      },
    };

    let (tx, rx) = oneshot::channel();

    let focus = config.focus.clone();

    let ecaps2_hash =
      ecaps2::hash_ecaps2(&ecaps2::compute_disco(&DISCO_INFO)?, Algo::Sha_256)?;
    let mut presence = vec![
      Muc::new().into(),
      Caps::new(DISCO_NODE, COMPUTED_CAPS_HASH.clone()).into(),
      ECaps2::new(vec![ecaps2_hash]).into(),
      Element::builder("stats-id", ns::DEFAULT_NS).append("gst-meet").build(),
      Element::builder("jitsi_participant_codecType", ns::DEFAULT_NS)
        .append(config.video_codec.as_str())
        .build(),
      Element::builder("audiomuted", ns::DEFAULT_NS).append("false").build(),
      Element::builder("videomuted", ns::DEFAULT_NS).append("false").build(),
      Element::builder("nick", "http://jabber.org/protocol/nick")
        .append(config.nick.as_str())
        .build(),
    ];
    if let Some(region) = &config.region {
      presence.extend([
        Element::builder("jitsi_participant_region", ns::DEFAULT_NS)
          .append(region.as_str())
          .build(),
        Element::builder("region", "http://jitsi.org/jitsi-meet")
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
        on_participant: None,
        on_participant_left: None,
        on_colibri_message: None,
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

  fn jid_in_muc(&self) -> Result<FullJid> {
    let resource = self
      .jid
      .node
      .as_ref()
      .ok_or_else(|| anyhow!("invalid jid"))?
      .split('-')
      .next()
      .ok_or_else(|| anyhow!("invalid jid"))?;
    Ok(self.config.muc.clone().with_resource(resource))
  }

  pub(crate) fn focus_jid_in_muc(&self) -> Result<FullJid> {
    Ok(self.config.muc.clone().with_resource("focus"))
  }

  #[tracing::instrument(level = "debug", err)]
  async fn send_presence(&self, payloads: &[Element]) -> Result<()> {
    let mut presence = Presence::new(presence::Type::None).with_to(self.jid_in_muc()?);
    presence.payloads = payloads.to_owned();
    self.xmpp_tx.send(presence.into()).await?;
    Ok(())
  }

  #[tracing::instrument(level = "debug", err)]
  pub async fn set_muted(&self, media_type: MediaType, muted: bool) -> Result<()> {
    let mut locked_inner = self.inner.lock().await;
    let element = Element::builder(media_type.jitsi_muted_presence_element_name(), ns::DEFAULT_NS)
      .append(muted.to_string())
      .build();
    locked_inner.presence.retain(|el| el.name() != media_type.jitsi_muted_presence_element_name());
    locked_inner.presence.push(element);
    self
      .send_presence(&locked_inner.presence)
      .await
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
      payloads: vec![Element::try_from(xmpp::jitsi::JsonMessage {
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
  fn filter(&self, element: &Element) -> bool {
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
  async fn take(&self, element: Element) -> Result<()> {
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
        if let Some(payload) = presence.payloads.iter().find(|payload| payload.is("x", ns::MUC_USER)) {
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
                    [uri, hash] if *uri == DISCO_NODE && *hash == COMPUTED_CAPS_HASH.to_base64() => {
                      let mut disco_info = DISCO_INFO.clone();
                      disco_info.node = Some(node);
                      let iq = Iq::from_result(iq.id, Some(disco_info))
                        .with_from(Jid::Full(self.jid.clone()))
                        .with_to(iq.from.unwrap());
                      self.xmpp_tx.send(iq.into()).await?;
                    }
                    _ => {
                      let error = StanzaError::new(
                        ErrorType::Cancel, DefinedCondition::ItemNotFound,
                        "en", format!("Unknown disco#info node: {}", node));
                      let iq = Iq::from_error(iq.id, error)
                        .with_from(Jid::Full(self.jid.clone()))
                        .with_to(iq.from.unwrap());
                      self.xmpp_tx.send(iq.into()).await?;
                    }
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
                    let colibri_channel = ColibriChannel::new(&colibri_url, self.tls_insecure).await?;
                    let (tx, rx) = mpsc::channel(8);
                    colibri_channel.subscribe(tx).await;
                    jingle_session.colibri_channel = Some(colibri_channel);

                    let self_ = self.clone();
                    tokio::spawn(async move {
                      let mut stream = ReceiverStream::new(rx);
                      while let Some(msg) = stream.next().await {
                        // Some message types are handled internally rather than passed to the on_colibri_message handler.

                        // End-to-end ping
                        if let ColibriMessage::EndpointMessage { to, .. } = &msg {
                          // if to == 

                        }

                        let locked_inner = self_.inner.lock().await;
                        if let Some(f) = &locked_inner.on_colibri_message {
                          if let Err(e) = f(self_.clone(), msg).await {
                            warn!("on_colibri_message failed: {:?}", e);
                          }
                        }
                      }
                    });
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
                        else if let Some(jingle_session) = self.jingle_session.lock().await.as_ref() {
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
