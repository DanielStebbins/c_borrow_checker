// Struct values, with ownership of both structs and members being tested.

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

void main(const Owner *p1, int *p2)
{
    struct Test test;
    int z = 5;
    test.testOwner.mutRef = &z;

    Owner oldOwner;
    oldOwner.value = 5;
    Owner newOwner = oldOwner;      // kills oldOwner.
    oldOwner.value = 3;             // ERROR: oldOwner is dead.
    Owner x;
    oldOwner = x;                   // lives oldOwner, kills x.
    oldOwner.value = 3;             // oldOwner is now alive.
    int y = oldOwner.value;         // kills oldOwner.value, but not oldOwner.
    printf("%d\n", oldOwner.value); // ERROR: oldOwner.value is dead.
    oldOwner = newOwner;            // makes live oldOwner and oldOwner.value, as assigning a whole new struct brings a new member value.
    return 0;
}