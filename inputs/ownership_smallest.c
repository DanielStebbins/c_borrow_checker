#include <stdio.h>

typedef struct
{
    int value;
}
Object;

int main()
{
    Object oldOwner;
    oldOwner.value = 5;
    Object newOwner = oldOwner;

    // Checker should throw an error.
    oldOwner.value = 3;

    printf("%d\n", newOwner.value);

    oldOwner = { .value = 1 };
    printf("%d\n", oldOwner.value);
}