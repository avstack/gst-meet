use std::{convert::TryFrom, fmt, future::Future, sync::Arc};

use anyhow::{anyhow, bail, Context, Result};
use futures::{
  sink::{Sink, SinkExt},
  stream::{Stream, StreamExt, TryStreamExt},
};
use maplit::hashmap;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite::{
  http::{Request, Uri},
  Message,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use xmpp_parsers::{
  bind::{BindQuery, BindResponse},
  disco::{DiscoInfoQuery, DiscoInfoResult},
  iq::{Iq, IqType},
  sasl::{Auth, Mechanism, Success},
  websocket::Open,
  BareJid, Element, FullJid, Jid,
};

use crate::{
  conference::{JitsiConference, JitsiConferenceConfig},
  pinger::Pinger,
  stanza_filter::StanzaFilter,
  util::generate_id,
  xmpp,
};

#[derive(Debug, Clone, Copy)]
enum JitsiConnectionState {
  OpeningPreAuthentication,
  ReceivingFeaturesPreAuthentication,
  Authenticating,
  OpeningPostAuthentication,
  ReceivingFeaturesPostAuthentication,
  Binding,
  Discovering,
  DiscoveringExternalServices,
  Idle,
}

struct JitsiConnectionInner {
  state: JitsiConnectionState,
  xmpp_domain: BareJid,
  jid: Option<FullJid>,
  external_services: Vec<xmpp::extdisco::Service>,
  connected_tx: Option<oneshot::Sender<Result<()>>>,
  stanza_filters: Vec<Box<dyn StanzaFilter + Send + Sync>>,
}

impl fmt::Debug for JitsiConnectionInner {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("JitsiConnectionInner")
      .field("state", &self.state)
      .field("xmpp_domain", &self.xmpp_domain)
      .field("jid", &self.jid)
      .finish()
  }
}

#[derive(Debug, Clone)]
pub struct JitsiConnection {
  tx: mpsc::Sender<Element>,
  inner: Arc<Mutex<JitsiConnectionInner>>,
}

impl JitsiConnection {
  pub async fn new(
    websocket_url: &str,
    xmpp_domain: &str,
  ) -> Result<(Self, impl Future<Output = ()>)> {
    let websocket_url: Uri = websocket_url.parse().context("invalid WebSocket URL")?;
    let xmpp_domain: BareJid = xmpp_domain.parse().context("invalid XMPP domain")?;

    info!("Connecting XMPP WebSocket to {}", websocket_url);
    let request = Request::get(websocket_url)
      .header("Sec-Websocket-Protocol", "xmpp")
      .body(())
      .context("failed to build WebSocket request")?;
    let (websocket, _response) = tokio_tungstenite::connect_async(request)
      .await
      .context("failed to connect XMPP WebSocket")?;
    let (sink, stream) = websocket.split();
    let (tx, rx) = mpsc::channel(64);

    let inner = Arc::new(Mutex::new(JitsiConnectionInner {
      state: JitsiConnectionState::OpeningPreAuthentication,
      xmpp_domain,
      jid: None,
      external_services: vec![],
      connected_tx: None,
      stanza_filters: vec![],
    }));

    let connection = Self {
      tx: tx.clone(),
      inner: inner.clone(),
    };

    let writer = JitsiConnection::write_loop(rx, sink);
    let reader = JitsiConnection::read_loop(inner, tx, stream);

    let background = async move {
      tokio::select! {
        res = reader => if let Err(e) = res { error!("fatal (in read loop): {:?}", e) },
        res = writer => if let Err(e) = res { error!("fatal (in write loop): {:?}", e) },
      }
    };

    Ok((connection, background))
  }

  pub async fn connect(&self) -> Result<()> {
    let (tx, rx) = oneshot::channel();

    {
      let mut locked_inner = self.inner.lock().await;
      locked_inner.connected_tx = Some(tx);
      let open = Open::new(locked_inner.xmpp_domain.clone());
      self.tx.send(open.into()).await?;
    }

    rx.await?
  }

  pub async fn join_conference(
    &self,
    glib_main_context: glib::MainContext,
    config: JitsiConferenceConfig,
  ) -> Result<JitsiConference> {
    let conference_stanza = xmpp::jitsi::Conference {
      machine_uid: Uuid::new_v4().to_string(),
      room: config.muc.to_string(),
      properties: hashmap! {
        // Disable voice processing
        "stereo".to_string() => "true".to_string(),
        "startBitrate".to_string() => "800".to_string(),
      },
    };

    let iq =
      Iq::from_set(generate_id(), conference_stanza).with_to(Jid::Full(config.focus.clone()));
    self.tx.send(iq.into()).await?;

    let conference = {
      let mut locked_inner = self.inner.lock().await;
      let conference = JitsiConference::new(
        glib_main_context,
        locked_inner
          .jid
          .as_ref()
          .context("not connected (no jid)")?
          .clone(),
        self.tx.clone(),
        config,
        locked_inner.external_services.clone(),
      )
      .await?;
      locked_inner
        .stanza_filters
        .push(Box::new(conference.clone()));
      conference
    };

    conference.connected().await?;

    Ok(conference)
  }

