fn sbi_sm_create_enclave(out_val: *mut u32, create_args: usize) -> u32 {
    keystone_sbi_create create_args_local; // sm.rs encalve argument
    let ret: u32 = copy_enclave_create_args(create_args, &create_args_local); // enclave.rs

    if ret { // create fail
        return ret;
    }

    ret = create_enclave(eid, create_args_local); // enclave.rs
    return ret;
}

fn sbi_sm_destroy_enclave(eid: u32) -> u32 {
    let ret: u32 = destroy_enclave(eid); // enclave.rs
    return ret;
}

fn sbi_sm_run_enclave(regs: *mut sbi_trap_regs, eid: u32) -> u32 {
    regs->a0 = run_enclave(regs, eid); // enclave.rs
    regs->mepc += 4;
    sbi_trap_exit(regs); // opensbi 提供函数
    0
}

fn sbi_sm_exit_enclave(regs: *mut sbi_trap_regs, retval: u32) -> u32 {
    regs->a0 = exit_enclave(regs, cpu_get_enclave_id()); // enclave.rs
    regs->a1 = retval;
    regs->mepc += 4;
    sbi_trap_exit(regs); // opensbi 提供函数
    0
}

fn sbi_sm_stop_enclave(regs: *mut sbi_trap_regs, request: u32) -> u32 {
    regs->a0 = stop_enclave(regs, request, cpu_get_enclave_id()); // enclave.rs
    regs->mepc += 4;
    sbi_trap_exit(regs); // opensbi 提供函数
    0
}

fn sbi_sm_resume_enclave(regs: *mut sbi_trap_regs, eid: u32) -> u32 {
    let ret: u32 = resume_enclave(regs, eid); // enclave.rs
    if !regs->zero {
        regs->a0 = ret;
    }
    regs->mepc += 4;

    sbi_trap_exit(regs); // opensbi 提供函数
    return 0;
}

fn sbi_sm_attest_enclave(report: u32, data: u32, size: u32) -> u32 {
    let ret: u32 = attest_enclave(report, data, size, cpu_get_enclave_id()); // enclave.rs
    return ret;
}

fn sbi_sm_get_sealing_key(seal_key: u32, key_ident: u32, key_ident_size: u32) -> u32 {
    let ret: u32 = get_sealing_key(seal_key, key_ident, key_ident_size, cpu_get_enclave_id()); // enclave.rs cpu.rs
    return ret;
}

fn sbi_sm_random() -> u32 {
    return platform_random();
}


fn sbi_sm_call_plugin(plugin_id: u32, call_id: u32, arg0: u32, arg1: u32) -> u32 {
    
}