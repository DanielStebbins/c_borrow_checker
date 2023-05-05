void foo(int a);

void main() {
    int initial = 5;
    const int *ref = &initial;      // ref created at scope level 1.
    while(1 > 0) {
        int x = 3;                  // x created at scope level 2.
        ref = &x;                   // address of x assigned to ref.
    }               
    foo(*ref);                      // ERROR: invalid reference ref to x.
}