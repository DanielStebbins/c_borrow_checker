void main() {
    int x = 5;              // x created at scope level 1.
    {
        int y = 3;          // y created at scope level 2.
        {
            int z = 1;      // z created at scope level 3.
        }
    }
}