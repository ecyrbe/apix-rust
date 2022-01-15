use std::path::PathBuf;

use super::http_utils::Language;
use anyhow::Result;
use bat::{Input, PrettyPrinter};
use reqwest::{Request, Response};
use serde_json::Value;
use term_size::dimensions_stdout;
use url::Position;

pub trait HttpDisplay {
  fn print(&self, theme: &str, enable_color: bool) -> Result<()>;
}

pub fn print_separator() {
  if let Some((width, _)) = dimensions_stdout() {
    println!("{}", "─".repeat(width));
  }
}

pub fn pretty_print(content: String, theme: &str, language: &str, enable_color: bool) -> Result<()> {
  match language {
    "json" => {
      let json: Value = serde_json::from_str(&content)?;
      let formatted = serde_json::to_string_pretty(&json)?;
      PrettyPrinter::new()
        .input(Input::from_reader(formatted.as_bytes()))
        .language(language)
        .colored_output(enable_color)
        .theme(theme)
        .print()
        .map_err(|err| anyhow::anyhow!("Failed to print result: {:#}", err))?;
    }
    _ => {
      PrettyPrinter::new()
        .input(Input::from_reader(content.as_bytes()))
        .language(language)
        .colored_output(enable_color)
        .theme(theme)
        .print()
        .map_err(|err| anyhow::anyhow!("Failed to print result: {:#}", err))?;
    }
  }
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
    pretty_print(output, theme, "yaml", enable_color)?;

    // pretty print body if present and it has a content type that match a language
    if let (Some(body), Some(language)) = (self.body(), self.get_language()) {
      println!();
      if let Some(bytes) = body.as_bytes() {
        PrettyPrinter::new()
          .input(Input::from_reader(bytes))
          .language(language)
          .colored_output(enable_color)
          .theme(theme)
          .print()
          .map_err(|err| anyhow::anyhow!("Failed to print result: {:#}", err))?;
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
    pretty_print(output, theme, "yaml", enable_color)?;
    Ok(())
  }
}
