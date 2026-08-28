# Architecture

`nomi` is split into a small reusable rename library and a terminal UI binary.
The library decides what may be renamed and performs the filesystem changes; the
binary owns user input, application state, and rendering.

## Modules

```text
main.rs
  ├─ cli.rs             command-line arguments
  ├─ app.rs             state and keyboard-event handling
  └─ ui.rs              Ratatui rendering

lib.rs
  ├─ error.rs           shared error types
  └─ rename.rs          directory entries and public rename API
       ├─ preview.rs    builds and validates rename plans
       └─ execute.rs    executes plans with rollback
```

`main.rs` parses the optional directory argument, creates `App`, initializes the
terminal, and starts the event loop. Terminal setup and restoration are delegated
to Ratatui.

`app.rs` is the coordinator. `App` holds the selected directory, its entries, the
two text inputs, matching mode, focus, selection cursor, confirmation state, and
status message. Keyboard events update this state. It calls into the rename
library to create previews and execute confirmed changes.

`ui.rs` renders an immutable view of `App`. It calculates the current preview,
draws the inputs and visible file rows, and overlays the confirmation dialog when
needed. Scrolling is derived from the cursor and available terminal height rather
than stored separately.

## Rename flow

1. `read_entries` reads the immediate children of the chosen directory. Entries
   are sorted by name and selected by default.
2. As the UI redraws, `RenamePreview::build` applies either a regex or literal
   replacement to every selected entry.
3. The preview validates destination names, duplicate destinations, existing
   files, and directory boundaries. Invalid plans are displayed but cannot be
   confirmed.
4. Pressing Enter opens a confirmation dialog for a valid, non-empty preview.
5. Confirmation rebuilds and validates the preview, then executes it.
6. The directory is read again so the UI reflects the resulting filesystem.

The preview contains both display names, indexed to match `App::entries`, and the
filesystem operations that will be executed.

## Filesystem safety

Renames happen in two phases so swaps and cycles work:

```text
original names → unique temporary names → destination names
```

If staging fails, already-staged entries are restored. If committing fails,
committed and still-staged entries are moved back where possible. An ordinary
rename failure is shown in the UI; an incomplete rollback exits the app because
the filesystem may require manual inspection.

The app only operates on immediate children of one directory. Both source and
destination paths must remain inside that directory, and filenames must be valid
UTF-8 because matching is string-based.

## Main boundaries for a rewrite

- `App` currently combines state, event handling, and orchestration.
- The UI rebuilds the preview on every draw rather than caching derived state.
- `RenamePreview` exposes parallel display and operation collections that callers
  must keep consistent.
- `TextInput` handles Unicode characters but does not account for grapheme or
  terminal display width.

The most stable part to retain independently is the rename pipeline: read entries,
build and validate a plan, then execute it transactionally.
