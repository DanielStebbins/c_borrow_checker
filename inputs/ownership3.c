// if-else block. (Accomplished using copies of the set of dead variables, unioned together after the if-else block ends.)

typedef struct Owner {
    int value;
} Owner;

void foo(Owner a);

void main(Owner z) {
    Owner x;
    if (1 > 2) {                // no analysis to show that only 'else' would ever run.
        Owner y = x;            // kills x.
    }
    else {
        x = z;                  // revives x.
    }
    foo(x);                     // ERROR: despite being revived in the 'else', x might still be dead if the 'if' was taken.
}