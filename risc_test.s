.global _start

_start: 
    addi a0, x0, 1

    # Setup the parameters to exit the program
    addi    a0, x0, 0   # Use 0 return code
    addi    a7, x0, 93  # Service command code 93 terminates
    ecall               # Call linux to terminate the program
    