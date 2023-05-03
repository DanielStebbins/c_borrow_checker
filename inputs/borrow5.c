// Function call mutable reference created while borrowed.

void foo(int *a);
void bar(int b);

void main() {
    int x = 5;
    int *m = &x;
    foo(&x);                // invalidates m.
    bar(*m);                // ERROR: Using m, invalid reference to x.

    int y = 10;
    const int *c = &y;
    foo(&y);                // invalidates c.
    bar(*c);                // ERROR: Using c, invalid reference to y.   
}