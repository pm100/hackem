# Copilot Instructions

## What This Is

`hackem` is a Hack CPU emulator and interactive debugger built with [egui](https://github.com/emilk/egui)/eframe. It runs natively and compiles to WebAssembly via Trunk. The Hack CPU is a 16-bit architecture with 32KB ROM and 32KB RAM, emulated cycle-accurately.

## Build & Run

```bash
# Native
cargo run --release

# Web (dev server at http://127.0.0.1:8080)
trunk serve

# Web (production)
trunk build --release
```

Rust toolchain is pinned to **1.76.0** (see `rust-toolchain`). The wasm32 target must be installed: `rustup target add wasm32-unknown-unknown`.

## CI Checks

Run all CI checks locally with `./check.sh` (Linux/macOS) or equivalent commands:

```bash
cargo check --workspace --all-targets
cargo check --workspace --all-features --lib --target wasm32-unknown-unknown
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::all
cargo test --lib                    # run all library tests
cargo test --workspace --all-targets --all-features
```

To run a single test:
```bash
cargo test test_load_file           # run by test name
cargo test --lib emulator           # run tests in a specific module path
```

Spell-checking uses `typos` (`cargo install typos-cli`, then `typos`).

## Architecture

```
src/
  main.rs          — eframe entry point (native + wasm32 cfg-gated)
  utils.rs         — say!/verbose! macros, SAY_CB output routing, SharedPtr
  ui/
    app.rs         — HackEgui (eframe::App): owns HackSystem + all windows + Shell, drives the main update loop
    widgets/
      console.rs   — Terminal-style console window with history, reverse-i-search, prompt locking
      cpu.rs       — CPU register display
      screen.rs    — Hack screen (mapped RAM 0x4000–0x5FFF), keyboard input via CURRENT_KEY
      files.rs     — File browser window
    key_lookup.rs  — egui key → Hack keyboard code mapping
    wasm.rs        — WASM-specific menu/update logic
  emulator/
    engine.rs      — HackEngine: ALU, fetch/decode/execute loop, breakpoints, StopReason
    code_loader.rs — Loads .hx binary files and raw binary (one binary instruction per line)
    hacksys.rs     — (stub/commented out, superseded by debugger/debug_em.rs)
  debugger/
    debug_em.rs    — HackSystem: wraps HackEngine + Pdb, address resolution (hex/decimal/symbol)
    shell.rs       — Shell: dispatches console commands to clap, routes output via SAY_CB
    syntax.rs      — All clap subcommand definitions (load_code, break, list_symbols, expr, etc.)
    expr.rs        — Expression evaluator (evalexpr crate)
    pdbio.rs       — PDB (program database) I/O helpers
```

**External dependency**: `common` crate at `c:/work/hack/common` — provides `common::pdb::database::Pdb` for debug symbol/type info loaded from JSON. This is a sibling workspace crate, not on crates.io.

## Key Conventions

### Output routing (`say!` / `verbose!`)
Use the `say!` and `verbose!` macros (defined in `utils.rs`) for emulator/debugger output — never `println!` directly in those modules. In tests they fall back to `println!`. In the running app, output is routed through the `SAY_CB` function pointer (set by `Shell::new()`).

### Dual-target (native / wasm32)
Code that only works natively is gated with `#[cfg(not(target_arch = "wasm32"))]`. The WASM path uses `wasm_bindgen_futures::spawn_local`. Both paths must compile cleanly — the CI checks both.

### `.hx` binary file format
The Hack binary format loaded by `load_code` (or File → Load Binary):
```
hackem v1.0 <halt_addr_hex>
ROM@<hex_offset>
<hex_value>
...
RAM@<hex_offset>
<hex_value>
...
```
Raw files (no header) are loaded as binary instructions into ROM (one instruction per line).

### Address expressions in the shell
Addresses accept: `$1a2b` or `0x1a2b` (hex), plain decimal, or a symbol name looked up from the loaded PDB. Prefix `=` to evaluate an `evalexpr` expression (e.g. `expr =2+2`).

### Shell commands (clap multicall)
All debugger commands are defined in `debugger/syntax.rs` and dispatched in `debugger/shell.rs`. Add new commands in both places. Commands return `Result<String>` — errors are formatted and shown in the console window, not propagated to the UI.

### Emulator timing
`HackEngine::execute_instructions(Duration)` runs instructions for up to the given wall-clock time and returns a `StopReason` (`RefreshUI`, `SysHalt`, `HardLoop`, `BreakPoint`, `WatchPoint`). The UI calls this with 50ms per frame when running.

### Clippy
`#![warn(clippy::all, rust_2018_idioms)]` is set in `main.rs`. CI enforces `-D warnings`.
