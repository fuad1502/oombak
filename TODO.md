TODO:
-[x] Experiment on tracing signals with Verilator.
-[ ] Create a proof of concept where I can generate a shared library from a
single Verilog file and a single configuration file (specifying exported
signals) that can be loaded and called from another program to:
    - Query exported signals.
    - Set / get signal value.
    - Move the simulation forward.
    - Reset the simulation.

NOTES:
- There are a multiple ways I can do to access signals:
    - Add `/* verilator public */` (or `/* verilator public_flat */`) on
    signals we would like to observe.
    - Use `--public-flat-rw` or `--public-depth` option when verilating the
    model. In practice, it is as if all signals (or signals from all modules up
    to a certain depth) are all marked with `/* verilator public */`. 
    - Generate a DPI function for accessing each signal "marked" by user. This
    is the preferred method since it's the recommended way.
