void main() {
    int x = 5;
    const int *ref1 = &x;
    const int *ref2 = ref1;
    const int *ref3 = ref1;             // ERROR: use of dead variable ref1 (Different from Rust, where references implement the Copy trait).
}