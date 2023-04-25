// Transfer ownership while borrowed.
#include <stdio.h>
void main() {
    int x = 5;
    int *m = &x;
    int x2 = x;             // invalidates m.
    printf("%d\n", *m);     // ERROR: Using m, invalid reference to x.

    int y = 10;
    const int *c = &y;
    int y2 = y;             // invalidates c.
    printf("%d\n", *c);     // ERROR: Using c, invalid reference to y.   
}