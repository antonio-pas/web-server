#[derive(Debug, Copy, Clone)]
pub enum RequestMethod {
  Get,
  Post,
  Put,
  Patch,
  Delete,
}
impl TryFrom<String> for RequestMethod {
  type Error = ();
  fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
    match value.as_str() {
      "GET" => Ok(RequestMethod::Get),
      "POST" => Ok(RequestMethod::Post),
      "PUT" => Ok(RequestMethod::Put),
      "DELETE" => Ok(RequestMethod::Delete),
      "PATCH" => Ok(RequestMethod::Patch),
      _ => Err(()),
    }
  }
}
