# Fake desmos

yeah

## Install

Download the binary from the releases page. Create an `input.txt` file (see this repository's for an example). Open a terminal and run the binary in the directory containing `input.txt`.

Alternatively, install the Cargo package manager. Clone the repo and run `cargo run` in the top level (`expr/`) to compile and run (this may take a few minutes).

## How to use

Drag to move and scroll to zoom. If you compiled the project, you can adjust scroll sensitivity on the last line of `src/calculator.rs` and recompile.

Edit `input.txt` and save to reload.

Press shift to toggle debug view of the functions.

## Language docs

You can define bindings (variables and functions) with ascii alphabetic letters. For example:

```
PI := 3.14
f(x) := x + sin(x)
```

Bindings must be defined before they are used.

Standalone expressions that contain an `x` will be plotted. Variable bindings and expressions that don't contain an `x` will be evaluated and printed.

Note every expression and binding has to be on a single line (for now).

## Bugs

-   saving `input.txt` will sometimes load an empty graph
    -   if this happens often, it may be easier to just run `cargo run` again
-   zooming too far outside a graph in can cause sudden dramatic slowdowns and maybe crashes
    -   this is caused by drawing lines far away from the viewport [for some reason](https://github.com/linebender/vello/issues/1045)
-   rectangular rendering artifacts can appear when drawing dense functions like `x * sin(1000 / x)`
-   parts of functions that are very vertical (slope of >1000) will not be drawn
    -   this is because they're treated as discontinuities
