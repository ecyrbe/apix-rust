mod build_args;
mod dialog;
mod display;
mod editor;
mod execute;
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
use build_args::build_cli;
use clap::App;
use clap_complete::{generate, Generator, Shell};
use cmd_lib::run_cmd;
use display::{pretty_print, pretty_print_file};
use editor::edit_file;
use execute::handle_execute;
use indexmap::indexmap;
use manifests::{ApixConfiguration, ApixKind, ApixManifest, ApixRequest, ApixRequestTemplate};
use match_params::{MatchParams, RequestParam};
use match_prompts::MatchPrompts;
use requests::RequestOptions;
use std::io;
use std::io::Write;
use std::string::ToString;
use validators::validate_url;

fn print_completions<G: Generator>(gen: G, app: &mut App) {
  generate(gen, app, app.get_name().to_string(), &mut io::stdout());
}

async fn handle_import(_url: &str) -> Result<()> {
  // let open_api = reqwest::get(url).await?.text().await?;
  // let result = import::import_api(open_api, import::OpenApiType::YAML)
  //     .await
  //     .map_err(|e| anyhow::anyhow!("Invalid Open Api description\n{:#}", e))?;
  // println!("api {}", serde_json::to_string(&result)?);
  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  let is_output_terminal = atty::is(atty::Stream::Stdout);
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
          serde_yaml::to_string(ApixConfiguration::once())?,
          &theme,
          "yaml",
          is_output_terminal,
        )?;
      }
      Some(("set", matches)) => {
        if let (Some(key), Some(value)) = (matches.value_of("name"), matches.value_of("value")) {
          if let Some(old_value) = ApixConfiguration::once().set(key.to_string(), value.to_string()) {
            println!("Replaced config key");
            pretty_print(
              format!("-{}: {}\n+{}: {}\n", key, old_value, key, value),
              &theme,
              "diff",
              is_output_terminal,
            )?;
          } else {
            println!("Set config key");
            pretty_print(format!("{}: {}\n", key, value), &theme, "yaml", is_output_terminal)?;
          }
          ApixConfiguration::once().save()?;
        }
      }
      Some(("get", matches)) => {
        let key = matches.value_of("name").unwrap();
        if let Some(value) = ApixConfiguration::once().get(key) {
          pretty_print(format!("{}: {}\n", key, value), &theme, "yaml", is_output_terminal)?;
        }
      }
      Some(("delete", matches)) => {
        let key = matches.value_of("name").unwrap();
        if let Some(value) = ApixConfiguration::once().delete(key) {
          println!("Deleted config key");
          pretty_print(format!("{}: {}\n", key, value), &theme, "yaml", is_output_terminal)?;
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
        handle_execute(
          file,
          &manifest,
          matches.match_params(RequestParam::Param),
          RequestOptions {
            verbose: matches.is_present("verbose"),
            theme: &theme,
            is_output_terminal,
            output_filename: matches.value_of("output-file").map(str::to_string),
            proxy_url: matches.value_of("proxy").map(str::to_string),
            proxy_login: matches.value_of("proxy-login").map(str::to_string),
            proxy_password: matches.value_of("proxy-password").map(str::to_string),
          },
        )
        .await?;
      } else if let Ok(name) = matches.match_or_input("name", "Request name") {
        match ApixManifest::find_manifest("request", &name) {
          Some((path, manifest)) => {
            let path = path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
            handle_execute(
              path,
              &manifest,
              matches.match_params(RequestParam::Param),
              RequestOptions {
                verbose: matches.is_present("verbose"),
                theme: &theme,
                is_output_terminal,
                output_filename: matches.value_of("output-file").map(str::to_string),
                proxy_url: matches.value_of("proxy").map(str::to_string),
                proxy_login: matches.value_of("proxy-login").map(str::to_string),
                proxy_password: matches.value_of("proxy-password").map(str::to_string),
              },
            )
            .await?;
          }
          None => {
            println!("No request where found with name {}", name);
          }
        }
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
            .map(serde_json::Value::String);

          let filename = format!("{}.yaml", &name);
          let request_manifest = ApixManifest::new_request(
            "test".to_string(),
            name,
            ApixRequest::new(
              vec![],
              indexmap! {},
              ApixRequestTemplate::new(method, url, headers, queries, body),
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
      Some(("edit", matches)) => {
        if let Some(filename) = matches.value_of("file") {
          edit_file(filename)?;
        } else {
          let resource = matches.match_or_select("resource", "Resource type", &["request", "story"])?;
          let name = matches.match_or_input("name", "Resource name")?;
          match ApixManifest::find_manifest_filename(&resource, &name) {
            Some(filename) => {
              edit_file(&filename)?;
            }
            None => {
              println!("No resource of type {} where found with name {}", resource, name);
            }
          }
        }
      }
      Some(("get", matches)) => {
        if let Some(kind) = matches.value_of("resource") {
          if let Some(name) = matches.value_of("name") {
            if let Some((path, _)) = ApixManifest::find_manifest(kind, name) {
              pretty_print_file(path, &theme, "yaml", is_output_terminal)?;
            } else {
              println!("No resource of type {} where found with name {}", kind, name);
            }
          } else if let Ok(manifests) = ApixManifest::find_manifests_by_kind(kind) {
            let mut printed = false;
            for (path, _) in manifests {
              pretty_print_file(path, &theme, "yaml", is_output_terminal)?;
              printed = true;
            }
            if !printed {
              println!("No resources of type {} where found", kind);
            }
          } else {
            println!("No resources of type {} where found", kind);
          }
        }
      }
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
          url,
          method,
          matches.match_headers().as_ref(),
          matches.match_params(RequestParam::Query).as_ref(),
          matches.match_body(),
          RequestOptions {
            verbose: matches.is_present("verbose"),
            theme: &theme,
            is_output_terminal,
            output_filename: matches.value_of("output-file").map(str::to_string),
            proxy_url: matches.value_of("proxy").map(str::to_string),
            proxy_login: matches.value_of("proxy-login").map(str::to_string),
            proxy_password: matches.value_of("proxy-password").map(str::to_string),
          },
        )
        .await?;
      }
    }
    _ => {}
  }
  Ok(())
}
