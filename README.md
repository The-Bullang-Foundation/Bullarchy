# Bullarchy

Bullarchy is the interactive toolchain for [Bullang](https://github.com/The-Bullang-Foundation/Bullang) projects. It handles project scaffolding, formatting, validation, transpilation, and editor integration.

It depends on Bullang as a library crate and can be installed independently — Bullang does not need to be installed as a binary for Bullarchy to work.

---

## Prerequisite

Cargo v1.92.0 or later.

## Installation

```bash
cargo install --git https://github.com/The-Bullang-Foundation/Bullarchy.git
```

If you are reinstalling over an existing version, add `--force`:

```bash
cargo install --git https://github.com/The-Bullang-Foundation/Bullarchy.git --force bullarchy
```

After installation, `bullarchy` is available from anywhere.

---

## Usage

```bash
bullarchy
```

Launches the interactive prompt:

```
command ->
```

---

### `update`

Reinstall Bullarchy from the latest commit on the main branch.

```
command -> update
```

---

## Commands

### `init`

Scaffold a new Bullang project.

```
command -> init my_project
command -> init my_project --depth 4
command -> init my_project --lang c --lib stdio.h
command -> init my_project --blueprint blueprint.bu
command -> init my_project --blueprint blueprint.bu --lang go
```

Options:

- `--depth N` — hierarchy depth from 1 (skirmish only) to 6 (full war chain). Default: 2.
- `--lang ext` — target language (`rs`, `py`, `c`, `cpp`, `go`). Written to inventory as `#lang:`.
- `--lib header` — external library declaration. Can be repeated for multiple libraries.
- `--blueprint file` — initialize from a `blueprint.bu` file instead of a depth value. Depth is inferred from the blueprint.
- `--path dir` — where to create the project (default: current directory).

Depth reference:

```
depth 1 → skirmish
depth 2 → tactic → skirmish
depth 3 → strategy → tactic → skirmish
depth 4 → battle → strategy → tactic → skirmish
depth 5 → theater → battle → strategy → tactic → skirmish
depth 6 → war → theater → battle → strategy → tactic → skirmish
```

---

### `convert`

Transpile a Bullang project folder or a single `.bu` file.

```
command -> convert my_project
command -> convert my_project -e py
command -> convert path/to/file.bu
command -> convert path/to/file.bu -o out.rs
```

Options:

- `-n name` — output folder name (project mode).
- `-e ext` — target language (`rs`, `py`, `c`, `cpp`, `go`). Overrides `#lang` from inventory.
- `--out dir` — explicit output path (project mode).
- `-o file` — output file (single-file mode; omit to write to stdout).

---

### `fmt`

Format all `.bu` files in the project to canonical style. Rewrites files in place. Escape block contents are never modified.

```
command -> fmt
command -> fmt my_project
command -> fmt --dry-run
```

- With no argument, formats from the current directory.
- `--dry-run` shows which files would change without writing anything.

---

### `check`

Validate and type-check the project from the current directory. Also reports any files not in canonical format — run `fmt` to fix.

```
command -> check
```

Runs three passes in order:

1. Structural validation (rank hierarchy, inventory consistency, function declarations)
2. Type checking
3. Format drift check

Exits with a clear report on the first failing pass.

---

### `editor-setup`

Write LSP configuration files for detected editors.

```
command -> editor-setup
```

Supports: Neovim (nvim-lspconfig), Helix, Emacs (eglot).
For VS Code: install the extension through the VS Code extension page.

The LSP server is built into Bullarchy and started directly by editors via `bullarchy lsp`.

---

### `help`

Print the list of available commands.

```
command -> help
```

---

### `exit`

Quit Bullarchy.

```
command -> exit
```

---

## LSP server

Bullarchy includes the Bullang language server. Editors invoke it directly — it bypasses the interactive prompt:

```bash
bullarchy lsp
```

Capabilities: diagnostics, hover (function signatures), go-to-definition. Run `editor-setup` to have Bullarchy write the configuration for your editor automatically.

---

## Relationship to Bullang and Bullscript

**[Bullang](https://github.com/The-Bullang-Foundation/Bullang)** is the language definition — grammar, AST, parser, formatter, and stdlib catalogue. Bullarchy depends on it as a library crate. Run `bullang stdlib` to browse available builtins.

**[Bullscript](https://github.com/The-Bullang-Foundation/Bullscript)** is the interactive writing and testing tool. It does not transpile — that is Bullarchy's role. The two tools are designed to be used together: write and test with Bullscript, transpile and integrate with Bullarchy.
