foo(const int *c);

void main() {
    int x = 5;
    const int *c = &x;
    int *m = c;             // ERROR: cannot move const reference to mut reference.
    foo(c);
}