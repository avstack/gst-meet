use std::{collections::HashMap, convert::TryFrom, fmt, future::Future, pin::Pin, sync::Arc};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use futures::stream::StreamExt;
use glib::ObjectExt;
use gstreamer::prelude::{ElementExt, ElementExtManual, GstBinExt};
use once_cell::sync::Lazy;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, trace, warn};
use xmpp_parsers::{
  disco::{DiscoInfoQuery, DiscoInfoResult, Feature},
  ecaps2::{self, ECaps2},
  hashes::Algo,
  iq::{Iq, IqType},
  jingle::{Action, Jingle},
  muc::{Muc, MucUser},
  ns,
  presence::{self, Presence},
  BareJid, Element, FullJid, Jid,
};

use crate::{jingle::JingleSession, source::MediaType, stanza_filter::StanzaFilter, xmpp};

static DISCO_INFO: Lazy<DiscoInfoResult> = Lazy::new(|| DiscoInfoResult {
  node: None,
  identities: vec![],
  features: vec![
    Feature::new(ns::JINGLE_RTP_AUDIO),
    Feature::new(ns::JINGLE_RTP_VIDEO),
    Feature::new(ns::JINGLE_ICE_UDP),
    Feature::new(ns::JINGLE_DTLS),
    // not supported yet: rtx
    // Feature::new("urn:ietf:rfc:4588"),

    // not supported yet: rtcp remb
    // Feature::new("http://jitsi.org/remb"),

    // not supported yet: transport-cc
    // Feature::new("http://jitsi.org/tcc"),

    // rtcp-mux
    Feature::new("urn:ietf:rfc:5761"),
    // rtp-bundle
    Feature::new("urn:ietf:rfc:5888"),
    // opus red
    Feature::new("http://jitsi.org/opus-red"),
  ],
  extensions: vec![],
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
  pub focus: FullJid,
  pub nick: String,
  pub region: String,
  pub video_codec: String,
}

#[derive(Clone)]
pub struct JitsiConference {
  pub(crate) glib_main_context: glib::MainContext,
  pub(crate) jid: FullJid,
  pub(crate) xmpp_tx: mpsc::Sender<Element>,
  pub(crate) config: JitsiConferenceConfig,
  pub(crate) external_services: Vec<xmpp::extdisco::Service>,
  pub(crate) inner: Arc<Mutex<JitsiConferenceInner>>,
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
  pub jid: FullJid,
  pub muc_jid: FullJid,
  pub nick: Option<String>,
  bin: Option<gstreamer::Bin>,
}

type BoxedBinResultFuture = Pin<Box<dyn Future<Output = Result<Option<gstreamer::Bin>>> + Send>>;
type BoxedResultFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

pub(crate) struct JitsiConferenceInner {
  pub(crate) jingle_session: Option<JingleSession>,
  participants: HashMap<String, Participant>,
  on_participant: Option<Arc<dyn (Fn(Participant) -> BoxedBinResultFuture) + Send + Sync>>,
  on_participant_left: Option<Arc<dyn (Fn(Participant) -> BoxedResultFuture) + Send + Sync>>,
  state: JitsiConferenceState,
  connected_tx: Option<oneshot::Sender<()>>,
  connected_rx: Option<oneshot::Receiver<()>>,
}

impl fmt::Debug for JitsiConferenceInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("JitsiConferenceInner")
      .field("state", &self.state)
      .finish()
  }
}

impl JitsiConference {
  #[tracing::instrument(level = "debug", skip(xmpp_tx), err)]
  pub(crate) async fn new(
    glib_main_context: glib::MainContext,
    jid: FullJid,
    xmpp_tx: mpsc::Sender<Element>,
    config: JitsiConferenceConfig,
    external_services: Vec<xmpp::extdisco::Service>,
  ) -> Result<Self> {
    let (tx, rx) = oneshot::channel();
    Ok(Self {
      glib_main_context,
      jid,
      xmpp_tx,
      config,
      external_services,
      inner: Arc::new(Mutex::new(JitsiConferenceInner {
        state: JitsiConferenceState::Discovering,
        participants: HashMap::new(),
        on_participant: None,
        on_participant_left: None,
        jingle_session: None,
        connected_tx: Some(tx),
        connected_rx: Some(rx),
      })),
    })
  }

  pub async fn connected(&self) -> Result<()> {
    let rx = {
      let mut locked_inner = self.inner.lock().await;
      locked_inner
        .connected_rx
        .take()
        .context("connected() called twice")?
    };
    rx.await?;
    Ok(())
  }

