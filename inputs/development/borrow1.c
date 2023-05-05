// Creating a const reference invalidates any previous mut references.

void foo(int a);

void main() {
    int x = 5;
    int *m1 = &x;
    int *m2 = &x;           // invalidates m1.
    const int *c = &x;      // invalidates m2.
    foo(*m2);               // ERROR: Using m2, invalid reference to x.
    m1 = &x;                // validates m1, invalidates c1.
    foo(*m1);    
}