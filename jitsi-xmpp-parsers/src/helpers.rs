use xmpp_parsers::Error;

/// Codec for colon-separated bytes of uppercase hexadecimal.
pub struct ColonSeparatedHex;

impl ColonSeparatedHex {
  pub fn decode(s: &str) -> Result<Vec<u8>, Error> {
    let mut bytes = vec![];
    for i in 0..(1 + s.len()) / 3 {
      let byte = u8::from_str_radix(&s[3 * i..3 * i + 2], 16)?;
      if 3 * i + 2 < s.len() {
        assert_eq!(&s[3 * i + 2..3 * i + 3], ":");
      }
      bytes.push(byte);
    }
    Ok(bytes)
  }

  pub fn encode(b: &[u8]) -> Option<String> {
    let mut bytes = vec![];
    for byte in b {
      bytes.push(format!("{:02X}", byte));
    }
    Some(bytes.join(":"))
  }
}
