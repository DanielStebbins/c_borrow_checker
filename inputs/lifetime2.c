void foo(int a);

void main() {
    int initial = 5;
    const int *ref = &initial;
    while(0 < 2) {
        if(1 > 0) {
            continue;
        } else {
            int x = 10;
            ref = &x;
        }
    }
    foo(*ref);
}