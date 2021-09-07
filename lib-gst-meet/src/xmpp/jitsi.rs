use std::{collections::HashMap, convert::TryFrom};

use anyhow::Result;
use xmpp_parsers::{iq::IqSetPayload, Element};

use crate::xmpp::ns;

pub(crate) struct Conference {
  pub(crate) machine_uid: String,
  pub(crate) room: String,
  pub(crate) properties: HashMap<String, String>,
}

impl IqSetPayload for Conference {}

impl TryFrom<Element> for Conference {
  type Error = anyhow::Error;

  fn try_from(_element: Element) -> Result<Conference> {
    unimplemented!()
  }
}

impl From<Conference> for Element {
  fn from(conference: Conference) -> Element {
    let mut builder = Element::builder("conference", ns::JITSI_FOCUS)
      .attr("machine-uid", conference.machine_uid)
      .attr("room", conference.room);
    for (name, value) in conference.properties {
      builder = builder.append(
        Element::builder("property", ns::JITSI_FOCUS)
          .attr("name", name)
          .attr("value", value)
          .build(),
      );
    }
    builder.build()
  }
}

pub(crate) struct JsonMessage {
  pub(crate) payload: serde_json::Value,
}

impl TryFrom<JsonMessage> for Element {
  type Error = anyhow::Error;

  fn try_from(message: JsonMessage) -> Result<Element> {
    Ok(
      Element::builder("json-message", ns::JITSI_JITMEET)
        .append(serde_json::to_string(&message.payload)?)
        .build(),
    )
  }
}
