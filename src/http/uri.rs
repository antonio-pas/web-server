#[derive(Debug, Clone)]
pub struct Uri {
  string: String,
  query: Option<u16>,
}
impl Uri {
  pub fn from_parts(path: &str, query: &str) -> Self {
    let sep = if query.is_empty() { "" } else { "?" };
    let string = format!("{path}{sep}{query}");
    let query = if query.is_empty() {
      None
    } else {
      Some(path.len() as u16)
    };
    Self { string, query }
  }
  pub fn from_str(src: &str) -> Self {
    let mut fragment = None;
    let mut query = None;
    for (i, char) in src.chars().enumerate() {
      if char == '?' {
        query = Some(i as u16);
      }
      if char == '#' {
        fragment = Some(i);
      }
    }
    let mut string = src.to_string();
    if let Some(l) = fragment {
      string.truncate(l)
    };
    Self { string, query }
  }
  pub fn path(&self) -> &str {
    self
      .query
      .and_then(|i| Some(&self.string[..i as usize]))
      .unwrap_or(&self.string)
  }
  pub fn query(&self) -> &str {
    self
      .query
      .and_then(|i| Some(&self.string[(i + 1) as usize..]))
      .unwrap_or_default()
  }
}
