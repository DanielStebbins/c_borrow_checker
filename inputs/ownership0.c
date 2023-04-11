// Integer variables only.

void main() {
    int z;
    int x = 5;
    int y = x;          // kills x.
    z = x;              // ERROR: Use of dead variable x.
    x = 10;             // revives x.
    z = x;              // kills x, but no error.
}