  #[tracing::instrument(level = "debug", err)]
  pub async fn leave(self) -> Result<()> {
    let mut inner = self.inner.lock().await;

    if let Some(jingle_session) = inner.jingle_session.take() {
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
  async fn send_presence(&self, payloads: Vec<Element>) -> Result<()> {
    let mut presence = Presence::new(presence::Type::None).with_to(self.jid_in_muc()?);
    presence.payloads = payloads;
    self.xmpp_tx.send(presence.into()).await?;
    Ok(())
  }

  #[tracing::instrument(level = "debug", err)]
  pub async fn set_muted(&self, media_type: MediaType, muted: bool) -> Result<()> {
    self
      .send_presence(vec![
        Element::builder(media_type.jitsi_muted_presence_element_name(), "")
          .append(muted.to_string())
          .build(),
      ])
      .await
  }

  pub async fn pipeline(&self) -> Result<gstreamer::Pipeline> {
    Ok(
      self
        .inner
        .lock()
        .await
        .jingle_session
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
        .inner
        .lock()
        .await
        .jingle_session
        .as_ref()
        .context("not connected (no jingle session)")?
        .audio_sink_element(),
    )
  }

  pub async fn video_sink_element(&self) -> Result<gstreamer::Element> {
    Ok(
      self
        .inner
        .lock()
        .await
        .jingle_session
        .as_ref()
        .context("not connected (no jingle session)")?
        .video_sink_element(),
    )
  }

  #[tracing::instrument(level = "trace", skip(f))]
  pub async fn on_participant(&self, f: impl (Fn(Participant) -> BoxedBinResultFuture) + Send + Sync + 'static) {
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
      match f(participant.clone()).await {
        Ok(Some(bin)) => {
          bin
            .set_property(
              "name",
              format!("participant_{}", participant.muc_jid.resource),
            )
            .unwrap();
          match self.add_bin(&bin).await {
            Ok(_) => {
              let mut locked_inner = self.inner.lock().await;
              if let Some(p) = locked_inner
                .participants
                .get_mut(&participant.muc_jid.resource)
              {
                p.bin = Some(bin);
              }
            },
            Err(e) => warn!("failed to add participant bin: {:?}", e),
          }
        },
        Ok(None) => {},
        Err(e) => warn!("on_participant failed: {:?}", e),
      }
    }
  }

  #[tracing::instrument(level = "trace", skip(f))]
  pub async fn on_participant_left(&self, f: impl (Fn(Participant) -> BoxedResultFuture) + Send + Sync + 'static) {
    self.inner.lock().await.on_participant_left = Some(Arc::new(f));
  }
}

#[async_trait]
impl StanzaFilter for JitsiConference {
  #[tracing::instrument(level = "trace")]
  fn filter(&self, element: &Element) -> bool {
    element.attr("from") == Some(self.config.focus.to_string().as_str())
      && element.is("iq", "jabber:client")
      || element
        .attr("from")
        .and_then(|from| from.parse::<BareJid>().ok())
        .map(|jid| jid == self.config.muc)
        .unwrap_or_default()
        && (element.is("presence", "jabber:client") || element.is("iq", "jabber:client"))
  }

