// Integer variables only.

int a = 5;

void foo(int b);

void main() {
    int z;
    int x = a;
    int y = x;          // Copy types, no error.
    z = x;              
    x = 10;             
    z = x;
    foo(x);          
}