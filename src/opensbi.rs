pub fn csr_read(csr: str) {
	register unsigned long __v;
	__asm__ __volatile__("csrr %0, " __ASM_STR(csr)
				     : "=r"(__v)
				     :
				     : "memory");
		__v;
	})
    unsafe {
        asm!("la t0, 1f\n\t" 
        "csrrw t0, mtvec, t0\n\t" 
        "csrw pmpaddr"#n", %0\n\t" 
        "csrw pmpcfg"#g", %1\n\t" 
        "sfence.vma\n\t"
        ".align 2\n\t" 
        "1: csrw mtvec, t0 \n\t" 
        : : "r" (addr), "r" (pmpc) : "t0");
    }
}

#define csr_write(csr, val)                                        \
	({                                                         \
		unsigned long __v = (unsigned long)(val);          \
		__asm__ __volatile__("csrw " __ASM_STR(csr) ", %0" \
				     :                             \
				     : "rK"(__v)                   \
				     : "memory");                  \
	})