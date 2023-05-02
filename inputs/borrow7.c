void main() {
    int x = 5;
    int *m1 = &x;
    int *m2 = m1;             
    foo(m1);            // ERROR: using invalid mutable reference m1.
}