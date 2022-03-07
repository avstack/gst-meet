use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use colibri::ColibriMessage;
use futures::{
  sink::SinkExt,
  stream::{StreamExt, TryStreamExt},
};
use rand::{thread_rng, RngCore};
use tokio::{
  sync::{mpsc, Mutex},
  time::sleep,
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite::{
  http::{Request, Uri},
  Message,
};
use tracing::{debug, error, info, warn};

use crate::tls::wss_connector;

const MAX_CONNECT_RETRIES: u8 = 3;
const CONNECT_RETRY_SLEEP: Duration = Duration::from_secs(3);

#[derive(Clone)]
pub(crate) struct ColibriChannel {
  send_tx: mpsc::Sender<ColibriMessage>,
  recv_tx: Arc<Mutex<Vec<mpsc::Sender<ColibriMessage>>>>,
}

impl ColibriChannel {
  pub(crate) async fn new(uri: &str, tls_insecure: bool) -> Result<Self> {
    let uri: Uri = uri.parse()?;
    let host = uri.host().context("invalid WebSocket URL: missing host")?;

    let mut retries = 0;
    let colibri_websocket = loop {
      let mut key = [0u8; 16];
      thread_rng().fill_bytes(&mut key);
      let request = Request::get(&uri)
        .header("sec-websocket-key", base64::encode(&key))
        .header("sec-websocket-version", "13")
        .header("host", host)
        // TODO: the server should probably not enforce this since non-browser clients are now possible
        .header("origin", format!("https://{}", host))
        .header("connection", "Upgrade")
        .header("upgrade", "websocket")
        .body(())?;
      match tokio_tungstenite::connect_async_tls_with_config(
        request,
        None,
        Some(wss_connector(tls_insecure).context("failed to build TLS connector")?),
      )
      .await
      {
        Ok((websocket, _)) => break websocket,
        Err(e) => {
          if retries < MAX_CONNECT_RETRIES {
            warn!("Failed to connect Colibri WebSocket, will retry: {:?}", e);
            sleep(CONNECT_RETRY_SLEEP).await;
            retries += 1;
          }
          else {
            return Err(e).context("Failed to connect Colibri WebSocket");
          }
        },
      }
    };

    info!("Connected Colibri WebSocket");

    let (mut colibri_sink, mut colibri_stream) = colibri_websocket.split();
    let recv_tx: Arc<Mutex<Vec<mpsc::Sender<ColibriMessage>>>> = Arc::new(Mutex::new(vec![]));
    let recv_task = {
      let recv_tx = recv_tx.clone();
      tokio::spawn(async move {
        while let Some(msg) = colibri_stream.try_next().await? {
          match msg {
            Message::Text(text) => {
              debug!("Colibri <<< {}", text);
              match serde_json::from_str::<ColibriMessage>(&text) {
                Ok(colibri_msg) => {
                  let mut txs = recv_tx.lock().await;
                  let txs_clone = txs.clone();
                  for (i, tx) in txs_clone.iter().enumerate().rev() {
                    if tx.send(colibri_msg.clone()).await.is_err() {
                      debug!("colibri subscriber closed, removing");
                      txs.remove(i);
                    }
                  }
                },
                Err(e) => warn!(
                  "failed to parse frame on colibri websocket: {:?}\nframe: {}",
                  e, text
                ),
              }
            },
            Message::Binary(data) => debug!(
              "received unexpected {} byte binary frame on colibri websocket",
              data.len()
            ),
            Message::Close(_) => {
              debug!("received close frame on colibri websocket");
              // TODO reconnect
              break;
            },
            Message::Frame(_) | Message::Ping(_) | Message::Pong(_) => {}, // handled automatically by tungstenite
          }
        }
        Ok::<_, anyhow::Error>(())
      })
    };

    let (send_tx, send_rx) = mpsc::channel(8);
    let send_task = tokio::spawn(async move {
      let mut stream = ReceiverStream::new(send_rx);
      while let Some(colibri_msg) = stream.next().await {
        match serde_json::to_string(&colibri_msg) {
          Ok(json) => {
            debug!("Colibri >>> {}", json);
            let msg = Message::Text(json);
            colibri_sink.send(msg).await?;
          },
          Err(e) => warn!("failed to serialise colibri message: {:?}", e),
        }
      }
      Ok::<_, anyhow::Error>(())
    });

    tokio::spawn(async move {
      tokio::select! {
        res = recv_task => if let Ok(Err(e)) = res {
          error!("colibri recv loop: {:?}", e);
        },
        res = send_task => if let Ok(Err(e)) = res {
          error!("colibri send loop: {:?}", e);
        },
      };
    });

    Ok(Self { send_tx, recv_tx })
  }

  pub(crate) async fn subscribe(&self, tx: mpsc::Sender<ColibriMessage>) {
    self.recv_tx.lock().await.push(tx);
  }

  pub(crate) async fn send(&self, msg: ColibriMessage) -> Result<()> {
    self.send_tx.send(msg).await?;
    Ok(())
  }
}
