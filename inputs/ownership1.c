// Struct values, with ownership of both structs and members being tested.

typedef struct Owner {
    int value;
} Owner;

int main()
{
    Owner oldOwner;
    oldOwner.value = 5;

    // kills oldOwner.
    Owner newOwner = oldOwner;

    // oldOwner dead.
    oldOwner.value = 3;
    Owner x;

    // lives oldOwner, kills x.
    oldOwner = x;
    oldOwner.value = 3;
    int y = oldOwner.value;
    printf("%d\n", oldOwner.value);
    return 0;
}