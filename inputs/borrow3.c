// Overwrite owner while borrowed.

typedef struct Owner {
    int value;
} Owner;

void foo(Owner a);

void main(Owner a, Owner b) {
    Owner x;
    Owner *m = &x;
    x = a;                  // invalidates m.
    foo(*m);                // ERROR: Using m, invalid reference to x.

    Owner y;
    const Owner *c = &y;
    y = b;                  // invalidates c.
    foo(*c);                // ERROR: Using c, invalid reference to y.   
}