mod dialog;
mod execute;
mod http_display;
mod http_utils;
mod import;
mod manifests;
mod match_params;
mod match_prompts;
mod progress_component;
mod requests;
mod template;
mod validators;
use anyhow::{anyhow, Result};
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ValueHint};
use clap_complete::{generate, Generator, Shell};
use cmd_lib::run_cmd;
use execute::handle_execute;
use http_display::pretty_print;
use indexmap::indexmap;
use manifests::{ApixConfiguration, ApixKind, ApixManifest, ApixRequest, ApixRequestTemplate};
use match_params::{match_body, match_headers, match_queries, RequestParam};
use match_prompts::MatchPrompts;
use once_cell::sync::Lazy;
use std::io;
use std::io::Write;
use std::string::ToString;
use validators::{validate_param, validate_url};

fn print_completions<G: Generator>(gen: G, app: &mut App) {
  generate(gen, app, app.get_name().to_string(), &mut io::stdout());
}

fn build_request_args() -> impl Iterator<Item = &'static Arg<'static>> {
  static ARGS: Lazy<[Arg<'static>; 8]> = Lazy::new(|| {
    [
      Arg::new("url")
        .help("url to request, can be a 'Tera' template")
        .required(true)
        .validator(validate_url),
      Arg::new("header")
        .short('H')
        .long("header")
        .help("set header name:value to send with request")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Header)),
      Arg::new("cookie")
        .short('c')
        .long("cookie")
        .help("set cookie name:value to send with request")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Cookie)),
      Arg::new("query")
        .short('q')
        .long("query")
        .help("set query name:value to send with request")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Query)),
      Arg::new("body")
        .short('b')
        .long("body")
        .help("set body to send with request, can be a 'Tera' template")
        .takes_value(true)
        .conflicts_with("file"),
      Arg::new("file")
        .short('f')
        .long("file")
        .help("set body from file to send with request, can be a 'Tera' template")
        .takes_value(true)
        .conflicts_with("body")
        .value_hint(ValueHint::FilePath),
      Arg::new("variable")
        .short('e')
        .long("env")
        .help("set variable name:value for 'Tera' template rendering")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Variable)),
      Arg::new("insecure")
        .help("allow insecure connections when using https")
        .short('i')
        .long("insecure"),
    ]
  });
  ARGS.iter()
}

fn build_exec_args() -> impl Iterator<Item = &'static Arg<'static>> {
  static EXEC_ARGS: Lazy<[Arg<'static>; 1]> = Lazy::new(|| {
    [Arg::new("file")
      .help("path to the manifest file request to execute")
      .short('f')
      .long("file")
      .required(true)
      .takes_value(true)
      .value_hint(ValueHint::FilePath)]
  });
  EXEC_ARGS.iter()
}

fn build_create_request_args() -> impl Iterator<Item = &'static Arg<'static>> {
  static CREATE_ARGS: Lazy<[Arg<'static>; 10]> = Lazy::new(|| {
    [
      Arg::new("name").help("name of request to create").index(1),
      Arg::new("method")
        .help("method of request to create")
        .possible_values(["GET", "POST", "PUT", "DELETE"])
        .ignore_case(true)
        .index(2),
      Arg::new("url")
        .help("url to request, can be a 'Tera' template")
        .validator(validate_url)
        .index(3),
      Arg::new("header")
        .short('H')
        .long("header")
        .help("set header name:value to send with request")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Header)),
      Arg::new("cookie")
        .short('c')
        .long("cookie")
        .help("set cookie name:value to send with request")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Cookie)),
      Arg::new("query")
        .short('q')
        .long("query")
        .help("set query name:value to send with request")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Query)),
      Arg::new("body")
        .short('b')
        .long("body")
        .help("set body to send with request, can be a 'Tera' template")
        .takes_value(true)
        .conflicts_with("file"),
      Arg::new("file")
        .short('f')
        .long("file")
        .help("set body from file to send with request, can be a 'Tera' template")
        .takes_value(true)
        .conflicts_with("body")
        .value_hint(ValueHint::FilePath),
      Arg::new("variable")
        .short('e')
        .long("env")
        .help("set variable name:value for 'Tera' template rendering")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Variable)),
      Arg::new("insecure")
        .help("allow insecure connections when using https")
        .short('i')
        .long("insecure"),
    ]
  });
  CREATE_ARGS.iter()
}

