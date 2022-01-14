use super::requests::AdvancedBody;
use anyhow::Result;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;
use strum_macros::Display;

#[derive(Display, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum RequestParam {
  Header,
  Cookie,
  Query,
  Param,
}

#[derive(Debug)]
struct HeaderTuple(HeaderName, HeaderValue);

impl FromStr for HeaderTuple {
  type Err = anyhow::Error;
  fn from_str(header_string: &str) -> Result<Self, Self::Err> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new("^([\\w-]+):(.*)$").unwrap());

    let header_split = RE.captures(header_string).ok_or_else(|| {
      anyhow::anyhow!(
        "Bad header format: \"{}\", should be of the form \"<name>:<value>\"",
        header_string
      )
    })?;
    Ok(HeaderTuple(
      HeaderName::from_str(&header_split[1])?,
      HeaderValue::from_str(&header_split[2])?,
    ))
  }
}

#[derive(Debug)]
struct StringTuple(String, String);

impl FromStr for StringTuple {
  type Err = anyhow::Error;
  fn from_str(query_string: &str) -> Result<Self, Self::Err> {
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new("^([\\w-]+):(.*)$").unwrap());

    let query = query_string.to_string();
    let header_split = RE.captures(&query).ok_or_else(|| {
      anyhow::anyhow!(
        "Bad query format: \"{}\", should be of the form \"<name>:<value>\"",
        query_string
      )
    })?;
    Ok(StringTuple(header_split[1].to_string(), header_split[2].to_string()))
  }
}

pub trait MatchParams {
  fn match_headers(&self) -> Option<reqwest::header::HeaderMap>;
  fn match_params(&self, param_type: RequestParam) -> Option<IndexMap<String, String>>;
  fn match_body(&self) -> Result<AdvancedBody>;
}

impl MatchParams for clap::ArgMatches {
  fn match_headers(&self) -> Option<reqwest::header::HeaderMap> {
    if let Ok(header_tuples) = self.values_of_t::<HeaderTuple>("header") {
      let headers = header_tuples.iter().map(|tuple| (tuple.0.clone(), tuple.1.clone()));
      Some(HeaderMap::from_iter(headers))
    } else {
      None
    }
  }

  fn match_params(&self, param_type: RequestParam) -> Option<IndexMap<String, String>> {
    if let Ok(param_tuples) = self.values_of_t::<StringTuple>(&param_type.to_string()) {
      let params = param_tuples.iter().map(|tuple| (tuple.0.clone(), tuple.1.clone()));
      Some(IndexMap::from_iter(params))
    } else {
      None
    }
  }

  fn match_body(&self) -> Result<AdvancedBody> {
    if let Some(body) = self.value_of("body") {
      Ok(AdvancedBody::String(body.to_string()))
    } else if let Some(file) = self.value_of("file") {
      Ok(AdvancedBody::File(file.to_string()))
    } else {
      Ok(AdvancedBody::None)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::{arg, App};

  // test match headers
  #[test]
  fn test_match_headers() {
    let matches = App::new("test")
      .arg(arg!(--header "Header to add").takes_value(true))
      .get_matches_from(vec!["test", "--header", "foo:bar"]);
    let headers = matches.match_headers();
    assert!(headers.is_some());
    let headers = headers.unwrap();
    assert_eq!(headers.get("foo"), Some(&"bar".parse::<HeaderValue>().unwrap()));
  }

  // test match queries
  #[test]
  fn test_match_queries() {
    let matches = App::new("test")
      .arg(arg!(--query "Query to add").takes_value(true))
      .get_matches_from(vec!["test", "--query", "foo:bar"]);
    let queries = matches.match_params(RequestParam::Query);
    assert!(queries.is_some());
    let queries = queries.unwrap();
    assert_eq!(queries.get("foo"), Some(&"bar".to_string()));
  }

  // test match params
  #[test]
  fn test_match_params() {
    let matches = App::new("test")
      .arg(arg!(--param "Param to add").takes_value(true))
      .get_matches_from(vec!["test", "--param", "foo:bar"]);
    let params = matches.match_params(RequestParam::Param);
    assert!(params.is_some());
    let params = params.unwrap();
    assert_eq!(params.get("foo"), Some(&"bar".to_string()));
  }

  // test match body
  #[test]
  fn test_match_body() {
    let matches = App::new("test")
      .arg(arg!(--body "Body to add").takes_value(true))
      .get_matches_from(vec!["test", "--body", "foo"]);
    let body = matches.match_body();
    assert!(body.is_ok());
    assert_eq!(body.unwrap().to_string().unwrap(), "foo".to_string());
  }
}
