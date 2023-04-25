// Overwrite owner while borrowed.

void main() {
    int x = 5;
    int *m = &x;
    x = 0;                  // invalidates m.
    printf("%d\n", *m);     // ERROR: Using m, invalid reference to x.

    int y = 10;
    const int *c = &y;
    y = 1;                  // invalidates c.
    printf("%d\n", *c);     // ERROR: Using c, invalid reference to y.   
}