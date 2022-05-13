use crate::enclave;

#[cfg(target_pointer_width = "32")]
pub mod log {
    pub const LOG_REGBYTES: usize = 2;
}

#[cfg(target_pointer_width = "64")]
pub mod log {
    pub const LOG_REGBYTES: usize = 3;
}

struct mprv_block { 
    words: [usize;8]
}

const REGBYTES: usize = 1 << log::LOG_REGBYTES;
const MPRV_BLOCK: usize = REGBYTES * 8;

pub fn copy_from_sm(dst: usize, src: usize, len: usize) -> i32 {
    
    unsafe {

        if src % REGBYTES  == 0 && dst % REGBYTES == 0 {
            while len >= MPRV_BLOCK {
                let res: i32 = copy_block_from_sm(dst, src as *const mprv_block); // mprv.s
                if res != 0 {
                    return res;
                }
    
                src += MPRV_BLOCK;
                dst += MPRV_BLOCK;
                len -= MPRV_BLOCK;
            }

            while len >= REGBYTES {
                let res: i32 = copy_word_from_sm(dst, src as *const usize); // mprv.s
                if res != 0 {
                    return res;
                }
    
                src += REGBYTES;
                dst += REGBYTES;
                len -= REGBYTES;
            }
        }
    
        while len > 0 {
            let res: i32 = copy1_from_sm(dst, src as *const u8); // mprv.s
            if res != 0 {
                return res;
            }
    
            src += 1;
            dst += 1;
            len -= 1;
        }
        
        return 0;
    }    
}

pub fn copy_to_sm(dst: usize, src: usize, len: usize) -> i32 {

    unsafe {

        if src % REGBYTES == 0 && dst % REGBYTES == 0 {
            while len >= MPRV_BLOCK {
                let res: i32 = copy_block_to_sm(dst as *const mprv_block, src); // mprv.s
                if res != 0 {
                    return res;
                }
    
                src += MPRV_BLOCK;
                dst += MPRV_BLOCK;
                len -= MPRV_BLOCK;
            }
    
            while len >= REGBYTES {
                let res: i32 = copy_word_to_sm(dst as *const usize, src); // mprv.s
                if res != 0 {
                    return res;
                }
    
                src += REGBYTES;
                dst += REGBYTES;
                len -= REGBYTES;
            }
        }
    
        while len > 0 {
            let res: i32 = copy1_to_sm(dst as *const u8, src);
            if res != 0 {
                return res;
            }
    
            src += 1;
            dst += 1;
            len -= 1;
        }
    
        return 0;
    }
}