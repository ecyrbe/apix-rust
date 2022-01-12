use std::path::PathBuf;

use super::http_utils::Language;
use anyhow::Result;
use bat::{Input, PrettyPrinter};
use reqwest::{Request, Response};
use url::Position;

pub trait HttpDisplay {
  fn print(&self, theme: &str, enable_color: bool) -> Result<()>;
}

pub fn pretty_print(content: &[u8], theme: &str, language: &str, enable_color: bool) -> Result<()> {
  PrettyPrinter::new()
    .input(Input::from_reader(content))
    .language(language)
    .colored_output(enable_color)
    .theme(theme)
    .print()
    .map_err(|err| anyhow::anyhow!("Failed to print result: {:#}", err))?;
  Ok(())
}

pub fn pretty_print_file(path: PathBuf, theme: &str, language: &str, enable_color: bool) -> Result<()> {
  PrettyPrinter::new()
    .input_file(path)
    .language(language)
    .colored_output(enable_color)
    .theme(theme)
    .grid(true)
    .header(true)
    .line_numbers(true)
    .print()
    .map_err(|err| anyhow::anyhow!("Failed to print result: {:#}", err))?;
  Ok(())
}

impl HttpDisplay for Request {
  fn print(&self, theme: &str, enable_color: bool) -> Result<()> {
    let mut output = format!(
      "{method} {endpoint} {protocol:?}\nhost: {host}\n",
      method = self.method(),
      endpoint = &self.url()[Position::BeforePath..],
      protocol = self.version(),
      host = self
        .url()
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("invalid host in URL: {}", self.url()))?
    );
    for (key, value) in self.headers() {
      output.push_str(&format!("{}: {}\n", key.as_str(), value.to_str()?));
    }
    pretty_print(output.as_bytes(), theme, "yaml", enable_color)?;

    // pretty print body if present and it has a content type that match a language
    if let (Some(body), Some(language)) = (self.body(), self.get_language()) {
      println!();
      if let Some(bytes) = body.as_bytes() {
        pretty_print(bytes, theme, language, enable_color)?;
      }
    }
    Ok(())
  }
}

impl HttpDisplay for Response {
  fn print(&self, theme: &str, enable_color: bool) -> Result<()> {
    let mut output = format!(
      "{protocol:?} {status}\n",
      protocol = self.version(),
      status = self.status()
    );
    for (key, value) in self.headers() {
      output.push_str(&format!("{}: {}\n", key.as_str(), value.to_str()?));
    }
    pretty_print(output.as_bytes(), theme, "yaml", enable_color)?;
    Ok(())
  }
}
