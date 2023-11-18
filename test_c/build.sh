/opt/riscv/bin/riscv64-unknown-elf-gcc main.c -c -ggdb -o main.o
#riscv64-linux-gnu-ld main.o -o a.out 
#/opt/riscv/bin/riscv64-unknown-elf-as main.s -o main.o -march=rv64i
/opt/riscv/bin/riscv64-unknown-elf-ld main.o -o a.out   
/opt/riscv/bin/riscv64-unknown-elf-objdump a.out -D