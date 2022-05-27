use core::arch::asm;
use crate::sbi_trap;

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

pub const PMP_R: u8 = 0x01;
pub const PMP_W: u8 = 0x02;
pub const PMP_X: u8 = 0x04;
pub const PMP_A: u8 = 0x18;
pub const PMP_A_TOR: u8 = 0x08;
pub const PMP_A_NA4: u8 = 0x10;
pub const PMP_A_NAPOT: u8 = 0x18;
pub const PMP_L: u8 = 0x80;

pub const IRQ_S_SOFT: usize = 1;
pub const IRQ_VS_SOFT: usize = 2;
pub const IRQ_M_SOFT: usize = 3;
pub const IRQ_S_TIMER: usize = 5;
pub const IRQ_VS_TIMER: usize = 6;
pub const IRQ_M_TIMER: usize = 7;
pub const IRQ_S_EXT: usize = 9;
pub const IRQ_VS_EXT: usize = 10;
pub const IRQ_M_EXT: usize = 11;
pub const IRQ_S_GEXT: usize = 12;
pub const IRQ_PMU_OVF: usize = 13;

pub const MIP_SSIP: usize = (1 << IRQ_S_SOFT);
pub const MIP_VSSIP: usize = (1 << IRQ_VS_SOFT);
pub const MIP_MSIP: usize = (1 << IRQ_M_SOFT);
pub const MIP_STIP: usize = (1 << IRQ_S_TIMER);
pub const MIP_VSTIP: usize = (1 << IRQ_VS_TIMER);
pub const MIP_MTIP: usize = (1 << IRQ_M_TIMER);
pub const MIP_SEIP: usize = (1 << IRQ_S_EXT);
pub const MIP_VSEIP: usize = (1 << IRQ_VS_EXT);
pub const MIP_MEIP: usize = (1 << IRQ_M_EXT);
pub const MIP_SGEIP: usize = (1 << IRQ_S_GEXT);
pub const MIP_LCOFIP: usize = (1 << IRQ_PMU_OVF);

/* clang-format off */
pub const MSTATUS_SIE: usize = 0x00000002;
pub const MSTATUS_MIE: usize = 0x00000008;
pub const MSTATUS_SPIE_SHIFT: usize = 5;
pub const MSTATUS_SPIE: usize = (1 << MSTATUS_SPIE_SHIFT);
pub const MSTATUS_UBE: usize = 0x00000040;
pub const MSTATUS_MPIE: usize = 0x00000080;
pub const MSTATUS_SPP_SHIFT: usize = 8;
pub const MSTATUS_SPP: usize = (1 << MSTATUS_SPP_SHIFT);
pub const MSTATUS_MPP_SHIFT: usize = 11;
pub const MSTATUS_MPP: usize = (3 << MSTATUS_MPP_SHIFT);
pub const MSTATUS_FS: usize = 0x00006000;
pub const MSTATUS_XS: usize = 0x00018000;
pub const MSTATUS_VS: usize = 0x01800000;
pub const MSTATUS_MPRV: usize = 0x00020000;
pub const MSTATUS_SUM: usize = 0x00040000;
pub const MSTATUS_MXR: usize = 0x00080000;
pub const MSTATUS_TVM: usize = 0x00100000;
pub const MSTATUS_TW: usize = 0x00200000;
pub const MSTATUS_TSR: usize = 0x00400000;
pub const MSTATUS32_SD: usize = 0x80000000;


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

	pub const BITS_PER_LONG: usize = 64;
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

	pub const BITS_PER_LONG: usize = 32;
}

pub const TICKET_SHIFT: usize = 16;

#[repr(align(4))]
#[cfg(target_endian = "big")]
pub struct spinlock_t {
	pub next: u16,
    pub owner: u16
}

#[repr(align(4))]
#[cfg(target_endian = "little")]
pub struct spinlock_t {
	pub owner: u16,
	pub next: u16
}

impl spinlock_t {
	pub fn new() -> Self {
		Self {
			next: 0,
			owner: 0
		}
	}
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
	::std::slice::from_raw_parts(
		(p as *const T) as *const u8,
		::std::mem::size_of::<T>(),
	)
}

unsafe fn any_as_usize_slice<T: Sized>(p: &T) -> &[usize] {
	::std::slice::from_raw_parts(
		(p as *const T) as *const usize,
		::std::mem::size_of::<T>(),
	)
}


