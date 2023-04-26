// Borrowing any part of a struct counts as borrowing the entire thing.

typedef struct Owner {
    int value;
} Owner;

void main {
    Owner x;
    int *mx1 = &x.value;
    Owner *mx2 = &x;             // invalidates mx1.
    printf("%d\n", *mx1);        // ERROR: using invalid reference mx1.

    Owner y;
    Owner *my1 = &y;             
    int *my2 = &y.value;         // invalidates my1.
    printf("%d\n", *my1);        // ERROR: using invalid reference my1.
}