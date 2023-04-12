void main() {
    int x = 5;
    int *ref0 = &x;                 // creates a read-only reference of x.
    int *ref1 = &x;                 // ERROR: creates a mutable reference to x while x has a living mutable reference.
    foo(x);                         // ERROR: transfering ownership of borrowed variable x.
}