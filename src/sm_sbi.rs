use crate::enclave;
use crate::sm;
use crate::cpu;
use crate::opensbi;
use crate::trap_sbi;

fn sbi_sm_create_enclave(eid: *mut usize, create_args: usize) -> usize {
    let create_args_local: sm::keystone_sbi_create; // sm.rs encalve argument
    let ret: usize = enclave::copy_enclave_create_args(create_args, &mut create_args_local) as usize; // enclave.rs

    if ret != 0 { // create fail
        return ret;
    }

    ret = enclave::create_enclave(eid, create_args_local) as usize; // enclave.rs
    return ret;
}

fn sbi_sm_destroy_enclave(eid: usize) -> usize {
    let ret: usize = enclave::destroy_enclave(eid); // enclave.rs
    return ret;
}

fn sbi_sm_run_enclave(regs: &mut trap_sbi::sbi_trap_regs /*opensbi*/, eid: usize) -> usize {
    regs.a0 = enclave::run_enclave(regs, eid); // enclave.rs
    regs.mepc += 4;
    opensbi::sbi_trap_exit(regs); // opensbi 提供函数
    0
}

fn sbi_sm_exit_enclave(regs: &mut trap_sbi::sbi_trap_regs, retval: usize) -> usize {
    regs.a0 = enclave::exit_enclave(regs, cpu::cpu_get_enclave_id()); // enclave.rs
    regs.a1 = retval;
    regs.mepc += 4;
    opensbi::sbi_trap_exit(regs); // opensbi 提供函数
    0
}

fn sbi_sm_stop_enclave(regs: &mut trap_sbi::sbi_trap_regs, request: usize) -> usize {
    regs.a0 = enclave::stop_enclave(regs, request, cpu::cpu_get_enclave_id()); // enclave.rs
    regs.mepc += 4;
    opensbi::sbi_trap_exit(regs); // opensbi 提供函数
    0
}

fn sbi_sm_resume_enclave(regs: &mut trap_sbi::sbi_trap_regs, eid: usize) -> usize {
    let ret: usize = enclave::resume_enclave(regs, eid); // enclave.rs
    if !regs.zero {
        regs.a0 = ret;
    }
    regs.mepc += 4;

    opensbi::sbi_trap_exit(regs); // opensbi 提供函数
    return 0;
}

fn sbi_sm_attest_enclave(report: usize, data: usize, size: usize) -> usize {
    let ret: usize = enclave::attest_enclave(report, data, size, cpu::cpu_get_enclave_id()); // enclave.rs
    return ret;
}

fn sbi_sm_get_sealing_key(seal_key: usize, key_ident: usize, key_ident_size: usize) -> usize {
    let ret: usize = enclave::get_sealing_key(seal_key, key_ident, key_ident_size, cpu::cpu_get_enclave_id()); // enclave.rs cpu.rs
    return ret;
}

fn sbi_sm_random() -> usize {
    return platform_random();
}


fn sbi_sm_call_plugin(plugin_id: usize, call_id: usize, arg0: usize, arg1: usize) -> usize {
    0
}