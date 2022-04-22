/* error codes */
pub mod ERROR {
    pub const SBI_ERR_SM_ENCLAVE_SUCCESS: usize = 0;
    pub const SBI_ERR_SM_ENCLAVE_UNKNOWN_ERROR: usize = 100000;
    pub const SBI_ERR_SM_ENCLAVE_INVALID_ID: usize = 100001;
    pub const SBI_ERR_SM_ENCLAVE_INTERRUPTED: usize = 100002;
    pub const SBI_ERR_SM_ENCLAVE_PMP_FAILURE: usize = 100003;
    pub const SBI_ERR_SM_ENCLAVE_NOT_RUNNABLE: usize = 100004;
    pub const SBI_ERR_SM_ENCLAVE_NOT_DESTROYABLE: usize = 100005;
    pub const SBI_ERR_SM_ENCLAVE_REGION_OVERLAPS: usize = 100006;
    pub const SBI_ERR_SM_ENCLAVE_NOT_ACCESSIBLE: usize = 100007;
    pub const SBI_ERR_SM_ENCLAVE_ILLEGAL_ARGUMENT: usize = 100008;
    pub const SBI_ERR_SM_ENCLAVE_NOT_RUNNING: usize = 100009;
    pub const SBI_ERR_SM_ENCLAVE_NOT_RESUMABLE: usize = 100010;
    pub const SBI_ERR_SM_ENCLAVE_EDGE_CALL_HOST: usize = 100011;
    pub const SBI_ERR_SM_ENCLAVE_NOT_INITIALIZED: usize = 100012;
    pub const SBI_ERR_SM_ENCLAVE_NO_FREE_RESOURCE: usize = 100013;
    pub const SBI_ERR_SM_ENCLAVE_SBI_PROHIBITED: usize = 100014;
    pub const SBI_ERR_SM_ENCLAVE_ILLEGAL_PTE: usize = 100015;
    pub const SBI_ERR_SM_ENCLAVE_NOT_FRESH: usize = 100016;
    pub const SBI_ERR_SM_DEPRECATED: usize = 100099;
    pub const SBI_ERR_SM_NOT_IMPLEMENTED: usize = 100100;

    pub const SBI_ERR_SM_PMP_SUCCESS: usize = 0;
    pub const SBI_ERR_SM_PMP_REGION_SIZE_INVALID: usize = 100020;
    pub const SBI_ERR_SM_PMP_REGION_NOT_PAGE_GRANULARITY: usize = 100021;
    pub const SBI_ERR_SM_PMP_REGION_NOT_ALIGNED: usize = 100022;
    pub const SBI_ERR_SM_PMP_REGION_MAX_REACHED: usize = 100023;
    pub const SBI_ERR_SM_PMP_REGION_INVALID: usize = 100024;
    pub const SBI_ERR_SM_PMP_REGION_OVERLAP: usize = 100025;
    pub const SBI_ERR_SM_PMP_REGION_IMPOSSIBLE_TOR: usize = 100026;
}

