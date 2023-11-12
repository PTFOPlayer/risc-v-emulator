.global _start


.text
_start:
    addi a1, x0, 10
    l0:
    addi a0, a0, 10
    beq a1, a0, l0

