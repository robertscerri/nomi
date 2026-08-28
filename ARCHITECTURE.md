# Architecture

`nomi` is split into a reusable core library and a terminal UI binary. The
library reads directory entries and reports shared errors; the binary owns user
input, application state, and rendering.

## Modules

```text
main.rs
  ├─ cli.rs             command-line arguments
  ├─ app.rs             state, keyboard events, and UI composition
  └─ ui.rs              stateless Ratatui views and panel decoration

lib.rs
  ├─ core.rs            sorted directory-entry loading
  └─ error.rs           shared error types
```

`main.rs` parses the optional directory argument, creates `App`, initializes the
terminal, and starts the event loop. Terminal setup and restoration are delegated
to Ratatui.

`app.rs` is the coordinator and the single owner of persistent UI state. `App`
holds the selected directory, entries, input values, input cursors, and one
`Focus` value. Keyboard events mutate that state, then each draw constructs
short-lived views borrowing immutable data from `App`.

`ui.rs` is internal to the binary. `TextInput` and `FileList` are stateless views
that do not own titles, focus, or application data. `Panel` wraps any child view
and receives its titles and focus styling from `App`, allowing block presentation
to change in response to app-level state without coupling leaf widgets to it.
Horizontal input scrolling is derived from the cursor and the panel's inner area.

## Main boundaries for a rewrite

- `App` intentionally combines state and event coordination while views remain
  immutable and stateless.
- Block metadata belongs to UI composition in `App`, not to leaf views.
- `TextInput` handles Unicode characters but does not account for grapheme or
  terminal display width.
