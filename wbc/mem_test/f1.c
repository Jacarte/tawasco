#include <stdio.h>
char MEM[1000000000] = {};


int main() {

    for(int i = 0; i< 1000000000; i++){
        MEM[i] = 1;
    }
    printf("%p\n", (char*) &MEM);
    printf("%p\n", (char*)(&MEM + 1000000000));
    return 0;
}