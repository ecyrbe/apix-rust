# APIX

APIX is a modern API fetcher for the command line.  
It brings ideas from tools like `Git`,`Kubernetes`, `Curl`, `Httpie`.
Indeed APIX is not just an API fetcher, it can :
- use OpenAPI definition files to make api calls easier
- store your request in file system for later use and versioning when combined with git
- handle secrets stored in hashi corp vault

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
