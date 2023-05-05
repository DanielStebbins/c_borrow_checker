void foo(int a);

void main() {
    int x = 5;
    int *m = &x;
    const int *c = m;      // invalidates m (DIFFERENT THAN RUST, in Rust this is dependent on whether m is later used to modify x).
    foo(*c);
    foo(*m);               // ERROR: invalid reference m to x.
}