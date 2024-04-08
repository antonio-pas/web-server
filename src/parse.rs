use std::collections;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net;

use crate::error::*;
use crate::http::{Headers, Request, RequestMethod, Uri};

pub struct Parser {
  input: Vec<u8>,
  pos: usize,
}

impl Parser {
  pub fn new(input: Vec<u8>) -> Self {
    Self { input, pos: 0 }
  }
  pub fn consume(&mut self) -> u8 {
    let byte = self.input[self.pos];
    self.pos += 1;
    byte
  }
  pub fn peek(&self) -> u8 {
    self.input[self.pos]
  }
  pub fn eof(&self) -> bool {
    self.pos >= self.input.len()
  }
  pub fn consume_while(&mut self, test: fn(u8) -> bool) -> Vec<u8> {
    let mut vec = vec![];
    while !self.eof() && test(self.peek()) {
      vec.push(self.consume());
    }
    vec
  }
  pub fn starts_with(&mut self, seq: &[u8]) -> bool {
    self.input[self.pos..].starts_with(seq)
  }
  pub fn parse(&mut self) -> Result<Request, Error> {
    use Parse::*;
    let method = self
      .consume_while(|b| b.is_ascii_uppercase())
      .iter()
      .map(|b| *b as char)
      .collect::<String>();
    if self.consume() != b' ' {
      return Err(Method.into());
    }

    let uri_path = self
      .consume_while(|b| b != b'?' && b != b' ' && b.is_ascii_graphic())
      .iter()
      .map(|b| *b as char)
      .collect::<String>();
    if self.peek() == b'?' {
      self.consume();
    }
    let uri_query = self
      .consume_while(|b| b != b'#' && b != b' ' && b.is_ascii_graphic())
      .iter()
      .map(|b| *b as char)
      .collect::<String>();
    let _uri_fragment = self
      .consume_while(|b| b != b' ' && b.is_ascii_graphic())
      .iter()
      .map(|b| *b as char)
      .collect::<String>();

    if self.consume() != b' ' {
      return Err(Uri.into());
    };

    let version = self
      .consume_while(|b| b.is_ascii_graphic() && b != b'\r')
      .iter()
      .map(|b| *b as char)
      .collect::<String>();
    if self.consume() != b'\r' {
      return Err(Version.into());
    }
    if self.consume() != b'\n' {
      return Err(Version.into());
    }

    let mut headers = Headers::new();
    loop {
      if self.starts_with(b"\r\n") {
        break;
      }

      let name = self
        .consume_while(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        .iter()
        .map(|b| *b as char)
        .collect::<String>();
      if self.consume() != b':' {
        return Err(Header.into());
      }
      if self.peek() == b' ' {
        self.consume();
      }
      let value = self
        .consume_while(|b| b.is_ascii_graphic() || b == b' ')
        .iter()
        .map(|b| *b as char)
        .collect::<String>();
      headers.insert(name, value);
      if self.consume() != b'\r' {
        return Err(Header.into());
      }
      if self.consume() != b'\n' {
        return Err(Header.into());
      }
    }

    let method = RequestMethod::try_from(method).map_err(|_| Method)?;
    let uri = crate::Uri::from_parts(&uri_path, &uri_query);
    let body = &self.input[self.pos..];

    let request = Request::new(method, uri, headers, body.to_vec().into());

    Ok(request)
  }
}
