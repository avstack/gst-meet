use std::convert::TryFrom;

use anyhow::{bail, Context, Result};
use xmpp_parsers::{iq::IqGetPayload, Element};

use crate::xmpp::ns;

#[derive(Debug)]
pub(crate) struct ServicesQuery {}

impl TryFrom<Element> for ServicesQuery {
  type Error = anyhow::Error;

  fn try_from(_elem: Element) -> Result<ServicesQuery> {
    unimplemented!()
  }
}

impl From<ServicesQuery> for Element {
  fn from(_services: ServicesQuery) -> Element {
    let builder = Element::builder("services", ns::EXTDISCO);
    builder.build()
  }
}

impl IqGetPayload for ServicesQuery {}

#[derive(Debug, Clone)]
pub struct Service {
  pub(crate) r#type: String,
  pub(crate) name: Option<String>,
  pub(crate) host: String,
  pub(crate) port: Option<u16>,
  pub(crate) transport: Option<String>,
  pub(crate) restricted: Option<bool>,
  pub(crate) username: Option<String>,
  pub(crate) password: Option<String>,
  pub(crate) expires: Option<String>,
}

#[derive(Debug)]
pub(crate) struct ServicesResult {
  pub(crate) services: Vec<Service>,
}

impl TryFrom<Element> for ServicesResult {
  type Error = anyhow::Error;

  fn try_from(elem: Element) -> Result<ServicesResult> {
    if !elem.is("services", ns::EXTDISCO) {
      bail!("not a services element");
    }
    Ok(ServicesResult {
      services: elem
        .children()
        .map(|child| {
          Ok(Service {
            r#type: child.attr("type").context("missing type attr")?.to_owned(),
            name: child.attr("name").map(ToOwned::to_owned),
            host: child.attr("host").context("missing host attr")?.to_owned(),
            port: child.attr("port").map(|p| p.parse()).transpose()?,
            transport: child.attr("transport").map(ToOwned::to_owned),
            restricted: child
              .attr("restricted")
              .map(|b| b.to_lowercase() == "parse" || b == "1"),
            username: child.attr("username").map(ToOwned::to_owned),
            password: child.attr("password").map(ToOwned::to_owned),
            expires: child.attr("expires").map(ToOwned::to_owned),
          })
        })
        .collect::<Result<_>>()?,
    })
  }
}
