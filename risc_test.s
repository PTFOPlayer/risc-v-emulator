.global _start

.data
    hello: .ascii "hello\n"

.text
_start:
    lui a0, 1
    addi a0, a0, 1
    slti a0, a0, 1
    sltiu a0, a0, 1
    xori a0, a0, 1
    ori a0, a0, 1
    andi a0, a0, 1
    srli a0, a0, 1
    srai a0, a0, 1
    add a0, x0, x0
    sub a0, x0, x0
