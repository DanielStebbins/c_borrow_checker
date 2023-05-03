void foo(const int* c);

void main() {
    int x = 5;
    const int *c1 = &x;
    const int *c2 = c1;             // no error.
    foo(c1);
    foo(c2);     
}