// Function signatures recognized as taking mutable or const references.

void foo(const int *c);
void bar(int *m);

void main() {
    int x = 5;
    const int *cx = &x;
    foo(&x);                    // makes a second constant reference.
    foo(cx);                    // no error, c is still valid.

    int y = 5;
    const int *cy = &y;
    bar(&x);                    // makes a mutable reference, invalidating cy
    foo(cy);                    // ERROR: using invalid pointer cy.
}