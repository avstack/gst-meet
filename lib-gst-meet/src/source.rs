#[derive(Debug, Clone)]
pub(crate) struct Source {
  pub(crate) ssrc: u32,
  pub(crate) participant_id: Option<String>,
  pub(crate) media_type: MediaType,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(C)]
pub enum MediaType {
  Video,
  Audio,
}

impl MediaType {
  pub(crate) fn jitsi_muted_presence_element_name(&self) -> &'static str {
    match self {
      MediaType::Video => "videomuted",
      MediaType::Audio => "audiomuted",
    }
  }
}
