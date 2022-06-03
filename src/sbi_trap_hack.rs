use crate::enclave;
use crate::sbi_trap;
use crate::opensbi;
use crate::sm_sbi;
use crate::error_code::ERROR;
// #include <sbi/riscv_asm.h>
// #include <sbi/riscv_encoding.h>
// #include <sbi/sbi_console.h>
// #include <sbi/sbi_ecall.h>
// #include <sbi/sbi_error.h>
// #include <sbi/sbi_hart.h>
// #include <sbi/sbi_illegal_insn.h>
// #include <sbi/sbi_ipi.h>
// #include <sbi/sbi_misaligned_ldst.h>
// #include <sbi/sbi_timer.h>
// #include <sbi/sbi_trap.h>

/** Representation of trap details */
struct sbi_trap_info {
	/** epc Trap program counter */
	epc: usize,
	/** cause Trap exception cause */
	cause: usize,
	/** tval Trap value */
	tval: usize,
	/** tval2 Trap value 2 */
	tval2: usize,
	/** tinst Trap instruction */
	tinst: usize
}

/**
 * Handle trap/interrupt
 *
 * This function is called by firmware linked to OpenSBI
 * library for handling trap/interrupt. It expects the
 * following:
 * 1. The 'mscratch' CSR is pointing to sbi_scratch of current HART
 * 2. The 'mcause' CSR is having exception/interrupt cause
 * 3. The 'mtval' CSR is having additional trap information
 * 4. The 'mtval2' CSR is having additional trap information
 * 5. The 'mtinst' CSR is having decoded trap instruction
 * 6. Stack pointer (SP) is setup for current HART
 * 7. Interrupts are disabled in MSTATUS CSR
 *
 * @param regs pointer to register state
 */
pub fn sbi_trap_handler_keystone_enclave(regs: &mut sbi_trap::sbi_trap_regs) {
	let rc: i32 = -2; // SBI_ENOTSUPP
	// let msg: str = "trap handler failed";
	let mcause: usize = opensbi::csr_read("CSR_MCAUSE");
	let mtval: usize = opensbi::csr_read("CSR_MTVAL");
	let mtval2: usize = 0;
	let mtinst: usize = 0;
	let trap: sbi_trap_info;
	let __riscv_xlen: usize = 64;

	// if (misa_extension('H')) {
	// 	mtval2 = csr_read(CSR_MTVAL2);
	// 	mtinst = csr_read(CSR_MTINST);
	// }

	if mcause & (1 << (__riscv_xlen - 1)) != 0 {
		mcause &= !(1 << (__riscv_xlen - 1));
		match mcause {
			IRQ_M_TIMER => {
				regs.mepc -= 4;
				sm_sbi::sbi_sm_stop_enclave(regs, enclave::STOP_TIMER_INTERRUPT);
				regs.a0 = ERROR::SBI_ERR_SM_ENCLAVE_INTERRUPTED;
				regs.mepc += 4;
			},
			IRQ_M_SOFT => {
				regs.mepc -= 4;
				sm_sbi::sbi_sm_stop_enclave(regs, enclave::STOP_TIMER_INTERRUPT);
				regs.a0 = ERROR::SBI_ERR_SM_ENCLAVE_INTERRUPTED;
				regs.mepc += 4;
			},
			_ => {
				msg = "unhandled external interrupt";
				sbi_trap_error(msg, rc, mcause, mtval, mtval2, mtinst, regs);
			}
		}
		return;
	}

	match mcause {
		CAUSE_ILLEGAL_INSTRUCTION => {
			rc  = sbi_illegal_insn_handler(mtval, regs);
			msg = "illegal instruction handler failed";
		}
	case CAUSE_MISALIGNED_LOAD:
		rc = sbi_misaligned_load_handler(mtval, mtval2, mtinst, regs);
		msg = "misaligned load handler failed";
		break;
	case CAUSE_MISALIGNED_STORE:
		rc  = sbi_misaligned_store_handler(mtval, mtval2, mtinst, regs);
		msg = "misaligned store handler failed";
		break;
	case CAUSE_SUPERVISOR_ECALL:
	case CAUSE_MACHINE_ECALL:
		rc  = sbi_ecall_handler(regs);
		msg = "ecall handler failed";
		break;
	default:
		/* If the trap came from S or U mode, redirect it there */
		trap.epc = regs->mepc;
		trap.cause = mcause;
		trap.tval = mtval;
		trap.tval2 = mtval2;
		trap.tinst = mtinst;
		rc = sbi_trap_redirect(regs, &trap);
		break;
	};

trap_error:
	if (rc)
		sbi_trap_error(msg, rc, mcause, mtval, mtval2, mtinst, regs);
}
