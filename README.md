# cargo-api

## Usage

```
Commands:
  crate     Print details for a specific crate
  search    Search for crates
  publish   Publish a crate version. Requires an API token
  owner     List, add, or remove the owners of a crate
  yank      Yank a crate version. Requires an API token
  unyank    Undo the yank of a crate version. Requires an API token
  download  Print the download URL of a crate version
  help      Print this message or the help of the given subcommand(s)

Options:
      --manifest-path <PATH>     Path to Cargo.toml
      --user-agent <USER_AGENT>  
      --token <TOKEN>            API token for endpoints which require authentication [env: CARGO_REGISTRY_TOKEN]
  -h, --help                     Print help
  -V, --version                  Print version

```

### crate endpoint: `cargo api crate <name>`

```
Usage: cargo api --user-agent <USER_AGENT> crate <name>
```

### search endpoint: `cargo api search [query]`

```
Usage: cargo api --user-agent <USER_AGENT> search [OPTIONS] [query]

Search options:
      --sort <SORT>          One of: alphabetical, relevance, downloads, recent-downloads, recent-updates, or new
      --page <PAGE>          
      --per-page <PER_PAGE>  
```

### publish endpoint: `cargo api publish --metadata <path> <crate-file>`

The metadata file is a JSON, of the `cargo publish` format, see
https://doc.rust-lang.org/cargo/reference/registry-web-api.html#publish.

```
Usage: cargo api --user-agent <USER_AGENT> publish --metadata <path> <crate-file>
```

### owners endpoint: `cargo api owner <list|add|remove> <name>`

```
Usage: cargo api --user-agent <USER_AGENT> owner list <name>
Usage: cargo api --user-agent <USER_AGENT> owner add <name> <owner>...
Usage: cargo api --user-agent <USER_AGENT> owner remove <name> <owner>...
```

### yank endpoint: `cargo api yank <name> <version>`

```
Usage: cargo api --user-agent <USER_AGENT> yank <name> <version>
```

### unyank endpoint: `cargo api unyank <name> <version>`

```
Usage: cargo api --user-agent <USER_AGENT> unyank <name> <version>
```

### download endpoint: `cargo api download <name> <version>`

```
Usage: cargo api --user-agent <USER_AGENT> download <name> <version>
```

## crates.io policy

https://crates.io/data-access#api
