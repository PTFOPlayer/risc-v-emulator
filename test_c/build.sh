riscv64-linux-gnu-gcc -Wa,-march=rv32i main.c -c -o main.o
riscv64-linux-gnu-ld main.o -o a.out 