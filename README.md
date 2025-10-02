# Fake desmos

yeah

## How to use

Install the Cargo package manager. Clone the repo and run `cargo run` in the top level (`expr/`).

Drag to move and scroll to zoom. You can adjust scroll sensitivity on the last line of `src/calculator.rs`.

Edit `input.txt` and save to reload.

Press shift to toggle debug view of the functions.

## Language docs

You can define bindings (variables and functions) with ascii alphabetic letters. For example:

```
PI := 3.14
f(x) := x + sin(x)
```

Bindings must be defined before they are used.

Standalone expressions that contain an `x` will be plotted. Expressions that don't contain an `x` and variable bindings will be evaluated and printed.

Note that multiline expressions aren't supported yet.

## Bugs

-   saving `input.txt` will sometimes load an empty graph
    -   if this happens often, it may be easier to just run `cargo run` again
-   zooming too far outside a graph in can cause sudden dramatic slowdowns and maybe crashes
    -   this is caused by drawing lines far away from the viewport
    -   zooming very quickly can also cause this for some reason
-   rectangular rendering artifacts can appear when drawing dense functions like `sin(100/x)`
-   parts of functions that are very vertical (slope of >1000) will not be drawn
    -   this is because they're treated as discontinuities
