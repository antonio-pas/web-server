pub type Headers = std::collections::HashMap<String, String>;

#[derive(Debug, Default)]
pub struct Body {
  inner: Vec<u8>,
}
impl Body {
  pub fn new(inner: Vec<u8>) -> Self {
    Self { inner }
  }
  pub fn get_bytes(&self) -> &[u8] {
    &self.inner[..]
  }
}
impl From<Vec<u8>> for Body {
  fn from(value: Vec<u8>) -> Self {
    Self::new(value)
  }
}
impl From<String> for Body {
  fn from(value: String) -> Self {
    Self::new(value.into_bytes())
  }
}
impl From<&str> for Body {
  fn from(value: &str) -> Self {
    Self::new(value.bytes().collect::<Vec<_>>())
  }
}
impl From<()> for Body {
  fn from(value: ()) -> Self {
    Self::new(Vec::new())
  }
}
