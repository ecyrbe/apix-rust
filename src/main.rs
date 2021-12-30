mod dialog;
mod execute;
mod http_display;
mod http_utils;
mod import;
mod manifests;
mod match_params;
mod requests;
mod template;
mod validators;
use anyhow::Result;
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ValueHint};
use clap_generate::{generate, Generator, Shell};
use execute::handle_execute;
use http_display::pretty_print;
use lazy_static::lazy_static;
use manifests::{ApixConfiguration, ApixKind, ApixManifest};
use match_params::{match_body, match_headers, match_queries, RequestParam};
use std::io;
use std::string::ToString;
use std::sync::Mutex;
use validators::{validate_param, validate_url};

fn print_completions<G: Generator>(gen: G, app: &mut App) {
  generate(gen, app, app.get_name().to_string(), &mut io::stdout());
}

fn build_request_args() -> impl Iterator<Item = &'static Arg<'static>> {
  lazy_static! {
    static ref ARGS: [Arg<'static>; 8] = [
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
    ];
  }
  ARGS.iter()
}

fn build_exec_args() -> impl Iterator<Item = &'static Arg<'static>> {
  lazy_static! {
    static ref EXEC_ARGS: [Arg<'static>; 1] = [Arg::new("file")
      .help("path to the manifest file request to execute")
      .short('f')
      .long("file")
      .required(true)
      .takes_value(true)
      .value_hint(ValueHint::FilePath)];
  }
  EXEC_ARGS.iter()
}

fn build_create_request_args() -> impl Iterator<Item = &'static Arg<'static>> {
  lazy_static! {
    static ref CREATE_ARGS: [Arg<'static>; 9] = [
      Arg::new("name")
        .help("name of request to create")
        .required(true)
        .index(1),
      Arg::new("url")
        .help("url to request, can be a 'Tera' template")
        .required(true)
        .validator(validate_url)
        .index(2),
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
        .help("set body from file to send with request, can be a 'Tera' template",)
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
    ];
  }
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
          App::new("apply").about("apply an apix manifest into current project"),
          App::new("create")
            .about("create a new apix manifest")
            .subcommands([App::new("request")
              .about("create a new request")
              .args(build_create_request_args())]),
          App::new("init").about("initialise a new API context"),
          App::new("switch").about("switch API context"),
          App::new("edit").about("edit an existing apix resource with current terminal EDITOR"),
          App::new("get")
            .about("get information about an apix resource")
            .arg(Arg::new("resource").possible_values(["resource", "context", "request", "session"])),
          App::new("delete")
            .about("delete an existing named resource")
            .args([Arg::new("resource"), Arg::new("name")]),
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

// load configuration as a lazy static varibale
lazy_static! {
  static ref CONFIG: Mutex<ApixConfiguration> = Mutex::new(ApixConfiguration::load().unwrap());
}

#[tokio::main]
async fn main() -> Result<()> {
  let matches = build_cli().get_matches();
  // read config file
  let theme = CONFIG.lock().unwrap().get("theme").unwrap().clone();
  match matches.subcommand() {
    Some(("completions", matches)) => {
      if let Ok(generator) = matches.value_of_t::<Shell>("shell") {
        let mut app = build_cli();
        print_completions(generator, &mut app);
      }
    }
    Some(("config", matches)) => match matches.subcommand() {
      Some(("list", _)) => {
        pretty_print(
          serde_yaml::to_string(&CONFIG.lock().unwrap().clone())?.as_bytes(),
          &theme,
          "yaml",
        )?;
      }
      Some(("set", matches)) => match (matches.value_of("name"), matches.value_of("value")) {
        (Some(key), Some(value)) => {
          if let Some(old_value) = CONFIG.lock().unwrap().set(key.to_string(), value.to_string()) {
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
          CONFIG.lock().unwrap().save()?;
        }
        _ => {}
      },
      Some(("get", matches)) => {
        let key = matches.value_of("name").unwrap();
        if let Some(value) = CONFIG.lock().unwrap().get(key) {
          pretty_print(format!("{}: {}\n", key, value).as_bytes(), &theme, "yaml")?;
        }
      }
      Some(("delete", matches)) => {
        let key = matches.value_of("name").unwrap();
        if let Some(value) = CONFIG.lock().unwrap().delete(key) {
          println!("Deleted config key");
          pretty_print(format!("{}: {}\n", key, value).as_bytes(), &theme, "yaml")?;
          CONFIG.lock().unwrap().save()?;
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
        let body = Some(match_body(matches)?).filter(|body| body.is_empty());
        requests::make_request(
          &url,
          &method,
          match_headers(matches).as_ref(),
          match_queries(matches).as_ref(),
          body,
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
