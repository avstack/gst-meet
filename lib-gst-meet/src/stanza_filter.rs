use anyhow::Result;
use async_trait::async_trait;
use xmpp_parsers::Element;

#[async_trait]
pub trait StanzaFilter {
  fn filter(&self, element: &Element) -> bool;
  async fn take(&self, element: Element) -> Result<()>;
}