/**
 * Maximum number of bits in a hartmask
 *
 * The hartmask is indexed using physical HART id so this define
 * also represents the maximum number of HART ids generic OpenSBI
 * can handle.
 */
const SBI_HARTMASK_MAX_BITS: usize = 128;

fn BITS_TO_LONGS(nbits: usize) -> usize {
	return ((nbits) + Const::BITS_PER_LONG - 1) / Const::BITS_PER_LONG;
}

/** Representation of hartmask */
struct sbi_hartmask {
	pub bits: [usize;((SBI_HARTMASK_MAX_BITS) + Const::BITS_PER_LONG - 1) / Const::BITS_PER_LONG]
}

pub struct atomic_t {
	counter: usize
}


pub struct sbi_tlb_info {
	pub start: usize,
	pub size: usize,
	pub asid: usize,
	pub vmid: usize,
	pub local_fn: fn(*mut sbi_tlb_info),
	pub smask: sbi_hartmask
}

fn RISCV_FENCE(p: &str, s: &str) {
	// __asm__ __volatile__ ("fence " #p "," #s : : : "memory")
	unsafe {
		asm!(
			format!("fence {}, {}", p, s),
		);
	}
}

pub fn csr_read(csr: &str) -> usize {
	let __v: usize;
	// __asm__ __volatile__("csrr %0, " __ASM_STR(csr)
	// 			     : "=r"(__v)                
	// 			     :                          
	// 			     : "memory");               
	//  __v;
	unsafe {
		asm!(
			format!("csrr {0}, {}", csr),
			out(reg) __v
		);
	}
	__v
}

pub fn csr_write(csr: &str, val: usize) {
	let __v: usize = val;
	// __asm__ __volatile__("csrw " __ASM_STR(csr) ", %0"
	// 			:
	// 			: "rK"(__v)
	// 			: "memory");
	unsafe {
		asm!(
			format!("csrw {}, {0}", csr),
			in(reg) __v
		);
	}
}

pub fn csr_clear(csr: &str, val: usize) {
	let __v: usize = val;
	// __asm__ __volatile__("csrc " __ASM_STR(csr) ", %0"
	// 				:
	// 				: "rK"(__v)
	// 				: "memory");
	unsafe {
		asm!(
			format!("csrc {}, {0}", csr),
			in(reg) __v
		);
	}
}

pub fn csr_set(csr: &str, val: usize) {
	let __v: usize = val;
	// __asm__ __volatile__("csrs " __ASM_STR(csr) ", %0"
	// 				:
	// 				: "rK"(__v)
	// 				: "memory");
	unsafe {
		asm!(
			format!("csrs {}, {0}", csr),
			in(reg) __v
		);
	}
}


pub fn spin_lock(lock: &mut spinlock_t) {
	let inc: usize = 1 << TICKET_SHIFT;
	let mask: usize = 0xffff;
	let l0: u32;
	let tmp1: u32;
	let tmp2: u32;

	// __asm__ __volatile__(
	// 	/* Atomically increment the next ticket. */
	// 	"	amoadd.w.aqrl	%0, %4, %3\n"

	// 	/* Did we get the lock? */
	// 	"	srli	%1, %0, %6\n"
	// 	"	and	%1, %1, %5\n"
	// 	"1:	and	%2, %0, %5\n"
	// 	"	beq	%1, %2, 2f\n"

	// 	/* If not, then spin on the lock. */
	// 	"	lw	%0, %3\n"
	// 	RISCV_ACQUIRE_BARRIER
	// 	"	j	1b\n"
	// 	"2:"
	// 	: "=&r"(l0), "=&r"(tmp1), "=&r"(tmp2), "+A"(*lock)
	// 	: "r"(inc), "r"(mask), "I"(TICKET_SHIFT)
	// 	: "memory");
	unsafe {
		asm!(
			/* Atomically increment the next ticket. */
			" amoadd.w.aqrl {0}, {4}, {3}",
			
			/* Did we get the lock? */
			" srli {1}, {0}, {6}",
			" and {1}, {1}, {5}",
			"1:	and	{2}, {0}, {5}",
			" beq {1}, {2}, 2f",

			/* If not, then spin on the lock. */
			" lw {0}, {3}",
			" fence r , rw",
			" j 1b",
			"2:",
			out(reg) l0,
			out(reg) tmp1,
			out(reg) tmp2,
			inout(reg) *lock,
			in(reg) inc,
			in(reg) mask,
			in(reg) TICKET_SHIFT
		);
	}
}

