# PIF - The (P)ackage Manager for (I)nteractive (F)iction

# What is it?

A CLI tool to help catalogize, search and install extensions for various interactive fiction systems such as:

- Dialog
- Inform6
- Tads3

# Why is it?

The main reason for the existence of this tool is to make it possible to easily find and retrive extensions from various sources like ifarchive.org and github without ever having to leave the console application.

It's written in Rust to easily obtain platform independence, and also because it features a great CLI extension (Clap) to build upon.

# Usage

Just type:
`pif`

To get the following menu:

```
PIF - A CLI package manager for interactive fiction languages

Usage: pif [OPTIONS] <COMMAND>

Commands:
  info     Show information about an extension
  update   Get an update of all extensions
  install  Install an extension
  list     Lists available extensions
  help     Print this message or the help of the given subcommand(s)

Options:
      --system <SYSTEM>    System [default: auto] [possible values: auto, tads3, dialog, inform6, inform7, unknown]
      --color <COLOR>      Color [default: auto] [possible values: always, auto, never]
  -v, --verbose <VERBOSE>  Verbosity level 1-3 (TODO) [default: 2]
  -h, --help               Print help
  -V, --version            Print version
```

- Update the master list for all the available extensions: `pif update`

- List all extensions for a given system (auto detecting which system if in a project folder): `pif list`

- Filter out an extensions based on chars from the extension name (fuzzy search) : `pif list -k t3cr`

- Filter out an extensions based on chars from the author name (fuzzy search) : `pif list -a tö`

- List all extensions for a given system (manua): `pif list --system tads3`

- Display information about an extension: `pif info t3cartographer`

- Install an extension: `pif install t3cartographer` (The extension name must be correct but is case insensitive)

- Help on available commands: `pif help list` - will show all available options

# How to add one of your extensions to this tool?

Internally PIF works by keeping a master list of extensions in a global repository, it doen't host them itself (but could potentially do that as well). The urls for each extension can in practicallity point anywhere but _must_ either point to:

1.  a single unarchived file
2.  a zip-file
3.  a github repository link (ending in ".git")

In order to add an extension to the list just add a pull request (PR) to the list where the URL can either point to a zip archive on for instance ifarchive.org (e.g https://ifarchive.org/if-archive/programming/tads3/library/contributions/t3cartographer.zip), or
a git url (e.g https://github.com/toerob/t3cartographer.git)

The master list for each system can currently be found here:

- https://github.com/toerob/rift/blob/main/dialog-extensions.json

- https://github.com/toerob/rift/blob/main/inform6-extensions.json

- https://github.com/toerob/rift/blob/main/tads3-extensions.json

(It may be moved to a more suitable place later.)

# Example

A PR should include the following information for a completely new extension:

```
{
      "name": "t3cartographer",
      "type": [
        "adv3",
        "adv3lite"
      ],
      "author": "Tomas Öberg",
      "desc": "The Tads3 Cartographer library is tool that generates a map from your Tads3 source code that can be used either as an aid when developing your game or as an in-game map for the player. It supports various output formats. Depending on your solution and framework it can be a map in svg, html, dot-graphics or plain old ascii.",
      "versions": [
        {
          "last-modified": "2022-08-14",
          "makefile-entries": [
            "t3cartographer/cartographer.tl"
          ],
          "url": "https://github.com/toerob/t3cartographer.git",
          "ext": "zip",
          "branch": "master",
          "version": "0.7"
        }
      ],
      "tags": []
    },
```

If you just wish to add a separate version of your extension just add another object within the versions array seen above:

```
  {
    "last-modified": "2023-06-06",
    "makefile-entries": [
      "t3cartographer/cartographer.tl"
    ],
    "url": "https://github.com/toerob/t3cartographer-0.8.git",
    "ext": "zip",
    "branch": "master",
    "version": "0.8"
  }
```

Or replace the entry completely.

# Disclaimer

A work in progress as most of these tools are. It also comes with no warranties, use at your own risk.

Copyright @2023 Tomas Öberg
