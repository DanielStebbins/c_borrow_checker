// Transfer ownership while borrowed.

typedef struct Owner {
    int value;
} Owner;

void foo(Owner a);

void main() {
    Owner x;
    Owner *m = &x;
    Owner x2 = x;             // invalidates m.
    foo(*m);                  // ERROR: Using m, invalid reference to x.

    Owner y;
    const Owner *c = &y;
    Owner y2 = y;             // invalidates c.
    foo(*c);                  // ERROR: Using c, invalid reference to y.   
}