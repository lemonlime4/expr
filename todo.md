-   exp(-x^2) causes lag
    -   why did exp(x^2) use to cause lag?
-   (it's solved now)

-   better background grid
    -   make it actually dependent on pixel count
    -   subdivide major grid cells
    -   add numbers
-   clamp line segments to viewport to avoid vello "performance cliff"
-   resample functions on separate thread

-   figure out why `input.txt` sometimes reads empty file

-   proper operator associativity
    -   is this really needed?
    -   all operators are currently right associative
-   better adaptive sampling with less line segments
    -   desmos puts two segments on left and right of a discontinuous point, anything to do with that?
-   optimize x^2 (and add special behavior for fractions?)
-   improve the runtime
-   better (order independent) name resolution

-   sliders
-   point and list types
-   implicits, parametrics, points, polygons

-   make errors properly display
-   diagnose multiple independent errors at the same time
