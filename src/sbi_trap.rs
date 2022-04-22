/** Representation of register state at time of trap/interrupt */
struct sbi_trap_regs {
	/** zero register state */
	usize zero,
	/** ra register state */
	usize ra,
	/** sp register state */
	usize sp,
	/** gp register state */
	usize gp,
	/** tp register state */
	usize tp,
	/** t0 register state */
	usize t0,
	/** t1 register state */
	usize t1,
	/** t2 register state */
	usize t2,
	/** s0 register state */
	usize s0,
	/** s1 register state */
	usize s1,
	/** a0 register state */
	usize a0,
	/** a1 register state */
	usize a1,
	/** a2 register state */
	usize a2,
	/** a3 register state */
	usize a3,
	/** a4 register state */
	usize a4,
	/** a5 register state */
	usize a5,
	/** a6 register state */
	usize a6,
	/** a7 register state */
	usize a7,
	/** s2 register state */
	usize s2,
	/** s3 register state */
	usize s3,
	/** s4 register state */
	usize s4,
	/** s5 register state */
	usize s5,
	/** s6 register state */
	usize s6,
	/** s7 register state */
	usize s7,
	/** s8 register state */
	usize s8,
	/** s9 register state */
	usize s9,
	/** s10 register state */
	usize s10,
	/** s11 register state */
	usize s11,
	/** t3 register state */
	usize t3,
	/** t4 register state */
	usize t4,
	/** t5 register state */
	usize t5,
	/** t6 register state */
	usize t6,
	/** mepc register state */
	usize mepc,
	/** mstatus register state */
	usize mstatus,
	/** mstatusH register state (only for 32-bit) */
	usize mstatusH
}