# PIF Configuration Reference

PIF stores its configuration in a YAML file:

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/pif/config/config.yaml` |
| Linux    | `~/.config/pif/config/config.yaml` |
| Windows  | `%APPDATA%\pif\config\config.yaml` |

The file is created automatically on first run with commented-out examples for every option.

---

## Config keys

### `install_dirs`

Override the default installation directory on a per-system basis. Without this, Inform uses the platform Extensions folder and every other system uses the current directory.

```yaml
install_dirs:
  tads3: ~/projects/tads3/lib
  dialog: ~/projects/dialog/lib
  inform: ~/Library/Inform/Extensions
  inform6: ~/projects/inform6/lib
```

Leading `~/` is expanded to the user's home directory. The directory is created automatically by `pif config dir set`.

---

### `verbose_level`

Controls how much detail `list` and `search` print per extension.

| Level | Output |
|-------|--------|
| `1`   | Name and version only |
| `2`   | + Author *(default)* |
| `3`   | + Description |

```yaml
verbose_level: 2
```

---

### `systems`

Restricts `list`, `search`, and `tags` to a specific set of systems when `--system` is not given on the command line **and** no IF project is detected in the current directory.

Useful when you work across multiple IF systems and want a consistent default view without having to pass `--system` every time.

```yaml
systems: [tads3, inform6, inform]
```

`--system all` on the command line always overrides this setting. If pif auto-detects a project (e.g. a `.t3m` makefile), the detected system takes precedence over `systems` for `list` and `tags`.

**Valid system names:** `tads3`, `tads2`, `dialog`, `inform`, `inform6`, `hugo`, `zil`

---

### `system_versions`

Pins which versions are considered for each system. Applies to `list` and `search` (controls which releases are shown and what counts as "latest") and to `install` (restricts which versions are eligible when no explicit version is given).

```yaml
system_versions:
  inform: [i10, i11.0]
  tads3: [3.1, 3.2]
```

Each entry in the list is a **matcher string**. Currently supported:

| Form | Behaviour | Example |
|------|-----------|---------|
| Bare string | Prefix match | `i10` matches `i10.1`, `i10.2`, … |

Future matcher syntax (not yet active): `>=`, `<`, `^`, `~` constraint expressions — same list, no config change needed when that lands.

Items within the list are OR-combined: a version matches if **any** entry matches it.

An explicit version pin on the command line (`pif install some-ext:i10.1`) always bypasses this filter.

---

## CLI reference

All configuration is managed with `pif config <category> <action>`. Run `pif config --help` or `pif config <category> --help` at any level for usage.

### `pif config show`

Display the current config file with syntax highlighting, together with its path:

```sh
pif config show
pif config show --color never   # plain text
```

---

### `pif config dir`

Manage per-system installation directories.

```sh
pif config dir set <system> <path>    # set a custom directory
pif config dir reset <system>         # reset one system to its default
pif config dir reset                  # reset all systems to their defaults
```

Examples:
```sh
pif config dir set tads3 ~/projects/tads3/lib
pif config dir set inform "~/Library/Inform/Extensions"
pif config dir reset tads3
pif config dir reset
```

---

### `pif config verbose`

Manage the verbosity level used by `list` and `search`.

```sh
pif config verbose set <level>   # 1, 2, or 3
pif config verbose reset         # back to default (2)
```

---

### `pif config systems`

Manage the systems filter applied when `--system` is not given and no IF project is detected.

```sh
pif config systems set <system> [<system>…]   # replace the entire list
pif config systems add <system> [<system>…]   # add to the existing list
pif config systems remove <system> [<system>…] # remove from the list
pif config systems reset                       # remove the filter entirely
```

Examples:
```sh
pif config systems set tads3 inform inform6
pif config systems add hugo
pif config systems remove inform6
pif config systems reset
```

After any `systems` change, the config file is shown with the affected values highlighted. Pass `--system all` on the command line to bypass the filter entirely. Duplicates are ignored on `add`.

---

### `pif config versions`

Manage per-system version specs. Controls which releases are shown and what counts as "latest" for `list`, `search`, and `install`.

```sh
pif config versions set <system> <spec> [<spec>…]    # replace the list for a system
pif config versions add <system> <spec> [<spec>…]    # add specs to a system
pif config versions remove <system> <spec> [<spec>…] # remove specific specs
pif config versions reset <system>                    # clear all specs for a system
pif config versions reset                             # clear all systems
```

Examples:
```sh
# Set Inform to branches i10 and i11.0
pif config versions set inform i10 i11.0

# Later, also allow i12
pif config versions add inform i12

# Drop i10 without touching i11.0 or i12
pif config versions remove inform i10

# Only consider TADS 3 releases in the 3.x line
pif config versions set tads3 3.

# Clear the Inform constraint
pif config versions reset inform

# Clear everything
pif config versions reset
```

After any `versions` change, the config file is shown with the affected values highlighted. An explicit version pin on the command line (`pif install some-ext:i10.1`) always bypasses this filter. Duplicates are ignored on `add`.

---

## Full example config

```yaml
main_repository_url: https://github.com/toerob/if-extensions
main_repository_branch: main

install_dirs:
  tads3: ~/projects/tads3/lib
  inform: ~/Library/Inform/Extensions

verbose_level: 3

systems: [tads3, inform]

system_versions:
  inform: [i10, i11.0]
```
