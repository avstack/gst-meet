use xmpp_parsers::{
  hashes::{Algo, Hash},
  jingle_dtls_srtp::Setup,
  ns::JINGLE_DTLS,
  Error,
};

use crate::helpers::ColonSeparatedHex;

generate_element!(
  /// Fingerprint of the key used for a DTLS handshake.
  Fingerprint, "fingerprint", JINGLE_DTLS,
  attributes: [
      /// The hash algorithm used for this fingerprint.
      hash: Required<Algo> = "hash",

      /// Indicates which of the end points should initiate the TCP connection establishment.
      setup: Option<Setup> = "setup"
  ],
  text: (
      /// Hash value of this fingerprint.
      value: ColonSeparatedHex<Vec<u8>>
  )
);

impl Fingerprint {
  /// Create a new Fingerprint from a Setup and a Hash.
  pub fn from_hash(setup: Setup, hash: Hash) -> Fingerprint {
    Fingerprint {
      hash: hash.algo,
      setup: Some(setup),
      value: hash.hash,
    }
  }

  /// Create a new Fingerprint from a Setup and parsing the hash.
  pub fn from_colon_separated_hex(
    setup: Setup,
    algo: &str,
    hash: &str,
  ) -> Result<Fingerprint, Error> {
    let algo = algo.parse()?;
    let hash = Hash::from_colon_separated_hex(algo, hash)?;
    Ok(Fingerprint::from_hash(setup, hash))
  }
}