  async fn write_loop<S>(rx: mpsc::Receiver<Element>, mut sink: S) -> Result<()>
  where
    S: Sink<Message> + Unpin,
    S::Error: std::error::Error + Send + Sync + 'static,
  {
    let mut rx = ReceiverStream::new(rx);
    while let Some(element) = rx.next().await {
      let mut bytes = Vec::new();
      element.write_to(&mut bytes)?;
      let xml = String::from_utf8(bytes)?;
      debug!("XMPP    >>> {}", xml);
      sink.send(Message::Text(xml)).await?;
    }
    Ok(())
  }

  async fn read_loop<S>(
    inner: Arc<Mutex<JitsiConnectionInner>>,
    tx: mpsc::Sender<Element>,
    mut stream: S,
  ) -> Result<()>
  where
    S: Stream<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
  {
    loop {
      let message = stream
        .try_next()
        .await?
        .ok_or_else(|| anyhow!("unexpected EOF"))?;
      let element: Element = match message {
        Message::Text(xml) => {
          debug!("XMPP    <<< {}", xml);
          xml.parse()?
        },
        _ => {
          warn!(
            "unexpected non-text message on XMPP WebSocket stream: {:?}",
            message
          );
          continue;
        },
      };

      let mut locked_inner = inner.lock().await;

      use JitsiConnectionState::*;
      match locked_inner.state {
        OpeningPreAuthentication => {
          Open::try_from(element)?;
          info!("Connected XMPP WebSocket");
          locked_inner.state = ReceivingFeaturesPreAuthentication;
        },
        ReceivingFeaturesPreAuthentication => {
          let auth = Auth {
            mechanism: Mechanism::Anonymous,
            data: vec![],
          };
          tx.send(auth.into()).await?;
          locked_inner.state = Authenticating;
        },
        Authenticating => {
          Success::try_from(element)?;

          let open = Open::new(locked_inner.xmpp_domain.clone());
          tx.send(open.into()).await?;
          locked_inner.state = OpeningPostAuthentication;
        },
        OpeningPostAuthentication => {
          Open::try_from(element)?;
          info!("Logged in anonymously");

          locked_inner.state = ReceivingFeaturesPostAuthentication;
        },
        ReceivingFeaturesPostAuthentication => {
          let iq = Iq::from_set(generate_id(), BindQuery::new(None));
          tx.send(iq.into()).await?;
          locked_inner.state = Binding;
        },
        Binding => {
          let iq = Iq::try_from(element)?;
          let jid = if let IqType::Result(Some(element)) = iq.payload {
            let bind = BindResponse::try_from(element)?;
            FullJid::try_from(bind)?
          }
          else {
            bail!("bind failed");
          };
          info!("My JID: {}", jid);
          locked_inner.jid = Some(jid.clone());

          locked_inner.stanza_filters.push(Box::new(Pinger {
            jid: jid.clone(),
            tx: tx.clone(),
          }));

          let iq = Iq::from_get(generate_id(), DiscoInfoQuery { node: None })
            .with_from(Jid::Full(jid.clone()))
            .with_to(Jid::Bare(locked_inner.xmpp_domain.clone()));
          tx.send(iq.into()).await?;
          locked_inner.state = Discovering;
        },
        Discovering => {
          let iq = Iq::try_from(element)?;
          if let IqType::Result(Some(element)) = iq.payload {
            let _disco_info = DiscoInfoResult::try_from(element)?;
          }
          else {
            bail!("disco failed");
          }

          let iq = Iq::from_get(generate_id(), xmpp::extdisco::ServicesQuery {})
            .with_from(Jid::Full(
              locked_inner.jid.as_ref().context("missing jid")?.clone(),
            ))
            .with_to(Jid::Bare(locked_inner.xmpp_domain.clone()));
          tx.send(iq.into()).await?;
          locked_inner.state = DiscoveringExternalServices;
        },
        DiscoveringExternalServices => {
          let iq = Iq::try_from(element)?;
          if let IqType::Result(Some(element)) = iq.payload {
            let services = xmpp::extdisco::ServicesResult::try_from(element)?;
            debug!("external services: {:?}", services.services);
            locked_inner.external_services = services.services;
          }
          else {
            bail!("extdisco failed");
          }

          if let Some(tx) = locked_inner.connected_tx.take() {
            tx.send(Ok(())).map_err(|_| anyhow!("channel closed"))?;
          }
          locked_inner.state = Idle;
        },
        Idle => {
          for filter in &locked_inner.stanza_filters {
            if filter.filter(&element) {
              filter.take(element).await?;
              break;
            }
          }
        },
      }
    }
  }
}
