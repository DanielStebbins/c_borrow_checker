// Moving values from behind references.

void foo(int *m);

void main() {
    int x = 5;
    int *m1 = &x;             
    int **mm = &m1;
    int *m2 = *mm;              // Owner (Struct) types and mutable references cannot
    foo(*mm);                   // be copied from behind references and throw an error.
}