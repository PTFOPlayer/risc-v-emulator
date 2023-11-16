#riscv64-linux-gnu-gcc -Wa,-march=rv32i main.c -c -o main.o
#riscv64-linux-gnu-ld main.o -o a.out 
/opt/riscv/bin/riscv64-unknown-elf-as main.s -o main.o -march=rv64i
/opt/riscv/bin/riscv64-unknown-elf-ld main.o -o a.out   
/opt/riscv/bin/riscv64-unknown-elf-objdump a.out -d  