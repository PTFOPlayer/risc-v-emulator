    .global main


    .text
main:
    addi    sp, sp, -32
# setting up arguments for syscall
    addi    a0, x0, 1
    addi    a7, x0, 64
#print all numbers
    la      a1, numbers
    addi    a2, x0, 18
    ecall


## numbers to be printed
    addi    a0, x0, -1024
    addi    a0, a0, -1024
    addi    a0, a0, -1024
    sw      a0, 4(sp)
    lw      a0, 4(sp)
## base
    addi    a1, x0, 0
    beq     a1, x0, f_0

    addi    sp, sp, -1024
    jal     print_number           # a0: number to be displayed, a1: base of number
    addi    sp, sp, 1024
# new line

    addi    a0, x0, 1
    addi    a7, x0, 64
    la      a1, nl
    addi    a2, x0, 1
    ecall

    jal     end



f_0:
    addi    a0, x0, 1
    addi    a7, x0, 64
    la      a1, error
    addi    a2, x0, 17
    ecall
    jal end

print_number: # a0: number to be displayed, a1: base of number
    add     t0, x0, a0
    add     t1, x0, a1
    sw      a7, 4(sp)
    add     t4, x0, sp
    add     t5, x0, x0
l_0:
    rem     t2, t0, t1
    div     t0, t0, t1

# putting value to stack
    sb      t2, 0(t4)
    addi    t4, t4, -4
    addi    t5, t5, 1

    bne     t0, x0, l_0            # if t0 != 0


    addi    a0, x0, 1
    addi    a7, x0, 64


    addi    t6, x0, 1
l_1:
# getting value from stack
    addi    t4, t4, 4
    addi    t5, t5, -1
    lb      t2, 0(t4)

    bge     t2, x0, non_0          # if t2 > x0 then non_0
    beq     t6, x0, no_minus
    addi    t6, x0, 0
    la      a1, minus
    addi    a2, x0, 1
    ecall
no_minus:
    sub     t2, x0, t2
non_0:

# syscall to print single number
    la      a1, numbers
    addi    a2, x0, 1
    add     a1, a1, t2
    ecall

    bne     t5, x0, l_1            # if t0 != 0
    lw      a7, 4(sp)
    ret

# end of program
end:
    .data
minus:
    .string "-"
numbers:
    .string "0123456789ABCDEF\n\n"
nl:
    .string "\n"
error:
.string "error: base is 0\n"


# Printing
#addi a0, x0, 1
#la a1, dolor
#addi a2, x0, 15
#addi a7, x0, 64
#ecall
# test of stack
#addi sp, sp, -32
#addi a3, x0, -1
#sh a3, 0(sp)
#lh a2, 0(sp)