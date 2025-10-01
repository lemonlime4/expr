# Fake desmos

yeah

## How to use

Install the Cargo package manager and run `cargo run` in the repo's top level (`expr/`).

Edit `input.txt` and save to reload.

Press shift to toggle debug view of the functions.

## Language docs

You can define bindings (variables and functions) with ascii alphabetic letters. For example:

```
PI := 3.14
f(x) := x + sin(x)
```

Standalone expressions that contain an `x` will be plotted. Expressions that don't contain an `x` will be evaluated and printed.

Note that multiline expressions aren't supported yet.

## Bugs

See `todo.md`

```

```
