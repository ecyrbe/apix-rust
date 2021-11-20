use super::http_utils::Language;
use anyhow::Result;
use bat::{Input, PrettyPrinter};
use reqwest::{Request, Response};
use url::Position;

pub trait HttpDisplay {
  fn print(&self, theme: &str) -> Result<()>;
}

pub fn pretty_print(content: &[u8], theme: &str, language: &str) -> Result<()> {
  PrettyPrinter::new()
    .input(Input::from_reader(content))
    .language(language)
    .theme(theme)
    .print()
    .map_err(|err| anyhow::anyhow!("Failed to print result: {}", err))?;
  Ok(())
}

impl HttpDisplay for Request {
  fn print(&self, theme: &str) -> Result<()> {
    let mut output = format!(
      "{method} {endpoint} {protocol:?}\nhost: {host}\n",
      method = self.method(),
      endpoint = &self.url()[Position::BeforePath..],
      protocol = self.version(),
      host = self
        .url()
        .host_str()
        .ok_or(anyhow::anyhow!("invalid host in URL: {}", self.url()))?
    );
    for header in self.headers() {
      output.push_str(&format!("{}: {}\n", header.0.as_str(), header.1.to_str()?));
    }
    pretty_print(output.as_bytes(), theme, "yaml")?;

    // pretty print body if present and it has a content type that match a language
    match (self.body(), self.get_language()) {
      (Some(body), Some(language)) => {
        println!("");
        pretty_print(body.as_bytes().unwrap(), theme, language)?;
      }
      _ => {}
    }

    Ok(())
  }
}

impl HttpDisplay for Response {
  fn print(&self, theme: &str) -> Result<()> {
    let mut output = format!(
      "{protocol:?} {status}\n",
      protocol = self.version(),
      status = self.status()
    );
    for header in self.headers() {
      output.push_str(&format!("{}: {}\n", header.0.as_str(), header.1.to_str()?));
    }
    pretty_print(output.as_bytes(), theme, "yaml")?;
    Ok(())
  }
}
