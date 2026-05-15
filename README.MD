# PIF - The (P)ackage Manager for (I)nteractive (F)iction

## What is it?

A CLI tool to find, install, and publish extensions for interactive fiction authoring systems:

- TADS 3
- Inform (7 and later)
- Inform 6
- Dialog

Written in Rust for platform independence, using [Clap](https://github.com/clap-rs/clap) for the CLI interface.

## Installation

### macOS (Homebrew)
```sh
brew install toerob/pif/pif
```

### macOS / Linux (shell installer)
```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/toerob/pif/releases/latest/download/pif-installer.sh | sh
```

### Windows
Download the latest `pif-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/toerob/pif/releases/latest), extract it, and place `pif.exe` somewhere on your `PATH`.

---

## Usage

```
pif [OPTIONS] <COMMAND>

Commands:
  info      Show information about an extension
  update    Fetch the latest extension registry
  install   Install an extension
  list      List available extensions
  tags      List all available tags
  publish   Publish an extension to the registry (interactive, opens a PR)
  registry  Manage the local installation registry
  help      Print this message or the help of the given subcommand(s)

Options:
      --system <SYSTEM>    [default: auto] [possible values: auto, all, tads3, inform, inform6, dialog, unknown]
      --color <COLOR>      [default: auto] [possible values: always, auto, never]
  -v, --verbose <VERBOSE>  Verbosity level 1-3 [default: 2]
```

### Common commands

Update the registry:
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

Filter by tag:
```
pif list -t puzzle
```

Target a specific system:
```
pif list --system tads3
```

List all available tags:
```
pif tags
pif tags --system inform
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

Install a specific version (colon-separated):
```
pif install t3cartographer:1.0.0
```

Install to a specific directory:
```
pif install t3cartographer -d libs/
```

**Default installation directories:**

| System | Default |
|--------|---------|
| Inform | macOS: `~/Library/Inform/Extensions/{Author}/` · Linux: `~/Inform/Extensions/{Author}/` · Windows: `My Documents\Inform\Extensions\{Author}\` |
| TADS 3 | Current directory (`./`) |
| Inform 6 | Current directory (`./`) |
| Dialog | Current directory (`./`) |

Inform extensions follow the Inform IDE's expected structure automatically. For other systems, use `-d` to point at your project's library folder.

No `--system` flag needed — pif detects the system from the extension's registry entry.

When a TADS 3 makefile (`.t3m`) is detected in the current directory, `pif install` will automatically add the required `-lib` / `-source` entries to it.

Publish your extension to the registry (interactive, opens a PR):
```
pif publish
pif publish /path/to/extension
```

### Registry

PIF keeps a local SQLite registry of where extensions have been installed on your machine, including the installed version.

```
pif registry list           # show all recorded installations
pif registry remove <name>  # remove a registry entry by name
pif registry remove <name> --path <path>  # restrict to a specific path
pif registry clean          # remove entries whose paths no longer exist
```

## Registry format

The registry is a directory tree hosted in a git repository:

```
registry/
  tads3/
    {namespace}/
      {package-id}/
        package.yaml
        releases/
          v1.0.0.yaml
  inform/
    {namespace}/
      {package-id}/
        package.yaml
        releases/
          v2-i10.1.yaml   ← Inform branch suffix
          v2.3.1.yaml
```

**package.yaml** describes the extension:

```yaml
schema-version: 1
id: t3cartographer
name: t3cartographer
author: Tomas Öberg
description: Generates a map from TADS 3 source code (SVG, HTML, DOT, ASCII output).
tags:
  - mapping
  - tads3
```

**releases/v1.0.0.yaml** describes a specific release:

```yaml
schema-version: 1
date: '2024-01-15'
source:
  url: https://github.com/toerob/t3cartographer.git
  format: git
  branch: main
build:
  exports:
    - type: lib
      path: t3cartographer/cartographer.tl
```

The `source.format` field can be `git`, `zip`, or a raw file extension (e.g. `h`, `t`, `i7x`).

For Inform releases, the filename suffix `-i{branch}` (e.g. `v2-i10.1.yaml`) identifies which Inform release branch the file targets.

The easiest way to submit an extension is `pif publish`, which walks you through the prompts, commits to a branch, and opens a GitHub compare URL for the PR.

## Disclaimer

Work in progress. No warranties — use at your own risk.

Copyright 2026 Tomas Öberg
