// Moving values from behind references.

void main() {
    int x = 5;
    int *m1 = &x;             
    int **mm = &m1;
    int *m2 = *mm;              // Owner (Struct) types and mutable references throw an error.
}