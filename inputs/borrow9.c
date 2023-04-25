void main() {
    int x = 5;
    int *m = &x;
    const int *c = m;      // invalidates m (DIFFERENT THAN RUST).
    printf("%d\n", *c);
    printf("%d\n", *m);     // ERROR: invalid reference m to x.
}