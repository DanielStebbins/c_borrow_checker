// Several const references coexist, but all are invalidated when a mutable reference is created.

void main() {
    int x = 5;
    const int *c1 = &x;         // adds a const reference to x.
    const int *c2;
    c2 = &x;                    // adds a second const reference to x.
    int *m = &x;                // invalidates c1 and c2.             
    printf("%d\n", *c1);        // ERROR: Using c1, invalid reference to x.
    printf("%d\n", *c2);        // ERROR: Using c2, invalid reference to x.
}