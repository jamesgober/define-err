<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br><b>define-err</b><br>
    <sub><sup>API REFERENCE</sup></sub>
</h1>
<div align="center">
    <sup>
        <a href="../README.md" title="Project Home"><b>HOME</b></a>
        <span>&nbsp;│&nbsp;</span>
        <span>API</span>
        <span>&nbsp;│&nbsp;</span>
        <a href="../dev/PLAN.md" title="Plan"><b>PLAN</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="../dev/ROADMAP.md" title="Roadmap"><b>ROADMAP</b></a>
    </sup>
</div>
<br>

The public API of define-err `0.1.0`. This release is the scaffold: it exposes one item, `VERSION`, and declares the feature set. The complete API of 1.0 (the `define_errors!` macro, codes, severity, categories, kinds, `ErrorInfo`, the catalog, the identity probe, `Secret<T>`, the reporting facade, and `ErrorRecord`) is written here at the `0.2.0` design lock, every item with the version it arrives in. The plan behind it, with item ids, is [`../dev/PLAN.md`](../dev/PLAN.md) section 8.

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>

## Table of contents

- [Setup](#setup)
  - [Features](#features)
  - [`no_std`](#no_std)
- [Crate root](#crate-root)
  - [`VERSION`](#version)

## Setup

```toml
[dependencies]
define-err = "0.1.0"
```

### Features

| Feature | Default | Implies | Brings (from the version shown) |
|---|---|---|---|
| `std` | yes | `alloc` | `std::io::Error` mappings (`0.4.0`); reentrancy guard and panic isolation in the facade (`0.5.0`) |
| `alloc` | through `std` | | `ErrorRecord`, boxed `dyn Error` sources, `set_boxed_reporter` (`0.5.0`) |
| `testing` | no | `std` | `testing::capture` and `CaptureReporter` (`0.5.0`) |

In `0.1.0` the features link `alloc` and `std` and carry no public items yet.

### `no_std`

The crate is `#![no_std]`. With `default-features = false` it uses `core` only; add `features = ["alloc"]` for the owned record and boxed sources. It builds for `thumbv7em-none-eabihf`, `thumbv6m-none-eabi`, `riscv32imac-unknown-none-elf`, and `wasm32-unknown-unknown`, checked in CI.

```toml
[dependencies]
define-err = { version = "0.1.0", default-features = false }
```

## Crate root

### `VERSION`

```rust
pub const VERSION: &str;
```

The crate version, as written in `Cargo.toml` (`"0.1.0"`). Item `ROOT-01`, since `0.1.0`.

```rust
assert_eq!(define_err::VERSION, "0.1.0");
```

Useful in diagnostics that name the error library a binary was built with:

```rust
let banner = format!("errors: define-err {}", define_err::VERSION);
assert!(banner.ends_with(define_err::VERSION));
```
