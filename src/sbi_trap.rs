/** Representation of register state at time of trap/interrupt */
pub struct sbi_trap_regs {
	/** zero register state */
	pub zero: usize,
	/** ra register state */
	pub ra: usize,
	/** sp register state */
	pub sp: usize,
	/** gp register state */
	pub gp: usize,
	/** tp register state */
	pub tp: usize,
	/** t0 register state */
	pub t0: usize,
	/** t1 register state */
	pub t1: usize,
	/** t2 register state */
	pub t2: usize,
	/** s0 register state */
	pub s0: usize,
	/** s1 register state */
	pub s1: usize,
	/** a0 register state */
	pub a0: usize,
	/** a1 register state */
	pub a1: usize,
	/** a2 register state */
	pub a2: usize,
	/** a3 register state */
	pub a3: usize,
	/** a4 register state */
	pub a4: usize,
	/** a5 register state */
	pub a5: usize,
	/** a6 register state */
	pub a6: usize,
	/** a7 register state */
	pub a7: usize,
	/** s2 register state */
	pub s2: usize,
	/** s3 register state */
	pub s3: usize,
	/** s4 register state */
	pub s4: usize,
	/** s5 register state */
	pub s5: usize,
	/** s6 register state */
	pub s6: usize,
	/** s7 register state */
	pub s7: usize,
	/** s8 register state */
	pub s8: usize,
	/** s9 register state */
	pub s9: usize,
	/** s10 register state */
	pub s10: usize,
	/** s11 register state */
	pub s11: usize,
	/** t3 register state */
	pub t3: usize,
	/** t4 register state */
	pub t4: usize,
	/** t5 register state */
	pub t5: usize,
	/** t6 register state */
	pub t6: usize,
	/** mepc register state */
	pub mepc: usize,
	/** mstatus register state */
	pub mstatus: usize,
	/** mstatusH register state (only for 32-bit) */
	pub mstatusH: usize
}