fn build_cli() -> App<'static> {
  App::new("apix")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .version(crate_version!())
    .author(crate_authors!())
    .args([Arg::new("verbose")
      .help("print full request and response")
      .short('v')
      .long("verbose")
      .global(true)])
    .subcommands([
      App::new("completions").about("generate shell completions").arg(
        Arg::new("shell")
          .help("shell to target for completions")
          .possible_values(Shell::possible_values())
          .required(true),
      ),
      App::new("config")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("configuration settings")
        .subcommands([
          App::new("list"),
          App::new("set").about("set configuration value").args([
            Arg::new("name")
              .help("name of configuration value to set")
              .required(true)
              .index(1),
            Arg::new("value")
              .help("value to set configuration value to")
              .required(true)
              .index(2),
          ]),
          App::new("get").about("get a configuration value").arg(
            Arg::new("name")
              .help("name of configuration value to get")
              .required(true),
          ),
          App::new("delete").about("delete a configuration value").arg(
            Arg::new("name")
              .help("name of configuration value to delete")
              .required(true),
          ),
        ]),
      App::new("init").about("initialise a new API context in the current directory by using git"),
      App::new("history").about("show history of requests sent (require project)"),
      App::new("get").about("get an http resource").args(build_request_args()),
      App::new("head").args(build_request_args()),
      App::new("post")
        .about("post to an http resource")
        .args(build_request_args()),
      App::new("delete")
        .about("delete an http resource")
        .args(build_request_args()),
      App::new("put")
        .about("put to an http resource")
        .args(build_request_args()),
      App::new("patch")
        .about("patch an http resource")
        .args(build_request_args()),
      App::new("exec")
        .about("execute a request from the current API context")
        .args(build_exec_args()),
      App::new("ctl")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .about("apix control interface for handling multiple APIs")
        .subcommands([
          App::new("switch").about("switch API context"),
          App::new("apply").about("apply an apix manifest into current project"),
          App::new("create")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .about("create a new apix manifest")
            .subcommands([
              App::new("request")
                .about("create a new request")
                .args(build_create_request_args()),
              App::new("story").about("create a new story"),
              // .args(build_create_story_args()),
            ]),
          App::new("edit")
            .about("edit an existing apix resource with current terminal EDITOR")
            .args([
              Arg::new("resource")
                .help("resource type to edit")
                .possible_values(["resource", "context", "story", "request", "config"])
                .required(true)
                .index(1),
              Arg::new("name")
                .help("name of apix resource to edit")
                .required(true)
                .index(2),
            ]),
          App::new("get")
            .about("get information about an apix resource")
            .arg(Arg::new("resource").possible_values(["resource", "context", "story", "request"])),
          App::new("delete").about("delete an existing named resource").args([
            Arg::new("resource")
              .help("resource type to delete")
              .possible_values(["resource", "context", "story", "request"])
              .required(true)
              .index(1),
            Arg::new("name")
              .help("name of apix resource to delete")
              .required(true)
              .index(2),
          ]),
          App::new("import")
            .about("import an OpenAPI description file in yaml or json")
            .arg(
              Arg::new("url")
                .help("Filename or URL to openApi description to import")
                .required(true),
            ),
        ]),
    ])
}