pub fn __smp_store_release(p: &mut u16, v: u16) { 
	RISCV_FENCE("rw", "w");
	*p = v;
}

pub fn spin_unlock(lock: &mut spinlock_t) {
	__smp_store_release(&mut (lock).owner, (lock).owner + 1);
}

/* Get current HART id */
pub fn current_hartid() -> usize { 
	csr_read("mhartid")
}

pub fn sbi_hart_hang() {
	unsafe {
		use riscv::asm::wfi;
		loop {
			wfi();
		}
		unreachable!("Unreachable!");
	}
}

pub fn sbi_memset(s: usize, c: u8, count: usize) -> usize {
	let temp: &[u8] = any_as_u8_slice(&s);

	for i in 0..count {
		temp[i] = c;
	}

	return s;
}

pub fn sbi_memcpy(dest: usize, src: usize, count: usize) -> usize {
	let temp1: &[u8] = any_as_u8_slice(&dest);
	let temp2: &[u8] = any_as_u8_slice(&src);
	
	for i in 0..count {
		temp1[i] = temp2[i];
	}

	return dest;
}

// /** Get pointer to sbi_domain for current HART */
// pub fn sbi_domain_thishart_ptr() {
// 	sbi_hartid_to_domain(current_hartid())
// }

/* Read & Write Memory barrier */
pub fn mb() { 
	RISCV_FENCE("iorw", "iorw");
}

/** Get pointer to sbi_scratch for current HART */
fn sbi_scratch_thishart_ptr() -> *mut sbi_scratch {
	return csr_read("mscratch") as *mut sbi_scratch;
}

/** Representation of per-HART scratch space */
struct sbi_scratch {
	/** Start (or base) address of firmware linked to OpenSBI library */
    fw_start: usize,
    /** Size (in bytes) of firmware linked to OpenSBI library */
    fw_size: usize,
    /** Arg1 (or 'a1' register) of next booting stage for this HART */
    next_arg1: usize,
    /** Address of next booting stage for this HART */
    next_addr: usize,
    /** Priviledge mode of next booting stage for this HART */
    next_mode: usize,
    /** Warm boot entry point address for this HART */
    warmboot_addr: usize,
    /** Address of sbi_platform */
    platform_addr: usize,
    /** Address of HART ID to sbi_scratch conversion function */
    hartid_to_scratch: usize,
    /** Address of trap exit function */
    trap_exit: fn(*mut sbi_trap::sbi_trap_regs),
    /** Temporary storage */
    tmp0: usize,
    /** Options for OpenSBI library */
    options: usize,
}

/**
 * As this this function only handlers scalar values of hart mask, it must be
 * set to all online harts if the intention is to send IPIs to all the harts.
 * If hmask is zero, no IPIs will be sent.
 */
// fn sbi_ipi_send_many(ulong hmask, ulong hbase, u32 event, void *data) -> i32 {
// 	int rc;
// 	ulong i, m;
// 	struct sbi_domain *dom = sbi_domain_thishart_ptr();
// 	struct sbi_scratch *scratch = sbi_scratch_thishart_ptr();

// 	if (hbase != -1UL) {
// 		rc = sbi_hsm_hart_interruptible_mask(dom, hbase, &m);
// 		if (rc)
// 			return rc;
// 		m &= hmask;

// 		/* Send IPIs */
// 		for (i = hbase; m; i++, m >>= 1) {
// 			if (m & 1UL)
// 				sbi_ipi_send(scratch, i, event, data);
// 		}
// 	} else {
// 		hbase = 0;
// 		while (!sbi_hsm_hart_interruptible_mask(dom, hbase, &m)) {
// 			/* Send IPIs */
// 			for (i = hbase; m; i++, m >>= 1) {
// 				if (m & 1UL)
// 					sbi_ipi_send(scratch, i, event, data);
// 			}
// 			hbase += BITS_PER_LONG;
// 		}
// 	}

// 	return 0;
// }

/**
 * Exit trap/interrupt handling
 *
 * This function is called by non-firmware code to abruptly exit
 * trap/interrupt handling and resume execution at context pointed
 * by given register state.
 *
 * @param regs pointer to register state
 */
