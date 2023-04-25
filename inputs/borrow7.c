void main() {
    int x = 5;
    int *m1 = &x;
    int *m2 = m1;             // ERROR: propagating mutable reference m1 to mutable reference m2.
}