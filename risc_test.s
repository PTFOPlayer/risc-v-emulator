.global _start

.data
    hello: .ascii "hello\n"

.text
_start:
    lui a1, 10
    addi a2, x0, 10