pub fn sbi_trap_exit(regs: *mut sbi_trap::sbi_trap_regs) {
	let scratch: *mut sbi_scratch = sbi_scratch_thishart_ptr();
	let trap_exit_ptr: fn(*mut sbi_trap::sbi_trap_regs) = (*scratch).trap_exit;
	trap_exit_ptr(regs);
	unreachable!("Unreachable!");
}

// pub fn sbi_tlb_request(hmask: usize, hbase: usize, tinfo: *mut sbi_tlb_info) -> i32
// {
// 	if !(*tinfo).local_fn {
// 		return -3; // SBI_ERR_INVALID_PARAM
// 	}

// 	tlb_pmu_incr_fw_ctr(tinfo);

// 	return sbi_ipi_send_many(hmask, hbase, tlb_event, tinfo);
// }

fn bitmap_zero_except(dst: &[usize], exception: usize, nbits: usize) {
	if nbits < Const::BITS_PER_LONG {
		dst[0] = 0;
	}
	else {
		let i: usize;
		let len: usize = BITS_TO_LONGS(nbits);
		for i in 0..len {
			dst[i] = 0;
		}
	}
	if exception < nbits {
		let val: usize = 1 << (exception % Const::BITS_PER_LONG);
		dst[exception / Const::BITS_PER_LONG] |= val;
	}
}

// /** Initialize hartmask to zero except a particular HART id */
pub fn SBI_HARTMASK_INIT_EXCEPT(__m: sbi_hartmask, __h: usize)	{
	bitmap_zero_except(&(__m).bits, __h, SBI_HARTMASK_MAX_BITS)
}

	
fn __SPIN_LOCK_UNLOCKED() -> spinlock_t {
	spinlock_t::new()
}
	
pub fn SPIN_LOCK_INIT(x: &mut spinlock_t) {	
	*x = __SPIN_LOCK_UNLOCKED();
}

pub fn SPIN_LOCK_INITIALIZER() -> spinlock_t {
	__SPIN_LOCK_UNLOCKED()
}



// qemu max hart id 
fn sbi_platform_hart_count() -> usize {
	8
}

/**
 * Get ulong HART mask for given HART base ID
 * @param scratch the per-HART scratch pointer
 * @param hbase the HART base ID
 * @param out_hmask the output ulong HART mask
 * @return 0 on success and SBI_Exxx (< 0) on failure
 * Note: the output HART mask will be set to zero on failure as well.
 */
pub fn sbi_hsm_hart_started_mask(hbase: usize, out_hmask: &mut usize) -> i32 {
	let i: usize;
	let hcount: usize = sbi_platform_hart_count();
  
	/*
	* The SBI_HARTMASK_MAX_BITS represents the maximum HART ids generic
	* OpenSBI can handle whereas sbi_platform_hart_count() represents
	* the maximum HART ids (or HARTs) on underlying platform.
	*
	* Currently, we only support continuous HART ids so this function
	* is written with same assumption. In future, this function will
	* change when we support discontinuous and sparse HART ids.
	*/
  
	*out_hmask = 0;
	if hcount <= hbase {
		return -3; // SBI_ERR_INVALID_PARAM
	}
	if Const::BITS_PER_LONG < (hcount - hbase) {
		hcount = Const::BITS_PER_LONG;
	}
  
	for i in hbase..hcount {
		if HSM.hart_get_status(i) == HsmState::Started {
			*out_hmask |= 1 << (i - hbase);
		}
	}
  
	return 0;
}

// pub fn sbi_ecall_register_extension(struct sbi_ecall_extension *ext) -> i32 {
// 	struct sbi_ecall_extension *t;

// 	if (!ext || (ext->extid_end < ext->extid_start) || !ext->handle)
// 		return SBI_EINVAL;

// 	sbi_list_for_each_entry(t, &ecall_exts_list, head) {
// 		unsigned long start = t->extid_start;
// 		unsigned long end = t->extid_end;
// 		if (end < ext->extid_start || ext->extid_end < start)
// 			/* no overlap */;
// 		else
// 			return SBI_EINVAL;
// 	}

// 	SBI_INIT_LIST_HEAD(&ext->head);
// 	sbi_list_add_tail(&ext->head, &ecall_exts_list);

// 	return 0;
// }