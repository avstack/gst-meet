use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use futures::{
  sink::SinkExt,
  stream::{StreamExt, TryStreamExt},
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tokio_tungstenite::tungstenite::{http::Request, Message};
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "colibriClass")]
pub enum ColibriMessage {
  #[serde(rename_all = "camelCase")]
  DominantSpeakerEndpointChangeEvent {
    dominant_speaker_endpoint: String,
    previous_speakers: Vec<String>,
  },
  #[serde(rename_all = "camelCase")]
  EndpointConnectivityStatusChangeEvent {
    endpoint: String,
    active: bool,
  },
  #[serde(rename_all = "camelCase")]
  EndpointMessage {
    from: String,
    to: Option<String>,
    msg_payload: serde_json::Value,
  },
  #[serde(rename_all = "camelCase")]
  EndpointStats {
    from: String,
    bitrate: Bitrates,
    packet_loss: PacketLoss,
    connection_quality: f32,
    #[serde(rename = "jvbRTT")]
    jvb_rtt: u16,
    server_region: String,
    max_enabled_resolution: u16,
  },
  #[serde(rename_all = "camelCase")]
  LastNChangedEvent {
    last_n: u16,
  },
  #[serde(rename_all = "camelCase")]
  LastNEndpointsChangeEvent {
    last_n_endpoints: Vec<String>,
  },
  #[serde(rename_all = "camelCase")]
  ReceiverVideoConstraint {
    max_frame_height: u16,
  },
  #[serde(rename_all = "camelCase")]
  ReceiverVideoConstraints {
    last_n: Option<u16>,
    selected_endpoints: Option<Vec<String>>,
    on_stage_endpoints: Option<Vec<String>>,
    default_constraints: Option<Constraints>,
    constraints: Option<HashMap<String, Constraints>>,
  },
  #[serde(rename_all = "camelCase")]
  SelectedEndpointsChangedEvent {
    selected_endpoints: Vec<String>,
  },
  #[serde(rename_all = "camelCase")]
  SenderVideoConstraints {
    video_constraints: Constraints,
  },
  #[serde(rename_all = "camelCase")]
  ServerHello {
    version: Option<String>,
  },
  #[serde(rename_all = "camelCase")]
  VideoTypeMessage {
    video_type: VideoType,
  },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoType {
  Camera,
  Desktop,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Constraints {
  pub ideal_height: Option<u16>,
  pub max_height: Option<u16>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bitrates {
  pub audio: Bitrate,
  pub video: Bitrate,
  #[serde(flatten)]
  pub total: Bitrate,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bitrate {
  pub upload: u32,
  pub download: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PacketLoss {
  pub total: u32,
  pub download: u32,
  pub upload: u32,
}

pub(crate) struct ColibriChannel {
  send_tx: mpsc::Sender<ColibriMessage>,
  recv_tx: Arc<Mutex<Vec<mpsc::Sender<ColibriMessage>>>>,
}

impl ColibriChannel {
  pub(crate) async fn new(colibri_url: &str) -> Result<Self> {
    let request =
      Request::get(colibri_url).body(())?;
    let (colibri_websocket, _response) =
      tokio_tungstenite::connect_async(request).await?;
    
    info!("Connected Colibri WebSocket");

    let (mut colibri_sink, mut colibri_stream) = colibri_websocket.split();
    let recv_tx: Arc<Mutex<Vec<mpsc::Sender<ColibriMessage>>>> = Arc::new(Mutex::new(vec![]));
    let recv_task = {
      let recv_tx = recv_tx.clone();
      tokio::spawn(async move {
        while let Some(msg) = colibri_stream.try_next().await? {
          match msg {
            Message::Text(text) => {
              debug!("colibri: {}", text);
              match serde_json::from_str::<ColibriMessage>(&text) {
                Ok(colibri_msg) => {
                  let mut txs = recv_tx.lock().await;
                  let txs_clone = txs.clone();
                  for (i, tx) in txs_clone.iter().enumerate().rev() {
                    if let Err(_) = tx.send(colibri_msg.clone()).await {
                      debug!("colibri subscriber closed, removing");
                      txs.remove(i);
                    }
                  }
                },
                Err(e) => warn!("failed to parse frame on colibri websocket: {:?}\nframe: {}", e, text),
              }
            },
            Message::Binary(data) => debug!("received unexpected {} byte binary frame on colibri websocket", data.len()),
            Message::Ping(_) | Message::Pong(_) => {},  // handled automatically by tungstenite
            Message::Close(_) => {
              debug!("received close frame on colibri websocket");
              // TODO reconnect
              break;
            },
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

    Ok(Self {
      send_tx,
      recv_tx,
    })
  }

  pub(crate) async fn subscribe(&self, tx: mpsc::Sender<ColibriMessage>) {
    self.recv_tx.lock().await.push(tx);
  }

  pub(crate) async fn send(&self, msg: ColibriMessage) -> Result<()> {
    self.send_tx.send(msg).await?;
    Ok(())
  }
}