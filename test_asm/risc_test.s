.global _start


.text
_start:
    addi a0, x0, 1
    la a1, lorem
    addi a2, x0, 12
    addi a7, x0, 64
    ecall


    addi a0, x0, 1
    la a1, dolor
    addi a2, x0, 15
    addi a7, x0, 64
    ecall
.data
    lorem: .string "lorem ipsum\n"
    dolor: .string "dolor sit amet\n"