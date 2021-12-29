use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;
use strum_macros::Display;

#[derive(Display, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum RequestParam {
  Header,
  Cookie,
  Query,
  Variable,
}

#[derive(Debug)]
struct HeaderTuple(HeaderName, HeaderValue);

impl FromStr for HeaderTuple {
  type Err = anyhow::Error;
  fn from_str(header_string: &str) -> Result<Self, Self::Err> {
    lazy_static! {
        static ref RE: Regex = Regex::new("^([\\w-]+):(.*)$").unwrap(); // safe unwrap
    }
    let header_split = RE.captures(header_string).ok_or(anyhow::anyhow!(
      "Bad header format: \"{}\", should be of the form \"<name>:<value>\"",
      header_string
    ))?;
    Ok(HeaderTuple(
      HeaderName::from_str(&header_split[1])?,
      HeaderValue::from_str(&header_split[2])?,
    ))
  }
}

#[derive(Debug)]
struct QueryTuple(String, String);

impl FromStr for QueryTuple {
  type Err = anyhow::Error;
  fn from_str(query_string: &str) -> Result<Self, Self::Err> {
    lazy_static! {
        static ref RE: Regex = Regex::new("^([\\w-]+):(.*)$").unwrap(); // safe unwrap
    }
    let query = query_string.to_string();
    let header_split = RE.captures(&query).ok_or(anyhow::anyhow!(
      "Bad query format: \"{}\", should be of the form \"<name>:<value>\"",
      query_string
    ))?;
    Ok(QueryTuple(header_split[1].to_string(), header_split[2].to_string()))
  }
}

pub fn match_headers(matches: &clap::ArgMatches) -> Option<reqwest::header::HeaderMap> {
  if let Ok(header_tuples) = matches.values_of_t::<HeaderTuple>("header") {
    let headers = header_tuples.iter().map(|tuple| (tuple.0.clone(), tuple.1.clone()));
    Some(HeaderMap::from_iter(headers))
  } else {
    None
  }
}

pub fn match_queries(matches: &clap::ArgMatches) -> Option<HashMap<String, String>> {
  if let Ok(query_tuples) = matches.values_of_t::<QueryTuple>("query") {
    let queries = query_tuples.iter().map(|tuple| (tuple.0.clone(), tuple.1.clone()));
    Some(HashMap::from_iter(queries))
  } else {
    None
  }
}

pub fn match_body(matches: &clap::ArgMatches) -> Result<String> {
  if let Some(body) = matches.value_of("body") {
    Ok(body.to_string())
  } else if let Some(file) = matches.value_of("file") {
    fs::read_to_string(file).map_err(|err| anyhow::anyhow!("Could not read file '{}': {:#}", file, err))
  } else {
    Ok(String::new())
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
    let headers = match_headers(&matches);
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
    let queries = match_queries(&matches);
    assert!(queries.is_some());
    let queries = queries.unwrap();
    assert_eq!(queries.get("foo"), Some(&"bar".to_string()));
  }

  // test match body
  #[test]
  fn test_match_body() {
    let matches = App::new("test")
      .arg(arg!(--body "Body to add").takes_value(true))
      .get_matches_from(vec!["test", "--body", "foo"]);
    let body = match_body(&matches);
    assert!(body.is_ok());
    assert_eq!(body.unwrap(), "foo".to_string());
  }
}
