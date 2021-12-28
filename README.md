# APIX

| warning                                                              |
| -------------------------------------------------------------------- |
| Apix is still in alpha/proof of concept state! Interface might brake |


APIX is a modern API fetcher for the command line.  
  
It brings ideas from tools like `Git`,`Kubernetes`, `Helm` ,`Curl`, `Httpie`.
Indeed APIX is not just an API fetcher, it can :
- store your request in file system in a readable format for later use (yaml)
- allows to version your requests naurally with git
- uses [Tera](https://tera.netlify.app/) template engine to make your requests (see [examples](/examples))
- uses [Dialoguer](https://docs.rs/dialoguer/latest/dialoguer/index.html) for interactive prompting you when you forget to supply requests parameters

Coming soon:  
- handle secrets stored in hashi corp vault
- import OpenAPI definition files to make api calls easier

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
> apix get https://jsonplaceholder.typicode.com/todos?_limit=3
or
> apix get https://jsonplaceholder.typicode.com/todos --query _limit:3
```

result:
```json
[
  {
    "userId": 1,
    "id": 1,
    "title": "delectus aut autem",
    "completed": false
  },
  {
    "userId": 1,
    "id": 2,
    "title": "quis ut nam facilis et officia qui",
    "completed": false
  },
  {
    "userId": 1,
    "id": 3,
    "title": "fugiat veniam minus",
    "completed": false
  }
]
```
you can also ask for verbose mode where apix will show you the full sended http request and response :
```bash
apix get -v https://jsonplaceholder.typicode.com/todos -q_limit:3
```

```http
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
x-powered-by: Express
x-ratelimit-limit: 1000
x-ratelimit-remaining: 999
x-ratelimit-reset: 1637414626
vary: Origin, Accept-Encoding
access-control-allow-credentials: true
cache-control: max-age=43200
pragma: no-cache
expires: -1
x-total-count: 200
access-control-expose-headers: X-Total-Count
x-content-type-options: nosniff
etag: W/"136-fTr038fftlG9yIOWHGimupdrQDg"
via: 1.1 vegur
cf-cache-status: HIT
age: 245
expect-ct: max-age=604800, report-uri="https://report-uri.cloudflare.com/cdn-cgi/beacon/expect-ct"
report-to: {"endpoints":[{"url":"https:\/\/a.nel.cloudflare.com\/report\/v3?s=CNFD1NndglvJcN9a1H3598U00GWBduBRMWYbNy2Cvy3QSws5PwaQGkHGmWkOuJ2gv%2FAmhUZkS3jPUc9VLF7sKQrtr2Rc%2FjdnmNP%2BbPZeMzGbA4yKcDakLQ7hGhDllBqUymlO5J2jcB%2BGBNETIoNnt8Q1mDdVJZhNC9cd"}],"group":"cf-nel","max_age":604800}
nel: {"success_fraction":0,"report_to":"cf-nel","max_age":604800}
server: cloudflare
cf-ray: 6b1ba3c97d633b85-CDG
alt-svc: h3=":443"; ma=86400, h3-29=":443"; ma=86400, h3-28=":443"; ma=86400, h3-27=":443"; ma=86400

[
  {
    "userId": 1,
    "id": 1,
    "title": "delectus aut autem",
    "completed": false
  },
  {
    "userId": 1,
    "id": 2,
    "title": "quis ut nam facilis et officia qui",
    "completed": false
  },
  {
    "userId": 1,
    "id": 3,
    "title": "fugiat veniam minus",
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