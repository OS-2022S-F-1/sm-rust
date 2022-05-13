use crate::error_code::ERROR;
use crate::enclave;

pub struct platform_enclave_data {
    
}

impl platform_enclave_data {
    pub fn new() -> Self {
        Self {

        }
    }
}

pub fn platform_create_enclave(enclave: &mut enclave::enclave) -> usize {
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
}

pub fn platform_destroy_enclave(enclave: &mut enclave::enclave){
    return;
}

pub fn platform_init_enclave(enclave: &mut enclave::enclave){
    return;
}

pub fn platform_switch_from_enclave(enclave: &mut enclave::enclave){
    return;
}

pub fn platform_switch_to_enclave(enclave: &mut enclave::enclave){
    return;
}

pub fn platform_init_global() -> usize {
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
}

pub fn platform_init_global_once() -> usize {
    return ERROR::SBI_ERR_SM_ENCLAVE_SUCCESS;
}