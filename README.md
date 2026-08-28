# nomi

`nomi` is a small cross-platform terminal UI for batch-renaming files and
directories. It previews selected changes before touching the filesystem and
supports both regular expressions and literal text replacement.

## Install and run

```sh
cargo install --path .
nomi
```

By default, `nomi` opens the current working directory. You may also pass a directory:

```sh
nomi path/to/photos
```

## Controls

| Key                              | Action                                    |
| -------------------------------- | ----------------------------------------- |
| `Tab` / `Shift+Tab`              | Move between inputs and file list         |
| `Left` / `Right`, `Home` / `End` | Move within an input                      |
| `Backspace` / `Delete`           | Remove text before or after the cursor    |
| `Ctrl+R` (`Cmd+R` on macOS)      | Toggle regex/literal mode                 |
| `Up` / `Down`                    | Move through files                        |
| `Space`                          | Include or exclude the highlighted item   |
| `Ctrl+A` (`Cmd+A` on macOS)      | Select or clear all items                 |
| `Enter`                          | Open confirmation; press again to execute |
| `Esc`                            | Cancel confirmation, or quit normally     |

Regex replacement uses `$1` or `${name}` capture syntax. For example, the pattern
`^IMG_(\d+)\.jpg$` and replacement `holiday_$1.jpg` changes `IMG_001.jpg` to
`holiday_001.jpg`.

The file list shows each changed source and its destination. A malformed regular
expression replaces the preview with an error until the pattern is valid again.
Only selected entries whose destination differs from their source are executed.

## Execution and limitations

Before execution, `nomi` requires confirmation and rejects empty destinations or
destinations containing path components. Renames are then performed directly and
sequentially with the platform filesystem API.

Destination conflicts, swaps, cycles, and rollback are not currently handled. A
failure can therefore leave earlier entries renamed while later entries remain
unchanged; the directory is reloaded and the error is displayed. Behavior when a
destination already exists may differ by platform.

`nomi` operates only on the immediate children of one directory and does not
recurse.
