mod http_display;
mod http_utils;
mod import;
mod manifests;
mod match_params;
mod requests;
mod validators;
use anyhow::Result;
use clap::{crate_version, App, AppSettings, Arg, ValueHint};
use clap_generate::{generate, Generator, Shell};
use http_display::pretty_print;
use lazy_static::lazy_static;
use manifests::ApixConfiguration;
use match_params::{match_body, match_headers, match_queries, RequestParam};
use std::io;
use std::string::ToString;
use validators::{validate_param, validate_url};

fn print_completions<G: Generator>(gen: G, app: &mut App) {
    generate(gen, app, app.get_name().to_string(), &mut io::stdout());
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
                    App::new("get").about("get a configuration value").arg(
                        Arg::new("name")
                            .about("name of configuration value to get")
                            .required(true),
                    ),
                    App::new("delete")
                        .about("delete a configuration value")
                        .arg(
                            Arg::new("name")
                                .about("name of configuration value to delete")
                                .required(true),
                        ),
                ]),
            App::new("history").about("show history of requests sent (require project)"),
            App::new("get")
                .about("get an http ressource")
                .args(build_request_args()),
            App::new("head").args(build_request_args()),
            App::new("post")
                .about("post to an http ressource")
                .args(build_request_args()),
            App::new("delete")
                .about("delete an http ressource")
                .args(build_request_args()),
            App::new("put")
                .about("put to an http ressource")
                .args(build_request_args()),
            App::new("patch")
                .about("patch an http ressource")
                .args(build_request_args()),
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
        .map_err(|e| anyhow::anyhow!("Invalid Open Api description\n{}", e))?;
    println!("api {}", serde_json::to_string(&result)?);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = build_cli().get_matches();
    // read config file
    let mut config = ApixConfiguration::load()?;
    let theme = config.get("theme").unwrap().clone();
    match matches.subcommand() {
        Some(("completions", matches)) => {
            if let Ok(generator) = matches.value_of_t::<Shell>("shell") {
                let mut app = build_cli();
                print_completions(generator, &mut app);
            }
        }
        Some(("config", matches)) => match matches.subcommand() {
            Some(("list", _)) => {
                pretty_print(serde_yaml::to_string(&config)?.as_bytes(), &theme, "yaml")?;
            }
            Some(("set", matches)) => match (matches.value_of("name"), matches.value_of("value")) {
                (Some(key), Some(value)) => {
                    if let Some(old_value) = config.set(key.to_string(), value.to_string()) {
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
                    config.save()?;
                }
                _ => {}
            },
            Some(("get", matches)) => {
                let key = matches.value_of("name").unwrap();
                if let Some(value) = config.get(key) {
                    pretty_print(format!("{}: {}", key, value).as_bytes(), &theme, "yaml")?;
                }
            }
            Some(("delete", matches)) => {
                let key = matches.value_of("name").unwrap();
                if let Some(value) = config.delete(key) {
                    println!("Deleted config key");
                    pretty_print(format!("{}: {}", key, value).as_bytes(), &theme, "yaml")?;
                    config.save()?;
                }
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
                requests::make_request(
                    &url,
                    &method,
                    &match_headers(matches).unwrap_or_default(),
                    &match_queries(matches).unwrap_or_default(),
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
