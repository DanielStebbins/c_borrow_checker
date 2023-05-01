typedef int randomType;

struct TestStruct {
    int x;
};

typedef struct Test2 {
    int value;
} TestTypeDef;

int foo(int *a, const int *b, float c, struct TestStruct d, TestTypeDef e, randomType *f, randomType g);

void main() {
    printf("%d\n", foo(2));
}