-   why did exp(x^2) use to cause lag? (it's solved now)

-   better background grid
    -   make it actually dependent on pixel count
    -   subdivide major grid cells
    -   add numbers
-   clamp line segments to viewport to avoid vello "performance cliff"

-   built in constants (pi, e, inf, nan?)

-   actually handle newlines in parsing
    -   currently each assignment/declaration/value has to be 1 line
-   figure out why `input.txt` sometimes reads empty file

-   find out less hacky way to parse unary operator
-   better adaptive sampling with less line segments
    -   desmos puts two segments on left and right of a discontinuous point, anything to do with that?
-   optimize x^2 (and add special behavior for fractions?)
-   improve the runtime

-   zooming in very fast causes lag
-   background flashes and disappears during lag

-   better (order independent) name resolution
-   sliders
-   point and list types
-   implicits, parametrics, points, polygons
-   two-finger touchscreen zooming

-   better parsing errors
    -   example: 0x says expected newline
-   make errors properly display
-   diagnose multiple independent errors at the same time
