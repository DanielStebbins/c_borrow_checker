void main() {
    int x = 5;
    const int *y = &x;          // adds a read-only reference to x.
    int z = x;                  // ERROR: killing borrowed variable x.
}