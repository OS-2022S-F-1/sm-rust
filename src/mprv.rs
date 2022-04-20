fn copy_from_sm<T>(dst: uintptr_t, src_buf: *mut T, len: size_t) -> i32 {
    let src: uintptr_t = src_buf as uintptr_t;

    if src % REGBYTES  == 0 && dst % REGBYTES == 0 {
        while len >= MPRV_BLOCK {
            let res: i32 = copy_block_from_sm(dst, src as *mut mprv_block); // mprv.s
            if res {
                return res;
            }

            src += MPRV_BLOCK;
            dst += MPRV_BLOCK;
            len -= MPRV_BLOCK;
        }

        while len >= REGBYTES {
            let res: i32 = copy_word_from_sm(dst, src as *mut uintptr_t); // mprv.s
            if res {
                return res;
            }

            src += REGBYTES;
            dst += REGBYTES;
            len -= REGBYTES;
        }
    }

    while len > 0 {
        let res: i32 = copy1_from_sm(dst, src as *mut uint8_t); // mprv.s
        if res {
            return res;
        }

        src += 1;
        dst += 1;
        len -= 1;
    }
    
    return 0;
}

fn copy_to_sm<T>(dst_buf: *mut T, src: uintptr_t, len: size_t) -> i32 {
    let dst: uintptr_t = dst_buf as uintptr_t;

    if src % REGBYTES == 0 && dst % REGBYTES == 0 {
        while len >= MPRV_BLOCK {
            let res: i32 = copy_block_to_sm(dst as *mut mprv_block, src); // mprv.s
            if res {
                return res;
            }

            src += MPRV_BLOCK;
            dst += MPRV_BLOCK;
            len -= MPRV_BLOCK;
        }

        while len >= REGBYTES {
            let res: i32 = copy_word_to_sm(dst as *mut uintptr_t, src); // mprv.s
            if res {
                return res;
            }

            src += REGBYTES;
            dst += REGBYTES;
            len -= REGBYTES;
        }
    }

    while len > 0 {
        let res: i32 = copy1_to_sm(dst as *mut uint8_t, src);
        if res {
            return res;
        }

        src += 1;
        dst += 1;
        len -= 1;
    }

    return 0;
}