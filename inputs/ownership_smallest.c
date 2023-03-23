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

    newOwner.value = 3;

    // Prints 5.
    printf("%d\n", newOwner.value);

    // Checker should throw an error, using a variable that no longer owns this value.
    printf("%d\n", oldOwner.value);
}