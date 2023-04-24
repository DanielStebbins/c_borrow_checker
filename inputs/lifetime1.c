int *foo() {
    int x = 5;          // x created at scope level 1.
    return &x;          // ERROR: returning a reference to a local variable.
}