void main() {
    int initial = 5;
    int *ref = &initial;        // ref created at scope level 1.
    if(1 > 0){
        int x = 3;              // x created at scope level 2.
        ref = &x;               // address of x assigned to ref.
    }                       
    printf("%d\n", *ref);       // ERROR: invalid reference ref to x.
}