# Oombak 🌊

[![CI](https://github.com/fuad1502/oombak/actions/workflows/CI.yml/badge.svg)](https://github.com/fuad1502/oombak/actions/workflows/CI.yml)

*Oombak (/ˈɔmbak/, "waves" in Indonesian)* is an interactive SystemVerilog simulator UI that runs on your terminal!

> [!NOTE]
> Currently, *Oombak* uses [Verilator](https://github.com/verilator/verilator) and [slang](https://github.com/MikePopoloski/slang) to simulate and parse your SystemVerilog source, respectively.

Here's a demo showing off some of *Oombak*'s features:

## Project structure

This project is comprised of several Rust *crates*:

- **oombak_gen**. A library crate that generates, from your SystemVerilog project, a dynamically-linked library (`libdut`) which represents a simulation instance of your design. It exports functions for interacting with the simulation, e.g. get / set signal value. It works by generating a SystemVerilog wrapper around your design with *DPI* functions to provide access to internal signals, and a corresponding CMake based C++ Verilator project.

- **oombak_rs**: Provides two Rust structs: `Dut` and `Probe`. `Dut` is essentially a Rust binding to **oombak_gen**'s `libdut`, while `Probe` allows you to traverse your design hierarchy and specify active "probe points" (signals you would like to inspect).

- **oombak_sim** and **oombak_local_sim**: *Oombak*'s simulator "backend". Currently, it runs on the same process as the UI's. However, I am planning to support remote server feature!

- **oombak_tui**: *Oombak*. *Oombak* uses [ratatui](https://github.com/ratatui/ratatui) library for the terminal drawing primitives. For the UI framework, I decided to implement my own. Learn more about the design [here](). I also made many interesting widgets that I am planning to put into a separate library crate. 

## Installation

Currently, *Oombak* is only supported on Linux and MacOS.

If you already have [rustup](https://www.rust-lang.org/learn/get-started), simply execute:

```sh
rustup update
cargo install oombak
```
You can also install our pre-built binaries from the [Release page](https://github.com/fuad1502/oombak/releases).

### Dependencies

### Linux (Ubuntu)

```sh
apt install verilator cmake
```
> [!WARNING]
> If you encounter problems with Verilator from your distributor's package repository, try using the packages built by [veryl-lang]() [here](https://github.com/veryl-lang/verilator-package/releases). Or, try the "Git Quick Install" step in Verilator's documentation [here](https://veripool.org/guide/latest/install.html#git-quick-install).

### MacOS

```sh
brew install verilator cmake
```

## Building from source

```sh
git clone https://github.com/fuad1502/oombak.git
cd oombak
cargo build
cargo test
```

## Quick start guide
