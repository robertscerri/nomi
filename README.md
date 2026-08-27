# nomi

`nomi` is a small cross-platform terminal UI for safely batch-renaming files and
directories. It previews every change before touching the filesystem and supports
both regular expressions and literal text replacement.

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

| Key | Action |
| --- | --- |
| `Tab` / `Shift+Tab` | Move between inputs and file list |
| `Left` / `Right`, `Home` / `End` | Move within an input |
| `Backspace` / `Delete` | Remove text before or after the cursor |
| `Ctrl+R` | Toggle regex/literal mode |
| `Up` / `Down`, `j` / `k` | Move through files |
| `Space` | Include or exclude the highlighted item |
| `Ctrl+A` | Select or clear all items |
| `Enter` | Review and confirm valid changes |
| `Esc` | Quit or close confirmation |

Regex replacement uses `$1` or `${name}` capture syntax. For example, the pattern
`^IMG_(\d+)\.jpg$` and replacement `holiday_$1.jpg` changes `IMG_001.jpg` to
`holiday_001.jpg`.

## Safety

`nomi` refuses duplicate destinations, existing destinations, invalid regular
expressions, and destination names containing path separators. Renames are staged
through unique temporary names, allowing swaps and cycles. If an operation fails,
`nomi` attempts to restore every staged item before reporting the error.

The initial release operates on the immediate children of one directory and does
not recurse.
