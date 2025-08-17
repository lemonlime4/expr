# current design
- bindings cannot be shadowed (by other bindings or function parameters)


# todo
- add functions
    - figure out how to parse multi arg functions without backtracking
        - make := into an operator and do another step?
- fix function args shadowing and deleting later bindings

# technical
- TODO 1 : figure out how to temporarily bind arguments when evaluating functions
- make arglist use one vec to avoid boxing in Expr


## parsing
- actually parse multi arg functions

- allow newlines inside expressions
- decide if function call syntax should also possibly do multiplication


## far future
- improve the runtime
- better (order independent) name resolution

- graphing
- point and list types
    - arglist formatting shouldn't have ()

- diagnose multiple independent errors at the same time