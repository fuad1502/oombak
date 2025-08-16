![oombak_banner.png](https://github.com/fuad1502/oombak/blob/master/doc/oombak_banner.png?raw=true)
#

[![CI](https://github.com/fuad1502/oombak/actions/workflows/CI.yml/badge.svg)](https://github.com/fuad1502/oombak/actions/workflows/CI.yml)

*Oombak (/ˈɔmbak/, "waves" in Indonesian)* is an interactive SystemVerilog simulator UI that runs on your terminal!

> [!NOTE]
> Currently, *Oombak* uses [Verilator](https://github.com/verilator/verilator) and [slang](https://github.com/MikePopoloski/slang) to simulate and parse your SystemVerilog source, respectively.

Here's a demo showing off some of *Oombak*'s features:

![demo.gif](https://github.com/fuad1502/oombak/blob/master/doc/demo.gif?raw=true)

> [!IMPORTANT]
> *Oombak* is still in its very early stage of development. Check out our [issue tracker](https://github.com/fuad1502/oombak/issues?q=is%3Aissue%20state%3Aopen%20label%3Atracker) for a list of features planned for future releases. And please feel free to open up an issue for bugs or feature requests! ❤️

## Project structure

This project is comprised of several Rust *crates*:

- **oombak_gen**. A library crate that generates, from your SystemVerilog project, a dynamically-linked library (`libdut`) which represents a simulation instance of your design. It exports functions for interacting with the simulation, e.g. get / set signal value. It works by generating a SystemVerilog wrapper around your design with *DPI* functions to provide access to internal signals, and a corresponding CMake based C++ Verilator project.

- **oombak_rs**: Provides two Rust structs: `Dut` and `Probe`. `Dut` is essentially a Rust binding to **oombak_gen**'s `libdut`, while `Probe` allows you to traverse your design hierarchy and specify active "probe points" (signals you would like to inspect).

- **oombak_sim** and **oombak_local_sim**: *Oombak*'s simulator "backend". Currently, it runs on the same process as the UI's. However, I am planning to support remote server feature in the future.

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

## User guide

### Basic operation

Once installed, you can launch *Oombak* by opening a terminal and execute:

```sh
oombak_tui
```

You can navigate through the interface with only a keyboard. Mouse support is planned.

At the bottom of the interface, there is a *command keys help bar* always available. It shows what *command keys* are avaiable in the currently active view:

![command_help_bar.gif](https://github.com/fuad1502/oombak/blob/master/doc/command_help_bar.gif?raw=true)

Depending on the number of available *command keys* and your window size, *command keys help bar* might not show all available *command keys*. To see all available *command keys*, you can toggle the *command keys help window* by pressing `<F2>`:

![command_help_window.gif](https://github.com/fuad1502/oombak/blob/master/doc/command_help_window.gif?raw=true)

You can scroll through the *command keys help window* by pressing `<F1>` or `<F3>`. Once toggled on, *command keys help window* **stays** there while you navigate through *Oombak*. It only captures `<F1>` through `<F3>` keys, and therefore won't interfere with other *command keys*. You can toggle it off by pressing `<F2>` again.

To quit *Oombak*, you can press `q`.

> [!TIP]
> *Oombak* utilizes overlaying windows to display widgets (e.g. file explorer). You can typically use either `q`, `<esc>`, or `<Ctrl>-D` to close them.

*Oombak* supports both arrow keys (`←`, `↓`, `↑`, `→`) and Vim motions (`h`, `j`, `k`, `l`) for simple navigation (moving left, down, up, and right).

> [!IMPORTANT]
> Key re-mapping through configuration file feature is planned for future releases. 

### Loading a design

There are two methods for loading a design, using the file explorer and using the terminal. 

To load a design using the file explorer, simply open up the file explorer by pressing `o`, and select your top level SystemVerilog file:

![file_explorer.gif](https://github.com/fuad1502/oombak/blob/master/doc/file_explorer.gif?raw=true)

You can use the terminal to send commands. One of those commands is the `load` command. It accepts a single parameter, a path to a SystemVerilog file. Open the terminal by pressing `t`, write down `load <path to top level SystemVerilog file>`, and press `<enter>`:

![terminal.gif](https://github.com/fuad1502/oombak/blob/master/doc/terminal.gif?raw=true)

Typically, you would open the terminal to inspect previous outputs, as shown above. For quickly executing commands, you can press `:`, write down your command, and press `<enter>`:

![quick_terminal.gif](https://github.com/fuad1502/oombak/blob/master/doc/quick_terminal.gif?raw=true)

> [!WARNING]
> If you encounter *top-level module not found* error, please ensure your top level SystemVerilog file name is the same as the top-level module name. 
> Only files with `.sv` extension within the same folder as your top level SystemVerilog file are compiled.

### Interacting with your simulation

There are several commands that you can invoke to interact with the simulation:

| Command      | parameters                                             | description                                    |
| :----------- | :----------------------------------------------------- | :--------------------------------------------- |
| run          | duration                                               | run the simulation for as long as the duration |
| set          | signal name, value                                     | sets the signal value                          |
| set-periodic | signal name, period, low state value, high state value | set period signal value                        |

> [!TIP] 
> All available commands can be listed by invoking the `help` command. 

If you prefer, you can also set signal values through the user interface. Scroll through available signals (by moving up or down) to focus on a signal, and press `<enter>`. This will open up a window for configuring the signal properties for that signal. Not only can you set the signal value, you can also configure how you would like the signal waveform to be displayed:

![signal_properties.gif](https://github.com/fuad1502/oombak/blob/master/doc/signal_properties.gif?raw=true)

### Probe editing

When you first load your design, only top level signals are displayed. If you would like to display internal signals, you can do so with the *probe editor*. Open the *probe editor* by pressing `s`, browse through the hierarchy and press `<enter>` on signals you would like to add (or remove, once added). Once you've made your selection, close the *probe editor* (`q`): 

![probe_editor.gif](https://github.com/fuad1502/oombak/blob/master/doc/probe_editor.gif?raw=true)

## UI framework design
