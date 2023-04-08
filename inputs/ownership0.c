// Integer variables only.

int main() {
    int z;
    int x = 5;
    int y = x;
    // x is dead here, and any use besides re-assignment prints an error.
    z = x;
    x = 10;
    z = x;
}