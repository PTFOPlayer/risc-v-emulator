.global _start


.text
_start:
    addi a0, x0, 1
    la a1, hello
    addi a2, x0, 12
    addi a7, x0, 64
    ecall

.data
    hello: .string "lorem ipsum\n"