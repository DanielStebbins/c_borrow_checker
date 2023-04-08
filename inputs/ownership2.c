// Capturing function call.

int main() {
    int x = 0;
    int y = 1;
    foo(x, &y);         // kills x, but not &y.
    int z = y;          // no error.
    z = x;              // ERROR: use of dead variable x.
}