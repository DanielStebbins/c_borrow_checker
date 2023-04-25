// Function call mutable reference created while borrowed.

void main() {
    int x = 5;
    int *m = &x;
    foo(&x);                // invalidates m.
    printf("%d\n", *m);     // ERROR: Using m, invalid reference to x.

    int y = 10;
    const int *c = &y;
    foo(&y);                // invalidates c.
    printf("%d\n", *c);     // ERROR: Using c, invalid reference to y.   
}