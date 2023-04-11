void main() {
    int x = 5;
    const int *ref1 = &x;
    int *ref2 = ref1;               // ERROR: immutable reference to x moved to a mutable reference variable.
}