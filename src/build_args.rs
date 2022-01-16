use super::match_params::RequestParam;
use super::validators::{validate_param, validate_url};
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ValueHint};
use clap_complete::Shell;
use once_cell::sync::Lazy;

pub fn build_request_args() -> impl Iterator<Item = &'static Arg<'static>> {
  static ARGS: Lazy<[Arg<'static>; 17]> = Lazy::new(|| {
    [
      Arg::new("url")
        .help("url to request, can be a 'Tera' template")
        .required(true)
        .value_hint(ValueHint::Url)
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
      Arg::new("param")
        .short('p')
        .long("param")
        .help("set parameter name:value for 'Tera' template rendering")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Param)),
      Arg::new("proxy")
        .help("set proxy url to use for request")
        .short('x')
        .long("proxy")
        .value_hint(ValueHint::Url)
        .takes_value(true),
      Arg::new("proxy-login")
        .help("set proxy login to use for request")
        .long("proxy-login")
        .takes_value(true),
      Arg::new("proxy-password")
        .help("set proxy password to use for request")
        .long("proxy-password")
        .takes_value(true),
      Arg::new("follow")
        .help("follow http redirects")
        .short('F')
        .long("follow"),
      Arg::new("max-redirects")
        .help("set max http redirects to follow")
        .long("max-redirects")
        .takes_value(true),
      Arg::new("timeout")
        .help("set request timeout in seconds")
        .long("timeout")
        .takes_value(true),
      Arg::new("user-agent")
        .help("set user agent to send with request")
        .long("user-agent")
        .takes_value(true),
      Arg::new("certificate")
        .help("add a custom certificate authority")
        .long("certificate")
        .takes_value(true)
        .multiple_occurrences(true)
        .value_hint(ValueHint::FilePath),
      Arg::new("bind-address")
        .help("set bind address for request")
        .long("bind-address")
        .takes_value(true),
      Arg::new("insecure")
        .help("allow insecure connections when using https")
        .long("insecure"),
    ]
  });
  ARGS.iter()
}

pub fn build_exec_args() -> impl Iterator<Item = &'static Arg<'static>> {
  static EXEC_ARGS: Lazy<[Arg<'static>; 6]> = Lazy::new(|| {
    [
      Arg::new("name").help("name of the request to execute").index(1),
      Arg::new("file")
        .help("Execute a manifest file request directly")
        .short('f')
        .long("file")
        .takes_value(true)
        .value_hint(ValueHint::FilePath)
        .conflicts_with("name"),
      Arg::new("param")
        .help("Set a parameter for the request")
        .short('p')
        .long("param")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Param)),
      Arg::new("proxy")
        .help("set proxy to use for request")
        .short('x')
        .long("proxy")
        .takes_value(true),
      Arg::new("proxy-login")
        .help("set proxy login to use for request")
        .long("proxy-login")
        .takes_value(true),
      Arg::new("proxy-password")
        .help("set proxy password to use for request")
        .long("proxy-password")
        .takes_value(true),
    ]
  });
  EXEC_ARGS.iter()
}

pub fn build_create_request_args() -> impl Iterator<Item = &'static Arg<'static>> {
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
      Arg::new("param")
        .short('p')
        .long("param")
        .help("set parameter name:value for 'Tera' template rendering")
        .multiple_occurrences(true)
        .takes_value(true)
        .validator(|param| validate_param(param, RequestParam::Param)),
      Arg::new("insecure")
        .help("allow insecure connections when using https")
        .short('i')
        .long("insecure"),
    ]
  });
  CREATE_ARGS.iter()
}

pub fn build_cli() -> App<'static> {
  App::new("apix")
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .version(crate_version!())
    .author(crate_authors!())
    .args([
      Arg::new("verbose")
        .help("print full request and response")
        .short('v')
        .long("verbose")
        .global(true),
      Arg::new("output-file")
        .help("output file")
        .short('o')
        .long("output-file")
        .takes_value(true)
        .value_hint(ValueHint::FilePath)
        .global(true),
    ])
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
      App::new("head")
        .about("get an http resource header")
        .args(build_request_args()),
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
                .index(1),
              Arg::new("name").help("name of apix resource to edit").index(2),
              Arg::new("file")
                .help("Edit a resource file directly")
                .short('f')
                .long("file")
                .takes_value(true)
                .value_hint(ValueHint::FilePath)
                .conflicts_with_all(&["resource", "name"]),
            ]),
          App::new("get").about("get information about an apix resource").args([
            Arg::new("resource")
              .possible_values(["resource", "context", "story", "request"])
              .index(1),
            Arg::new("name").help("name of apix resource to edit").index(2),
          ]),
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