  #[tracing::instrument(level = "trace", err)]
  async fn take(&self, element: Element) -> Result<()> {
    let mut locked_inner = self.inner.lock().await;

    use JitsiConferenceState::*;
    match locked_inner.state {
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

        let jitsi_disco_info = DiscoInfoResult {
          node: Some("http://jitsi.org/jitsimeet".to_string()),
          identities: vec![],
          features: vec![],
          extensions: vec![],
        };

        let jitsi_disco_hash =
          ecaps2::hash_ecaps2(&ecaps2::compute_disco(&jitsi_disco_info)?, Algo::Sha_256)?;
        self
          .send_presence(vec![
            Muc::new().into(),
            ECaps2::new(vec![jitsi_disco_hash]).into(),
            Element::builder("stats-id", "").append("gst-meet").build(),
            Element::builder("jitsi_participant_codecType", "")
              .append(self.config.video_codec.as_str())
              .build(),
            Element::builder("jitsi_participant_region", "")
              .append(self.config.region.as_str())
              .build(),
            Element::builder("audiomuted", "").append("false").build(),
            Element::builder("videomuted", "").append("false").build(),
            Element::builder("nick", "http://jabber.org/protocol/nick")
              .append(self.config.nick.as_str())
              .build(),
            Element::builder("region", "http://jitsi.org/jitsi-meet")
              .attr("id", &self.config.region)
              .build(),
          ])
          .await?;
        locked_inner.state = JoiningMuc;
      },
      JoiningMuc => {
        let presence = Presence::try_from(element)?;
        if BareJid::from(presence.from.as_ref().unwrap().clone()) == self.config.muc {
          debug!("Joined MUC: {}", self.config.muc);
          locked_inner.state = Idle;
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
                if query.node.is_none() {
                  let iq = Iq::from_result(iq.id, Some(DISCO_INFO.clone()))
                    .with_from(Jid::Full(self.jid.clone()))
                    .with_to(iq.from.unwrap());
                  self.xmpp_tx.send(iq.into()).await?;
                }
                else {
                  panic!("don't know how to handle disco info node: {:?}", query.node);
                }
              }
            },
            IqType::Set(element) => {
              if let Ok(jingle) = Jingle::try_from(element) {
                if let Some(Jid::Full(from_jid)) = iq.from {
                  if jingle.action == Action::SessionInitiate {
                    if from_jid.resource == "focus" {
                      // Acknowledge the IQ
                      let result_iq = Iq::empty_result(Jid::Full(from_jid.clone()), iq.id.clone())
                        .with_from(Jid::Full(self.jid.clone()));
                      self.xmpp_tx.send(result_iq.into()).await?;

                      locked_inner.jingle_session =
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

                    locked_inner
                      .jingle_session
                      .as_mut()
                      .context("not connected (no jingle session")?
                      .source_add(jingle)
                      .await?;
                  }
                }
                else {
                  debug!("Received Jingle IQ from invalid JID: {:?}", iq.from);
                }
              }
              else {
                debug!("Received non-Jingle IQ");
              }
            },
            IqType::Result(_) => {
              if let Some(jingle_session) = locked_inner.jingle_session.as_ref() {
                if Some(iq.id) == jingle_session.accept_iq_id {
                  let colibri_url = jingle_session.colibri_url.clone();

                  locked_inner.jingle_session.as_mut().unwrap().accept_iq_id = None;

                  debug!("Focus acknowledged session-accept");

                  if let Some(colibri_url) = colibri_url {
                    info!("Connecting Colibri WebSocket to {}", colibri_url);

                    let request =
                      tokio_tungstenite::tungstenite::http::Request::get(colibri_url).body(())?;
                    let (colibri_websocket, _response) =
                      tokio_tungstenite::connect_async(request).await?;
                    info!("Connected Colibri WebSocket");

                    let (colibri_sink, mut colibri_stream) = colibri_websocket.split();
                    let colibri_receive_task = tokio::spawn(async move {
                      while let Some(msg) = colibri_stream.next().await {
                        debug!("colibri: {:?}", msg);
                      }
                      Ok::<_, anyhow::Error>(())
                    });
                    let (colibri_tx, colibri_rx) = mpsc::channel(8);
                    locked_inner.jingle_session.as_mut().unwrap().colibri_tx = Some(colibri_tx);
                    let colibri_transmit_task = tokio::spawn(async move {
                      let stream = ReceiverStream::new(colibri_rx);
                      stream.forward(colibri_sink).await?;
                      Ok::<_, anyhow::Error>(())
                    });

                    tokio::spawn(async move {
                      tokio::select! {
                        res = colibri_receive_task => if let Ok(Err(e)) = res {
                          error!("colibri read loop: {:?}", e);
                        },
                        res = colibri_transmit_task => if let Ok(Err(e)) = res {
                          error!("colibri write loop: {:?}", e);
                        },
                      };
                    });
                  }

                  if let Some(connected_tx) = locked_inner.connected_tx.take() {
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
              for payload in presence.payloads {
                if !payload.is("x", ns::MUC_USER) {
                  continue;
                }
                let muc_user = MucUser::try_from(payload)?;
                debug!("MUC user presence: {:?}", muc_user);
                for item in muc_user.items {
                  if let Some(jid) = &item.jid {
                    if jid == &self.jid {
                      continue;
                    }
                    let participant = Participant {
                      jid: jid.clone(),
                      muc_jid: from.clone(),
                      nick: item.nick,
                      bin: None,
                    };
                    if locked_inner
                      .participants
                      .insert(from.resource.clone(), participant.clone())
                      .is_none()
                    {
                      debug!("new participant: {:?}", jid);
                      if let Some(f) = &locked_inner.on_participant {
                        debug!("calling on_participant with new participant");
                        match f(participant).await {
                          Ok(Some(bin)) => {
                            bin.set_property("name", format!("participant_{}", from.resource))?;
                            match self.add_bin(&bin).await {
                              Ok(_) => {
                                if let Some(p) = locked_inner.participants.get_mut(&from.resource) {
                                  p.bin = Some(bin);
                                }
                              },
                              Err(e) => warn!("failed to add participant bin: {:?}", e),
                            }
                          },
                          Ok(None) => {},
                          Err(e) => warn!("on_participant failed: {:?}", e),
                        }
                      }
                    }
                    else if presence.type_ == presence::Type::Unavailable {
                      locked_inner.participants.remove(&from.resource.clone());
                      debug!("participant left: {:?}", jid);
                      if let Some(f) = &locked_inner.on_participant_left {
                        debug!("calling on_participant_left with old participant");
                        if let Err(e) = f(participant).await {
                          warn!("on_participant_left failed: {:?}", e);
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
