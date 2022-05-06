use core::arch::asm;

pub const MSTATUS_MPP_SHIFT: usize = 11;

pub const SATP_MODE_OFF: usize = 0;
pub const SATP_MODE_SV32: usize = 1;
pub const SATP_MODE_SV39: usize = 8;
pub const SATP_MODE_SV48: usize = 9;
pub const SATP_MODE_SV57: usize = 10;
pub const SATP_MODE_SV64: usize = 11;

pub const HGATP_MODE_OFF: usize = 0;
pub const HGATP_MODE_SV32X4: usize = 1;
pub const HGATP_MODE_SV39X4: usize = 8;
pub const HGATP_MODE_SV48X4: usize = 9;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

#[cfg(target_pointer_width = "64")]
pub mod Const {
	pub const MSTATUS64_SD: usize = 0x8000000000000000;
	pub const SSTATUS64_SD: usize = MSTATUS64_SD;
	pub const SATP64_MODE: usize = 0xF000000000000000;
	pub const HGATP64_PPN: usize = 0x00000FFFFFFFFFFF;
	pub const HGATP64_VMID_SHIFT: usize = 44;
	pub const HGATP64_VMID_MASK: usize = 0x03FFF00000000000;
	pub const HGATP64_MODE_SHIFT: usize = 60;

    pub const MSTATUS_SD: usize = MSTATUS64_SD;
	pub const SSTATUS_SD: usize = SSTATUS64_SD;
	pub const SATP_MODE: usize = SATP64_MODE;

	pub const HGATP_PPN: usize = HGATP64_PPN;
	pub const HGATP_VMID_SHIFT: usize = HGATP64_VMID_SHIFT;
	pub const HGATP_VMID_MASK: usize = HGATP64_VMID_MASK;
	pub const HGATP_MODE_SHIFT: usize = HGATP64_MODE_SHIFT;
}

#[cfg(target_pointer_width = "32")]
pub mod Const {
	pub const MSTATUS32_SD: usize = 0x80000000;
	pub const SSTATUS32_SD: usize = MSTATUS32_SD;
	pub const SATP32_MODE: usize = 0x80000000;
	pub const HGATP32_PPN: usize = 0x003FFFFF;
	pub const HGATP32_VMID_SHIFT: usize = 22;
	pub const HGATP32_VMID_MASK: usize = 0x1FC00000;
	pub const HGATP32_MODE_SHIFT: usize = 31;

    pub const MSTATUS_SD: usize = MSTATUS32_SD;
	pub const SSTATUS_SD: usize = SSTATUS32_SD;
	pub const SATP_MODE: usize = SATP32_MODE;

	pub const HGATP_PPN: usize = HGATP32_PPN;
	pub const HGATP_VMID_SHIFT: usize = HGATP32_VMID_SHIFT;
	pub const HGATP_VMID_MASK: usize = HGATP32_VMID_MASK;
	pub const HGATP_MODE_SHIFT: usize = HGATP32_MODE_SHIFT;
}


/**
 * Maximum number of bits in a hartmask
 *
 * The hartmask is indexed using physical HART id so this define
 * also represents the maximum number of HART ids generic OpenSBI
 * can handle.
 */
const SBI_HARTMASK_MAX_BITS: usize = 128;

/** Representation of hartmask */
struct sbi_hartmask {
	DECLARE_BITMAP(bits, SBI_HARTMASK_MAX_BITS);
}


pub struct sbi_tlb_info {
	start: usize,
	size: usize,
	asid: usize,
	vmid: usize,
	void (*local_fn)(struct sbi_tlb_info *tinfo);
	struct sbi_hartmask smask;
}

pub fn SBI_TLB_INFO_INIT(__p, __start, __size, __asid, __vmid, __lfn, __src) { 
	(__p)->start = (__start); \
	(__p)->size = (__size); \
	(__p)->asid = (__asid); \
	(__p)->vmid = (__vmid); \
	(__p)->local_fn = (__lfn); \
	SBI_HARTMASK_INIT_EXCEPT(&(__p)->smask, (__src)); \
} while (0)

pub fn csr_read(csr: &str) -> usize {
	let __v: usize;
	__asm__ __volatile__("csrr %0, " __ASM_STR(csr)
				     : "=r"(__v)                
				     :                          
				     : "memory");               
		__v;
	unsafe {
		asm!(
			"csrr %0, {csr}",
			
		);
	}
}

