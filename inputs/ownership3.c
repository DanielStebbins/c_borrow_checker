// if-else block. (Accomplished using copies of the set of dead variables, unioned together after the if-else block ends.)

void main() {
    int x = 3;
    if (1 > 2) {            // no analysis to show that only 'else' would ever run.
        int y = x;          // kills x.
    }
    else {
        x = 5;              // revives x.
    }
    foo(x);                 // ERROR: despite being revived in the 'else', x might still be dead if the 'if' was taken.
}