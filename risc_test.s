.global _start


.text
_start:
    jal a1, l1
    l0:
    addi a0, a0, 1
    jal a1, l0
    l1:
    addi a2, x0, 10
    jal a1, l0

