// Struct values, with ownership of both structs and members being tested.

// #include <stdio.h>

typedef struct Owner {
    int value;
    const float tester;
    const int *constRef;
    int *mutRef;
} Owner;

struct Test {
    int x;
    Owner testOwner;
};

typedef int randomWord;

int x = 5;

void foo(Owner o);

void main(const Owner *p1, int *p2)
{
    randomWord random = 5;
    struct Test test;
    int z = 5;
    test.testOwner.mutRef = &z;         // Creates variables for previously unknown names test.testOwner and test.testOwner.mutRef.
    Owner testKill = test.testOwner;    // kills test.testOwner.
    Owner testError = test.testOwner;
    struct Test newTest;
    test = newTest;                     // makes live test and any owner-type members of test.

    Owner oldOwner;
    oldOwner.value = 5;
    Owner newOwner = oldOwner;      // kills oldOwner.
    oldOwner.value = 3;             // ERROR: oldOwner is dead.
    Owner x;
    oldOwner = x;                   // lives oldOwner, kills x.
    oldOwner.value = 3;             // oldOwner is now alive.
    int y = oldOwner.value;         // no effect, since oldOwner.value is a copy type.
    foo(oldOwner);                  // Kills oldOwner.
    oldOwner = newOwner;            // makes live oldOwner and any owner-type members of oldOwner.
    return 0;
}