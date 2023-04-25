// Creating a const reference invalidates any previous mut references.

void main() {
    int x = 5;
    int *m1 = &x;
    int *m2 = &x;           // invalidates m1.
    const int *c = &x;      // invalidates m2.
    printf("%d\n", *m2);    // ERROR: Using m2, invalid reference to x.
    m1 = &x;                // validates m1, invalidates c1.
    printf("%d\n", *m1);    
}