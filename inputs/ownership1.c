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
    return 0;
}