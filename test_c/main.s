	.file	"main.c"
	.option nopic
	.attribute arch, "rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0"
	.attribute stack_align, 16
	.text
	.globl	main
main:
	addi	sp,sp,-32
	sd	s0,24(sp)
	addi	s0,sp,32
	li	a5,10
	sh	a5,-18(s0)
	li	a5,20
	sh	a5,-20(s0)
	lhu	a4,-18(s0)
	lhu	a5,-20(s0)
	addw	a5,a4,a5
	slli	a5,a5,48
	srli	a5,a5,48
	sh	a5,-22(s0)
	nop
	mv	a0,a5
	ld	s0,24(sp)
	addi	sp,sp,32
	jr	ra
	.size	main, .-main
	.ident	"GCC: (GNU) 13.2.0"
