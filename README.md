# Bullarchy GUI

A graphical interface for the [Bullarchy](https://github.com/My-sidequests/Bullarchy) toolchain — the project manager and transpiler CLI for the [Bullang](https://github.com/My-sidequests/Bullang) language.

Bullarchy GUI exposes all six commands (`init`, `convert`, `fmt`, `check`, `editor-setup`, `update`) through a local web interface with a cosmic dark theme.

---

## How it works

Bullarchy GUI runs a small [axum](https://github.com/tokio-rs/axum) HTTP server on `localhost:7474` and opens the interface in your default browser. The frontend communicates with the server over `POST /api/<command>`. No internet connection is required beyond the initial font load.

---

## Installation

```bash
cargo install --git https://github.com/My-sidequests/Bullarchy-gui.git
```

Then launch it from any directory:

```bash
bullarchy-gui
```

The browser will open automatically. The server stays running until you close the terminal or press `Ctrl+C`.

---

## Commands

| Command        | Description |
|---------------|-------------|
| `init`         | Scaffold a new Bullang project (depth-based or blueprint) |
| `convert`      | Transpile a `.bu` project or single file to rs / py / c / cpp / go |
| `fmt`          | Reformat all `.bu` files to canonical style |
| `check`        | Validate, type-check, and verify formatting |
| `editor-setup` | Write LSP configs for Neovim, Vim, Helix, Emacs |
| `update`       | Reinstall Bullarchy GUI from the latest commit |

---

## Architecture

```
bullarchy-gui/
├── src/
│   ├── main.rs          # axum server, embedded frontend
│   ├── routes.rs        # HTTP handlers — one per command
│   ├── cmd/             # command logic (mirrored from bullarchy)
│   ├── build.rs         # transpiler pass
│   ├── codegen/         # 5 language backends
│   ├── init/            # project scaffolding + blueprint parser
│   ├── validator/       # structural + parse validation
│   └── ...              # shared modules
└── frontend/
    ├── index.html       # app shell
    ├── style.css        # cosmic dark theme
    └── app.js           # panel logic, star field, API calls
```

The frontend is embedded into the binary at compile time via `include_str!` — no separate file serving needed after installation.

---

## Relationship to Bullarchy (terminal)

Bullarchy GUI is a **separate repository** that mirrors the terminal version's command logic exactly. Both tools share the `bullang` library crate. The GUI wraps command output that would normally go to stdout/stderr and returns it as JSON to the browser.

For the terminal version, see [Bullarchy](https://github.com/My-sidequests/Bullarchy).
