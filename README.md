<p align="center">
  <a href="https://github.com/ecyrbe/apix-rust">
    <img align="center" src="https://raw.githubusercontent.com/ecyrbe/apix-rust/main/docs/logo.svg" width="512px" alt="Apix logo">
  </a>
</p>

<p align="center">
   APIX is a modern HTTP client for the command line.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-2021-3584e4?logo=rust" alt="langue rust">
  <img src="https://img.shields.io/github/workflow/status/ecyrbe/apix-rust/Rust/main" alt="build status">
  <img src="https://img.shields.io/github/languages/code-size/ecyrbe/apix-rust?color=ffa348"& alt="code size">
  <a href="https://github.com/ecyrbe/apix-rust/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/ecyrbe/apix-rust?color=ed333b" alt="MIT license">
  </a>
</p>

```diff
- WARNING: Apix is still in alpha/proof of concept state! Interface might brake -
```

Apix brings ideas from tools like `Git`,`Kubernetes`, `Helm` ,`Httpie`.
Indeed it's is not just a simple HTTP client, Apix is :  
- **Pretty** as it uses [Bat](https://github.com/sharkdp/bat) to pretty print requests and responses
```bash
> apix get https://apix.io/json
{
  "id": 0,
  "test": "hello"
}
```
- **Beatifull** as it uses [Indicatif](https://docs.rs/indicatif/latest/indicatif/index.html) to show modern command line progress bars when uploading or downloading files
```bash
> apix get https://apix.io/test.mp4
Downloading File test.mp4
⠙ [00:00:28] [████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 14.98MiB/298.06MiB (549.87KiB/s, 8m)
```
- **Friendly** to use as it uses [Dialoguer](https://docs.rs/dialoguer/latest/dialoguer/index.html) for interactive prompt to guide you when creating requests or executing them
```bash
> apix exec -f request.yaml
✔ todoId · 1
? email ("ecyrbe@gmail.com") › bad\gmail.com
✘ Invalid input:
cause 0: "bad\gmail.com" is not a "email"
```
- **Powerfull** as it uses [Tera](https://tera.netlify.app/) template engine to allow your requests to do complex things if you want them to (see [examples](/examples))
- **Easy** to use in the command line as it uses [Clap](https://docs.rs/clap/latest/clap/) autocompletion
- **Reusable** as it stores your requests in your file system in a readable format for later use (yaml)
- **Helping you not forget** as it stores the response of the requests so you can consult them at any time
- **Helping you test APIs** as it allows you to create request stories (chain requests, use results of a request to make another one)
- **Team player** as it allows you to naturally share your request collections with git

Coming soon:  
- **Secure** as it handles secrets stored in hashi corp vault
- **Enterprise friend** as it allows you to import OpenAPI definition files into apix requests files

## help

```bash
apix 0.1.0

USAGE:
    apix [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    print full request and response
    -V, --version    Print version information

SUBCOMMANDS:
    completions    generate shell completions
    config         configuration settings
    ctl            apix control interface for handling multiple APIs
    delete         delete an http resource
    exec           execute a request from the current API context
    get            get an http resource
    head           get an http resource
    help           Print this message or the help of the given subcommand(s)
    history        show history of requests sent (require project)
    patch          patch an http resource
    post           post to an http resource
    put            put to an http resource
```

## make simple http requests

Even if Apix allows you to use advanced mode by coupling it to a git repository and interpret openapi declarations (swagger), you also can use Apix as a replacement for curl, wget, httpie ...  
  
Apix will colorize the output according to http **content-type** header information.  
  
By default Apix will assume you are doing an API request using json.

```bash
> apix get https://jsonplaceholder.typicode.com/todos?_limit=1
or
> apix get https://jsonplaceholder.typicode.com/todos --query _limit:1

[
  {
    "userId": 1,
    "id": 1,
    "title": "delectus aut autem",
    "completed": false
  }
]
```
you can also ask for verbose mode where apix will show you the full sended http request and response :
```bash
> apix get -v https://jsonplaceholder.typicode.com/todos -q_limit:1
GET /todos?_limit=3 HTTP/1.1
host: jsonplaceholder.typicode.com
user-agent: apix/0.1.0
accept: application/json
accept-encoding: gzip
content-type: application/json

HTTP/1.1 200 OK
date: Sun, 21 Nov 2021 17:29:22 GMT
content-type: application/json; charset=utf-8
transfer-encoding: chunked
connection: keep-alive
access-control-allow-credentials: true
cache-control: max-age=43200
pragma: no-cache
expires: -1
etag: W/"136-fTr038fftlG9yIOWHGimupdrQDg"

[
  {
    "userId": 1,
    "id": 1,
    "title": "delectus aut autem",
    "completed": false
  }
]
```

## Context

Apix handle contexts gracefully. Contexts are named resources to handle:
- API bindings (based on openAPI)
- Session authentification
- Session cookies
- variables bindings that can be used with templates

### Create a new context

```bash
apix ctl init MyContext
```

### Switching to another context

```bash
apix ctl switch OtherContext
```

### List all contexts

```bash
apix ctl get contexts
```

### Delete a context

```bash
apix ctl delete MyContext
```

## apix commands

### apix get

```bash
apix-get 

get an http resource

USAGE:
    apix get [OPTIONS] <url>

ARGS:
    <url>    url to request, can be a 'Tera' template

OPTIONS:
    -b, --body <body>        set body to send with request, can be a 'Tera' template
    -c, --cookie <cookie>    set cookie name:value to send with request
    -e, --env <variable>     set variable name:value for 'Tera' template rendering
    -f, --file <file>        set body from file to send with request, can be a 'Tera' template
    -h, --help               Print help information
    -H, --header <header>    set header name:value to send with request
    -i, --insecure           allow insecure connections when using https
    -q, --query <query>      set query name:value to send with request
    -v, --verbose            print full request and response
```
# Persistance

|   type   | persist mode | gitignore |               description               |
| :------: | :----------: | :-------: | :-------------------------------------: |
|  config  |     file     |    no     |             from cli config             |
| requests |     file     |    no     |
|  params  |     file     |    yes    |        auto saved from dialoguer        |
| results  |     file     |    yes    |       auto saved after execution        |
| cookies  |     file     |    yes    | auto saved after execution, auto reused |
| storage  |     file     |    yes    |       saved from request response       |
| secrets  |     web      |    N/A    |          from hashi corp vault          |
