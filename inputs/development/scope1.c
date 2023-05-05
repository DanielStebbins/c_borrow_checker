int a = 4;

void main() {
    int x = 5;                                  // x created at scope level 1.
    if(x > 0) {           
        int y = 3;                              // y created at scope level 2.
        while(x < y) {
            int z = 1;                          // z created at scope level 3.
            a = 2;                              // global variable a.
        }
        int b = 2;                              // b created at scope level 2.
    }
}