mod config;
mod http_display;
mod http_utils;
mod import;
use anyhow::Result;
use clap::{crate_version, App, AppSettings, Arg, ValueHint};
use clap_generate::{generate, Generator, Shell};
use config::ApixConfig;
use http_display::{pretty_print, HttpDisplay};
use http_utils::Language;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Method,
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::str::FromStr;
use std::string::ToString;
use strum_macros::Display;
use url::Url;

#[derive(Debug)]
struct HeaderTuple(HeaderName, HeaderValue);

impl FromStr for HeaderTuple {
    type Err = anyhow::Error;
    fn from_str(header_string: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new("^([\\w-]+):(.*)$").unwrap(); // safe unwrap
        }
        let header_split = RE.captures(header_string).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Bad header format: \"{}\", should be of the form \"<name>:<value>\"",
                header_string
            ),
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
        let header_split = RE.captures(&query).ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Bad query format: \"{}\", should be of the form \"<name>:<value>\"",
                query_string
            ),
        ))?;
        Ok(QueryTuple(
            header_split[1].to_string(),
            header_split[2].to_string(),
        ))
    }
}

#[derive(Display, Debug)]
#[strum(serialize_all = "snake_case")]
enum RequestParam {
    Header,
    Cookie,
    Query,
    Variable,
}

fn print_completions<G: Generator>(gen: G, app: &mut App) {
    generate(gen, app, app.get_name().to_string(), &mut io::stdout());
}

fn validate_url(str_url: &str) -> Result<Url, io::Error> {
    let parsed_url = Url::parse(str_url);
    match parsed_url {
        Ok(url) => {
            if !["https", "http"].contains(&url.scheme()) {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Apix only supports http(s) protocols for now",
                ))
            } else {
                Ok(url)
            }
        }
        Err(err) => Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
    }
}

fn validate_param(param: &str, request_type: RequestParam) -> Result<(), io::Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new("^([\\w-]+):(.*)$").unwrap();
    }
    if RE.is_match(param) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Bad {} format: \"{}\", should be of the form \"<name>:<value>\"",
                request_type, param
            ),
        ))
    }
}

fn build_request_args() -> impl Iterator<Item = &'static Arg<'static>> {
    lazy_static! {
        static ref ARGS: [Arg<'static>; 8] = [
            Arg::new("url")
                .about("url to request, can be a 'Tera' template")
                .required(true)
                .validator(validate_url),
            Arg::new("header")
                .short('H')
                .long("header")
                .about("set header name:value to send with request")
                .multiple_occurrences(true)
                .takes_value(true)
                .validator(|param| validate_param(param, RequestParam::Header)),
            Arg::new("cookie")
                .short('c')
                .long("cookie")
                .about("set cookie name:value to send with request")
                .multiple_occurrences(true)
                .takes_value(true)
                .validator(|param| validate_param(param, RequestParam::Cookie)),
            Arg::new("query")
                .short('q')
                .long("query")
                .about("set query name:value to send with request")
                .multiple_occurrences(true)
                .takes_value(true)
                .validator(|param| validate_param(param, RequestParam::Query)),
            Arg::new("body")
                .short('b')
                .long("body")
                .about("set body to send with request, can be a 'Tera' template")
                .takes_value(true)
                .conflicts_with("file"),
            Arg::new("file")
                .short('f')
                .long("file")
                .about("set body from file to send with request, can be a 'Tera' template")
                .takes_value(true)
                .conflicts_with("body")
                .value_hint(ValueHint::FilePath),
            Arg::new("variable")
                .short('e')
                .long("env")
                .about("set variable name:value for 'Tera' template rendering")
                .multiple_occurrences(true)
                .takes_value(true)
                .validator(|param| validate_param(param, RequestParam::Variable)),
            Arg::new("insecure")
                .about("allow insecure connections when using https")
                .short('i')
                .long("insecure"),
        ];
    }
    ARGS.iter()
}

fn build_cli() -> App<'static> {
    App::new("apix")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .args([Arg::new("verbose")
            .about("print full request and response")
            .short('v')
            .long("verbose")
            .global(true)])
        .subcommands([
            App::new("completions")
                .about("generate shell completions")
                .arg(
                    Arg::new("shell")
                        .about("shell to target for completions")
                        .possible_values(Shell::arg_values())
                        .required(true),
                ),
            App::new("config")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .about("configuration settings")
                .subcommands([
                    App::new("list"),
                    App::new("set").about("set configuration value").args([
                        Arg::new("name")
                            .about("name of configuration value to set")
                            .required(true)
                            .index(1),
                        Arg::new("value")
                            .about("value to set configuration value to")
                            .required(true)
                            .index(2),
                    ]),
                    App::new("get")
                        .about("get a configuration value")
                        .args([Arg::new("key").about("key to get").required(true)]),
                    App::new("delete"),
                ]),
            App::new("history").about("show history of requests sent (require project)"),
            App::new("get")
                .about("get an http ressource")
                .args(build_request_args()),
            App::new("head").args(build_request_args()),
            App::new("post").args(build_request_args()),
            App::new("delete").args(build_request_args()),
            App::new("put").args(build_request_args()),
            App::new("patch").args(build_request_args()),
            App::new("exec")
                .about("execute a request from the current API context")
                .args(build_request_args()),
            App::new("ctl")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .about("apix control interface for handling multiple APIs")
                .subcommands([
                    App::new("apply").about("apply an apix manifest into current project"),
                    App::new("init").about("initialise a new API context"),
                    App::new("switch").about("switch API context"),
                    App::new("edit")
                        .about("edit an existing apix resource with current terminal EDITOR"),
                    App::new("get")
                        .about("get information about an apix resource")
                        .arg(
                            Arg::new("resource")
                                .possible_values(["resource", "context", "request", "session"]),
                        ),
                    App::new("delete")
                        .about("delete an existing named resource")
                        .args([Arg::new("resource"), Arg::new("name")]),
                    App::new("import")
                        .about("import an OpenAPI description file in yaml or json")
                        .arg(
                            Arg::new("url")
                                .about("Filename or URL to openApi description to import")
                                .required(true),
                        ),
                ]),
        ])
}