async fn handle_import(url: &str) -> Result<()> {
  // let open_api = reqwest::get(url).await?.text().await?;
  // let result = import::import_api(open_api, import::OpenApiType::YAML)
  //     .await
  //     .map_err(|e| anyhow::anyhow!("Invalid Open Api description\n{:#}", e))?;
  // println!("api {}", serde_json::to_string(&result)?);
  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  let matches = build_cli().get_matches();
  // read config file
  let theme = ApixConfiguration::once().get("theme").unwrap().clone();
  match matches.subcommand() {
    Some(("completions", matches)) => {
      if let Ok(generator) = matches.value_of_t::<Shell>("shell") {
        let mut app = build_cli();
        print_completions(generator, &mut app);
      }
    }
    Some(("init", _)) => {
      run_cmd! {git --version}.map_err(|_| anyhow!("git command not found"))?;
      // create .gitignore
      let mut gitignore =
        std::fs::File::create(".gitignore").map_err(|e| anyhow!("Failed to create .gitignore\ncause: {}", e))?;
      gitignore
        .write_all(b".apix/context.yaml\n")
        .map_err(|e| anyhow!("Failed to write to .gitignore\ncause: {}", e))?;
      gitignore
        .flush()
        .map_err(|e| anyhow!("Failed to save .gitignore\ncause: {}", e))?;
      // init git
      run_cmd! {
        git init
        git add .gitignore
        git commit -m "Apix init commit"
      }
      .map_err(|e| anyhow!("Failed to init apix repository\ncause: {}", e))?;
    }
    Some(("config", matches)) => match matches.subcommand() {
      Some(("list", _)) => {
        pretty_print(
          serde_yaml::to_string(ApixConfiguration::once())?.as_bytes(),
          &theme,
          "yaml",
        )?;
      }
      Some(("set", matches)) => match (matches.value_of("name"), matches.value_of("value")) {
        (Some(key), Some(value)) => {
          if let Some(old_value) = ApixConfiguration::once().set(key.to_string(), value.to_string()) {
            println!("Replaced config key");
            pretty_print(
              format!("-{}: {}\n+{}: {}\n", key, old_value, key, value).as_bytes(),
              &theme,
              "diff",
            )?;
          } else {
            println!("Set config key");
            pretty_print(format!("{}: {}\n", key, value).as_bytes(), &theme, "yaml")?;
          }
          ApixConfiguration::once().save()?;
        }
        _ => {}
      },
      Some(("get", matches)) => {
        let key = matches.value_of("name").unwrap();
        if let Some(value) = ApixConfiguration::once().get(key) {
          pretty_print(format!("{}: {}\n", key, value).as_bytes(), &theme, "yaml")?;
        }
      }
      Some(("delete", matches)) => {
        let key = matches.value_of("name").unwrap();
        if let Some(value) = ApixConfiguration::once().delete(key) {
          println!("Deleted config key");
          pretty_print(format!("{}: {}\n", key, value).as_bytes(), &theme, "yaml")?;
          ApixConfiguration::once().save()?;
        }
      }
      _ => {}
    },
    Some(("history", _submatches)) => {}
    Some(("exec", matches)) => {
      if let Some(file) = matches.value_of("file") {
        let content = std::fs::read_to_string(file)?;
        let manifest: ApixManifest = serde_yaml::from_str(&content)?;
        handle_execute(file, &manifest, &theme, matches.is_present("verbose")).await?;
      }
    }
    Some(("ctl", matches)) => match matches.subcommand() {
      Some(("apply", _submatches)) => {}
      Some(("create", matches)) => match matches.subcommand() {
        Some(("request", matches)) => {
          let name = matches.match_or_input("name", "Request Name")?;
          let methods = ["GET", "POST", "PUT", "DELETE"];
          let method = matches.match_or_select("method", "Request method", &methods)?;
          let url = matches.match_or_validate_input("url", "Request url", |url: &String| {
            validate_url(&url.to_owned()).map(|_| ())
          })?;
          let headers = matches.match_or_input_multiples("header", "Add request headers?")?;
          let queries = matches.match_or_input_multiples("query", "Add request query parameters?")?;

          let body = matches
            .match_or_optional_input("body", "Add a request body?")?
            .map(|body| serde_json::Value::String(body.to_owned()));

          let filename = format!("{}.yaml", &name);
          let request_manifest = ApixManifest::new_request(
            "test".to_string(),
            name,
            ApixRequest::new(
              vec![],
              indexmap! {},
              ApixRequestTemplate::new(method.to_string(), url, headers, queries, body),
            ),
          );
          let request_manifest_yaml = serde_yaml::to_string(&request_manifest)?;
          // save to file with name of request
          std::fs::write(filename, request_manifest_yaml)?;
        }
        Some(("story", _submatches)) => {}
        _ => {}
      },
      Some(("switch", _submatches)) => {}
      Some(("edit", matches)) => {}
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
        requests::make_request(
          &url,
          &method,
          match_headers(matches).as_ref(),
          match_queries(matches).as_ref(),
          match_body(matches)?,
          matches.is_present("verbose"),
          &theme,
        )
        .await?;
      }
    }
    _ => {}
  }
  Ok(())
}
