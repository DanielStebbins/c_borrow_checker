// Capturing function call.

struct Owner {
    int value;
};

void foo(struct Owner x, struct Owner *y);

void main() {
    struct Owner x;
    struct Owner y;
    foo(x, &y);                         // kills x, but not &y.
    struct Owner z = y;                 // no error.
    z = x;                              // ERROR: use of dead variable x.
}