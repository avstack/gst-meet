#[cfg(any(
  feature = "tls-rustls-native-roots",
  feature = "tls-rustls-webpki-roots"
))]
use std::sync::Arc;

#[cfg(not(feature = "tls-insecure"))]
use anyhow::bail;
use anyhow::{Context, Result};
use tokio_tungstenite::Connector;

#[cfg(feature = "tls-rustls-native-roots")]
pub(crate) fn wss_connector(insecure: bool) -> Result<tokio_tungstenite::Connector> {
  let mut roots = rustls::RootCertStore::empty();
  for cert in
    rustls_native_certs::load_native_certs().context("failed to load native root certs")?
  {
    roots
      .add(&rustls::Certificate(cert.0))
      .context("failed to add native root certs")?;
  }

  let mut config = rustls::ClientConfig::builder()
    .with_safe_defaults()
    .with_root_certificates(roots)
    .with_no_client_auth();
  #[cfg(feature = "tls-insecure")]
  if insecure {
    config
      .dangerous()
      .set_certificate_verifier(Arc::new(InsecureServerCertVerifier));
  }
  #[cfg(not(feature = "tls-insecure"))]
  if insecure {
    bail!(
      "Insecure TLS mode can only be enabled if the tls-insecure feature was enabled at compile time."
    )
  }
  Ok(Connector::Rustls(Arc::new(config)))
}

#[cfg(feature = "tls-rustls-webpki-roots")]
pub(crate) fn wss_connector(insecure: bool) -> Result<tokio_tungstenite::Connector> {
  let mut roots = rustls::RootCertStore::empty();
  roots.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
    rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
      ta.subject,
      ta.spki,
      ta.name_constraints,
    )
  }));

  let config = rustls::ClientConfig::builder()
    .with_safe_defaults()
    .with_root_certificates(roots)
    .with_no_client_auth();
  #[cfg(feature = "tls-insecure")]
  if insecure {
    config
      .dangerous()
      .set_certificate_verifier(Arc::new(InsecureServerCertVerifier));
  }
  #[cfg(not(feature = "tls-insecure"))]
  if insecure {
    bail!(
      "Insecure TLS mode can only be enabled if the tls-insecure feature was enabled at compile time."
    )
  }
  Ok(Connector::Rustls(Arc::new(config)))
}

#[cfg(any(feature = "tls-native", feature = "tls-native-vendored"))]
pub(crate) fn wss_connector(insecure: bool) -> Result<tokio_tungstenite::Connector> {
  let mut builder = native_tls::TlsConnector::builder();
  builder.min_protocol_version(Some(native_tls::Protocol::Tlsv12));
  #[cfg(feature = "tls-insecure")]
  if insecure {
    builder.danger_accept_invalid_certs(true);
    builder.danger_accept_invalid_hostnames(true);
  }
  #[cfg(not(feature = "tls-insecure"))]
  if insecure {
    bail!(
      "Insecure TLS mode can only be enabled if the tls-insecure feature was enabled at compile time."
    )
  }
  Ok(Connector::NativeTls(
    builder
      .build()
      .context("failed to build native TLS connector")?,
  ))
}

#[cfg(all(
  feature = "tls-insecure",
  any(
    feature = "tls-rustls-native-roots",
    feature = "tls-rustls-webpki-roots"
  )
))]
struct InsecureServerCertVerifier;

#[cfg(all(
  feature = "tls-insecure",
  any(
    feature = "tls-rustls-native-roots",
    feature = "tls-rustls-webpki-roots"
  )
))]
impl rustls::client::ServerCertVerifier for InsecureServerCertVerifier {
  fn verify_server_cert(
    &self,
    _end_entity: &rustls::Certificate,
    _intermediates: &[rustls::Certificate],
    _server_name: &rustls::ServerName,
    _scts: &mut dyn Iterator<Item = &[u8]>,
    _ocsp_response: &[u8],
    _now: std::time::SystemTime,
  ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
    Ok(rustls::client::ServerCertVerified::assertion())
  }
}
