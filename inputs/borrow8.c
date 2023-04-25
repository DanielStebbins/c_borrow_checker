void main() {
    int x = 5;
    const int *c1 = &x;
    const int *c2 = c1;             // no error.
    printf("%d\n", *c1);
    printf("%d\n", *c2);            
}