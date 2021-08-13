use std::convert::TryFrom;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tokio::sync::mpsc;
use xmpp_parsers::{iq::Iq, Element, FullJid, Jid};

use crate::stanza_filter::StanzaFilter;

#[derive(Debug)]
pub(crate) struct Pinger {
  pub(crate) jid: FullJid,
  pub(crate) tx: mpsc::Sender<Element>,
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
