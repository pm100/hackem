# hackem

A cycle-accurate emulator and interactive debugger for the [Hack CPU](https://www.nand2tetris.org/) — the 16-bit architecture from the *Nand to Tetris* course. Built with [egui](https://github.com/emilk/egui)/eframe; runs natively and in the browser via WebAssembly.

---

## Features

- **Full Hack CPU emulation** — 16-bit ALU, 32 KB ROM, 32 KB RAM, accurate A/D/PC registers
- **512×256 screen** — real-time pixel display mapped from RAM (`0x4000–0x5FFF`), incremental updates
- **Keyboard input** — key presses forwarded to the Hack keyboard register (`0x6000`)
- **Output port** — writes to `RAM[0x7FFF]` are captured as ASCII and shown in the console
- **Interactive debugger console** with command history, reverse-search (Ctrl-R), and tab completion
- **Breakpoints** and **watchpoints** (read/write)
- **Disassembler** with breakpoint annotations and PC indicator
- **Symbol support** — load a PDB (JSON debug info) and use symbol names as addresses
- **Expression evaluator** — arithmetic expressions anywhere an address is expected
- **Dockable GUI** — Console, Code, CPU registers, two Data views, and Screen tabs
- **Break button** — interrupt a running program from the toolbar

---

## Building & Running

### Prerequisites

```sh
rustup target add wasm32-unknown-unknown   # for web builds
cargo install --locked trunk               # for web builds
```

> The Rust toolchain is pinned to **1.76.0** via `rust-toolchain`.

### Native

```sh
cargo run --release
```

On Linux, install the required system libraries first:

```sh
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
                     libxkbcommon-dev libssl-dev
```

### Web (dev server)

```sh
trunk serve
# open http://127.0.0.1:8080/index.html#dev
```

### Web (production build)

```sh
trunk build --release
# output in dist/  — upload to GitHub Pages or any static host
```

---

## File Formats

### `.hx` — hackem binary (native format)

Produced by the Hack assembler/compiler toolchain. Contains ROM and optional RAM preload sections.

```
hackem v1.0 <halt_addr_hex>
ROM@<hex_offset>
<hex_word>
...
RAM@<hex_offset>
<hex_word>
...
```

Example:
```
hackem v1.0 0x000e
ROM@0000
0002
8c10
0011
...
RAM@0000
0000
```

### `.hack` — raw binary

One 16-bit instruction per line in binary (`0`/`1` characters), no header. Loaded directly into ROM starting at address 0.

```
0000000000000010
1110110000010000
...
```

Load either format with `File → Load Binary` or the `load` command.

---

## Debugger Commands

Type `about` in the console for the full command list. All commands support aliases.

### Execution

| Command | Alias | Description |
|---------|-------|-------------|
| `go` | `g` | Run continuously |
| `stop` | `halt`, `pause` | Break into a running program |
| `next_instruction` | `ni`, `si` | Step one instruction |

You can also click **⏹ Break** in the toolbar to stop execution.

### Breakpoints

| Command | Alias | Description |
|---------|-------|-------------|
| `break <addr>` | `b` | Set breakpoint |
| `list_breakpoints` | `lbp` | List all breakpoints |
| `delete_breakpoint [addr]` | `dbp` | Delete one or all breakpoints |

### Watchpoints

| Command | Alias | Description |
|---------|-------|-------------|
| `watch <addr> [-r] [-w]` | `w` | Set watchpoint (default: rw) |
| `list_watchpoints` | `lwp` | List all watchpoints |
| `delete_watchpoint [addr]` | `dwp` | Delete one or all watchpoints |

### Memory & Registers

| Command | Alias | Description |
|---------|-------|-------------|
| `reg` | | Display PC, A, D registers |
| `dis [addr] [-n N]` | | Disassemble N instructions (default 16) from addr or PC |
| `mem <addr> [-n N]` | `m` | Hex dump N words from addr |
| `print <addr> [-i\|-s]` | `p` | Print value as integer or string |
| `write_memory <addr> <val>` | `wm` | Write a value to RAM |

### Symbols & Expressions

| Command | Alias | Description |
|---------|-------|-------------|
| `load_code <file>` | `load` | Load `.hx` or `.hack` binary |
| `load_pdb <file>` | `pdb` | Load debug symbol database (JSON) |
| `list_symbols [filter]` | `lsy` | List PDB symbols, optional substring filter |
| `expr <expression>` | | Evaluate an expression |
| `cd <dir>` | | Change working directory |

### Address Syntax

Addresses are accepted in several forms wherever `<addr>` appears:

| Format | Example | Meaning |
|--------|---------|---------|
| Hex with `$` | `$1a2b` | 0x1A2B |
| Hex with `0x` | `0x1a2b` | 0x1A2B |
| Decimal | `8192` | 8192 |
| Symbol name | `main` | Looked up from loaded PDB |
| `=<expression>` | `=base+4` | Evaluated by expression engine |

---

## Memory Map

| Range | Description |
|-------|-------------|
| `0x0000–0x7FFF` | ROM (32 KB, read-only at runtime) |
| `0x0000–0x3FFF` | RAM (general purpose) |
| `0x4000–0x5FFF` | Screen (512×256 pixels, 1 bit/pixel) |
| `0x6000` | Keyboard (current key code, read-only) |
| `0x7FFF` | Output port (write ASCII bytes; shown in console) |

---

## CI

```sh
cargo check --workspace --all-targets
cargo check --workspace --all-features --lib --target wasm32-unknown-unknown
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
typos   # spell check (cargo install typos-cli)
```

---

## Project Structure

```
src/
  main.rs              Entry point (native + wasm32 cfg-gated)
  utils.rs             say!/verbose! macros, SAY_CB output routing
  ui/
    app.rs             Main app: owns HackSystem, all windows, Shell
    widgets/
      console.rs       Terminal console (history, reverse-search)
      cpu.rs           CPU register display
      screen.rs        Hack screen renderer
      code.rs          Disassembly / code view
      data.rs          Memory hex viewer
    key_lookup.rs      egui key → Hack keyboard code
  emulator/
    engine.rs          HackEngine: ALU, fetch/decode/execute, breakpoints
    code_loader.rs     .hx and raw binary loader
  debugger/
    debug_em.rs        HackSystem: wraps engine + PDB, address resolution
    shell.rs           Command dispatcher
    syntax.rs          clap command definitions
    expr.rs            Expression evaluator
    pdbio.rs           PDB I/O helpers
common/                Sibling crate — Pdb debug symbol/type database
```

---

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