pub fn csr_write(csr: &str, val: usize) {
	let __v: usize = val;
	__asm__ __volatile__("csrw " __ASM_STR(csr) ", %0"
				:
				: "rK"(__v)
				: "memory");
}

pub fn csr_clear(csr: &str, val: usize) {
	let __v: usize = val;
	__asm__ __volatile__("csrc " __ASM_STR(csr) ", %0"
					:
					: "rK"(__v)
					: "memory");
}

pub fn csr_set(csr: &str, val: usize) {
	let __v: usize = val;
	__asm__ __volatile__("csrs " __ASM_STR(csr) ", %0"
					:
					: "rK"(__v)
					: "memory");
}


pub fn spin_lock(spinlock_t *lock) {
	let inc: usize = 1u << TICKET_SHIFT;
	let mask: usize = 0xffffu;
	let l0: u32;
	let tmp1: u32;
	let tmp2: u32;

	__asm__ __volatile__(
		/* Atomically increment the next ticket. */
		"	amoadd.w.aqrl	%0, %4, %3\n"

		/* Did we get the lock? */
		"	srli	%1, %0, %6\n"
		"	and	%1, %1, %5\n"
		"1:	and	%2, %0, %5\n"
		"	beq	%1, %2, 2f\n"

		/* If not, then spin on the lock. */
		"	lw	%0, %3\n"
		RISCV_ACQUIRE_BARRIER
		"	j	1b\n"
		"2:"
		: "=&r"(l0), "=&r"(tmp1), "=&r"(tmp2), "+A"(*lock)
		: "r"(inc), "r"(mask), "I"(TICKET_SHIFT)
		: "memory");
}

// fn spin_unlock(spinlock_t* lock) {
// 	__smp_store_release(&lock->owner, lock->owner + 1);
// }

// fn sbi_trap_exit(regs: &mut sbi_trap_regs) {
// 	let scratch: sbi_scratch = sbi_scratch_thishart_ptr();

// 	((trap_exit_t)scratch->trap_exit)(regs);
// 	__builtin_unreachable();
// }

/* Get current HART id */
pub fn current_hartid() -> usize { 
	csr_read("CSR_MHARTID")
}

pub fn sbi_hart_hang() {
	use riscv::asm::wfi;
	loop {
		wfi();
	}
	__builtin_unreachable();
}

pub fn sbi_memset(void *s, int c, size_t count) -> usize {
	char *temp = s;

	while (count > 0) {
		count--;
		*temp++ = c;
	}

	return s;
}

pub fn sbi_memcpy(void *dest, const void *src, size_t count) -> usize {
	char *temp1 = dest;
	const char *temp2 = src;

	while (count > 0) {
		*temp1++ = *temp2++;
		count--;
	}

	return dest;
}

/** Get pointer to sbi_domain for current HART */
pub fn sbi_domain_thishart_ptr() {
	sbi_hartid_to_domain(current_hartid())
}

/* Read & Write Memory barrier */
pub fn mb() { 
	RISCV_FENCE(iorw,iorw);
}

/**
 * Exit trap/interrupt handling
 *
 * This function is called by non-firmware code to abruptly exit
 * trap/interrupt handling and resume execution at context pointed
 * by given register state.
 *
 * @param regs pointer to register state
 */
pub fn sbi_trap_exit(const struct sbi_trap_regs *regs) {
	struct sbi_scratch *scratch = sbi_scratch_thishart_ptr();

	((trap_exit_t)scratch->trap_exit)(regs);
	__builtin_unreachable();
}

// typedef struct {
// 	#if __BYTE_ORDER__ == __ORDER_BIG_ENDIAN__
// 		   u16 next;
// 		   u16 owner;
// 	#else
// 		   u16 owner;
// 		   u16 next;
// 	#endif
// 	} __aligned(4) spinlock_t;
	
// 	#define __SPIN_LOCK_UNLOCKED	\
// 		(spinlock_t) { 0, 0 }
	
// 	#define SPIN_LOCK_INIT(x)	\
// 		x = __SPIN_LOCK_UNLOCKED
	
// 	#define SPIN_LOCK_INITIALIZER	\
// 		__SPIN_LOCK_UNLOCKED
	
// 	#define DEFINE_SPIN_LOCK(x)	\
// 		spinlock_t SPIN_LOCK_INIT(x)