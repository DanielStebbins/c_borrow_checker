void main() {
    int x = 5;
    const int *ref = &x;        // creates a read-only reference of x.
    int *mutable_ref = &x;      // ERROR: creates a mutable reference to x while x has living read-only references.
    foo(&x);                    // ERROR: creates a (assumed) mutable reference to x while x has living read-only references.
}