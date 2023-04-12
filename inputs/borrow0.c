void main() {
    int x = 5;
    int *y;
    y = &x;                 // adds a mutable reference to x.
    foo(x);                 // ERROR: transfering ownership of borrowed variable x.
    int z = x;              // ERROR: transfering ownership of borrowed variable x.
}