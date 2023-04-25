void main() {
    int x = 5;
    const int *c = &x;
    int *m = c;             // ERROR: propagating const reference c to mutable reference m.
}