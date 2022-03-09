use std::{convert::TryFrom, time::Duration};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tokio::{sync::mpsc, task::JoinHandle, time};
use tracing::warn;
use xmpp_parsers::{iq::Iq, ping::Ping, Element, FullJid, Jid};

use crate::{stanza_filter::StanzaFilter, util::generate_id};

const PING_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug)]
pub(crate) struct Pinger {
  pub(crate) jid: FullJid,
  pub(crate) tx: mpsc::Sender<Element>,
  pub(crate) ping_task: JoinHandle<()>,
}

impl Pinger {
  pub(crate) fn new(jid: FullJid, tx: mpsc::Sender<Element>) -> Pinger {
    let ping_tx = tx.clone();
    let ping_task = tokio::spawn(async move {
      let mut interval = time::interval(PING_INTERVAL);
      loop {
        interval.tick().await;
        if let Err(e) = ping_tx.send(Iq::from_get(generate_id(), Ping).into()).await {
          warn!("failed to send XMPP ping: {:?}", e);
        }
      }
    });
    Pinger { jid, tx, ping_task }
  }
}

#[async_trait]
impl StanzaFilter for Pinger {
  #[tracing::instrument(level = "trace")]
  fn filter(&self, element: &Element) -> bool {
    element.is("iq", "jabber:client") && element.has_child("ping", "urn:xmpp:ping")
  }

  #[tracing::instrument(level = "trace", err)]
  async fn take(&self, element: Element) -> Result<()> {
    let iq = Iq::try_from(element)?;
    let result_iq = Iq::empty_result(
      iq.from.ok_or_else(|| anyhow!("iq missing from"))?,
      iq.id.clone(),
    )
    .with_from(Jid::Full(self.jid.clone()));
    self.tx.send(result_iq.into()).await?;
    Ok(())
  }
}
