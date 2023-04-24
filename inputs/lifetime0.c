void main() {
    int *ref;               // ref created at scope level 1.
    {
        int x = 3;          // x created at scope level 2.
        ref = &x;           // address of x assigned to ref.
    }                       // ERROR: x goes out of scope while borrowed.
    // Possible error occurs here when ref is used?
}