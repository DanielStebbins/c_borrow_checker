// Moving values from behind references.

void main() {
    int x = 5;
    const int *m1 = &x;             // Const references and Copy types give no errors.
    int **mm = &m1;
    const int *m2 = *mm;
}