async fn handle_import(url: &str) -> Result<()> {
    let open_api = reqwest::get(url).await?.text().await?;
    let result = import::import_api(open_api, import::OpenApiType::YAML)
        .await
        .map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid Open Api description\n{}", e),
            )
        })?;
    println!("api {}", serde_json::to_string(&result)?);
    Ok(())
}

fn match_headers(matches: &clap::ArgMatches) -> Option<reqwest::header::HeaderMap> {
    if let Ok(header_tuples) = matches.values_of_t::<HeaderTuple>("header") {
        let headers = header_tuples
            .iter()
            .map(|tuple| (tuple.0.clone(), tuple.1.clone()));
        Some(HeaderMap::from_iter(headers))
    } else {
        None
    }
}

fn match_queries(matches: &clap::ArgMatches) -> Option<HashMap<String, String>> {
    if let Ok(query_tuples) = matches.values_of_t::<QueryTuple>("query") {
        let queries = query_tuples
            .iter()
            .map(|tuple| (tuple.0.clone(), tuple.1.clone()));
        Some(HashMap::from_iter(queries))
    } else {
        None
    }
}

fn match_body(matches: &clap::ArgMatches) -> Option<String> {
    if let Some(body) = matches.value_of("body") {
        Some(body.to_string())
    } else if let Some(file) = matches.value_of("file") {
        Some(fs::read_to_string(file).unwrap())
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = build_cli().get_matches();
    // read config file
    let mut config = ApixConfig::read_config()?;
    let theme = config.get("theme").unwrap();
    match matches.subcommand() {
        Some(("completions", matches)) => {
            if let Ok(generator) = matches.value_of_t::<Shell>("shell") {
                let mut app = build_cli();
                println!("debug {}", generator);
                print_completions(generator, &mut app);
            }
        }
        Some(("config", matches)) => match matches.subcommand() {
            Some(("list", _)) => {
                pretty_print(serde_yaml::to_string(&config)?.as_bytes(), &theme, "yaml")?;
            }
            Some(("set", matches)) => {
                let key = matches.value_of("name").unwrap();
                let value = matches.value_of("value").unwrap();
                config.set(key.to_string(), value.to_string());
                config.save_config()?;
            }
            Some(("get", matches)) => {
                let key = matches.value_of("key").unwrap();
                println!("{}", config.get(key).unwrap());
            }
            Some(("delete", matches)) => {
                let key = matches.value_of("key").unwrap();
                config.delete(key);
                config.save_config()?;
            }
            _ => {}
        },
        Some(("history", _submatches)) => {}
        Some(("exec", _submatches)) => {}
        Some(("ctl", matches)) => match matches.subcommand() {
            Some(("apply", _submatches)) => {}
            Some(("init", _submatches)) => {}
            Some(("switch", _submatches)) => {}
            Some(("edit", _submatches)) => {}
            Some(("get", _submatches)) => {}
            Some(("delete", _submatches)) => {}
            Some(("import", matches)) => {
                if let Some(url) = matches.value_of("url") {
                    handle_import(url).await?;
                }
            }
            _ => {}
        },
        Some((method, matches)) => {
            if let Some(url) = matches.value_of("url") {
                let client = reqwest::Client::new();
                let req = client
                    .request(Method::from_str(&method.to_uppercase())?, url)
                    .headers(match_headers(matches).unwrap_or_default())
                    .query(&match_queries(matches).unwrap_or_default())
                    .body(match_body(matches).unwrap_or_default())
                    .build()?;
                if matches.is_present("verbose") {
                    req.print(&theme)?;
                    println!("");
                }
                let result = client.execute(req).await?;
                if matches.is_present("verbose") {
                    result.print(&theme)?;
                    println!("");
                }
                let language = result.get_language();
                let body = result.text().await?;
                pretty_print(body.as_bytes(), &theme, language.unwrap_or_default())?;
            }
        }
        _ => {}
    }
    Ok(())
}
