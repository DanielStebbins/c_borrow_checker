void main() {
    int x = 5;
    const int *y = &x;          // adds a read-only reference to x.
    int z = x;                  // ERROR: transfering ownership of borrowed variable x.
    foo(x);                     // ERROR: transfering ownership of borrowed variable x.
}