.section .text.init

.global _start

_start:
	mrs x6, MPIDR_EL1
	and x6, x6, #0x3
	cbz x6, primary_cpu

	mov x5, 0xd8
secondary_spin:
	wfe
	ldr x4, [x5, x6, lsl #3]
	cbz x4, secondary_spin
	mov x0, #0
	b boot_kernel

primary_cpu:
	bl kinit

boot_kernel:
	mov x1, #0
	mov x2, #0
	mov x3, #0
	br x4