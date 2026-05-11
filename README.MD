# PIF - The (P)ackage Manager for (I)nteractive (F)iction

## What is it?

A CLI tool to find, install, and publish extensions for interactive fiction authoring systems:

- TADS 3
- Dialog
- Inform 6

Written in Rust for platform independence, using [Clap](https://github.com/clap-rs/clap) for the CLI interface.

## Usage

```
pif [OPTIONS] <COMMAND>

Commands:
  info      Show information about an extension
  update    Fetch the latest extension index
  install   Install an extension
  list      List available extensions
  publish   Publish an extension to the pif index
  registry  Manage the local installation registry
  help      Print this message or the help of the given subcommand(s)

Options:
      --system <SYSTEM>    [default: auto] [possible values: auto, tads3, dialog, inform6, inform7]
      --color <COLOR>      [default: auto] [possible values: always, auto, never]
  -v, --verbose <VERBOSE>  Verbosity level 1-3 [default: 2]
```

### Common commands

Update the index:
```
pif update
```

List extensions (auto-detects the IF system from the current directory):
```
pif list
```

Filter by extension name (fuzzy):
```
pif list -k cartographer
```

Filter by author (fuzzy):
```
pif list -a tomas
```

Target a specific system:
```
pif list --system tads3
```

Show details about an extension:
```
pif info t3cartographer
```

Install one or more extensions:
```
pif install t3cartographer
pif install t3cartographer conspace
```

Install a specific version (semver, colon-separated):
```
pif install t3cartographer:1.0
```

Install to a specific directory:
```
pif install t3cartographer -d libs/
```

When a makefile (`.t3m`) is detected in the current directory, `pif install` will offer to add the required build entries automatically.

Publish your extension to the index (interactive, opens a PR):
```
pif publish
pif publish /path/to/extension
```

### Registry

PIF keeps a local SQLite registry of where extensions have been installed on your machine.

```
pif registry list           # show all recorded installations
pif registry remove <name>  # remove a registry entry by name
pif registry remove <name> --path <path>  # restrict to a specific path
pif registry clean          # remove entries whose paths no longer exist
```

## Adding your extension to the index

The index is a YAML file hosted in a git repository. Each extension entry looks like this:

```yaml
name: t3cartographer
author: Tomas Öberg
desc: Generates a map from TADS 3 source code (SVG, HTML, DOT, ASCII output).
homepage: https://github.com/toerob/t3cartographer
versions:
- last-modified: 2024-01-15
  type:
  - adv3
  - adv3lite
  version: 1.0.0
  url: https://github.com/toerob/t3cartographer.git
  branch: master
  build-entries:
  - t3cartographer/cartographer.tl
```

The `url` field must point to one of:

1. A single file (any extension)
2. A ZIP archive
3. A GitHub repository URL ending in `.git`

The `build-entries` field lists directives that `pif install` will offer to add to the project's build file. Each entry is a complete directive line:

- TADS 3 library: `t3cartographer/cartographer.tl` (auto-prefixed with `-lib`)
- TADS 3 source: `mygame.t` (auto-prefixed with `-source`)
- Compiler define: `-D LANGUAGE=sv_se`
- Bare flag: `-D __DEBUG`

The easiest way to submit an extension is `pif publish`, which walks you through the prompts, commits to a branch, and opens a GitHub compare URL for the PR. To submit manually, fork the index repository and open a pull request.

## Disclaimer

Work in progress. No warranties — use at your own risk.

Copyright 2026 Tomas Öberg
