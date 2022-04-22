pub const RISCV_PGSIZE: usize =  PAGE_SIZE; //opensbi
pub const RISCV_PGSHIFT: usize =  PAGE_SHIFT;

/*
/* page table entry (PTE) fields */
#define PTE_V                _UL(0x001) /* Valid */
#define PTE_R                _UL(0x002) /* Read */
#define PTE_W                _UL(0x004) /* Write */
#define PTE_X                _UL(0x008) /* Execute */
#define PTE_U                _UL(0x010) /* User */
#define PTE_G                _UL(0x020) /* Global */
#define PTE_A                _UL(0x040) /* Accessed */
#define PTE_D                _UL(0x080) /* Dirty */
#define PTE_SOFT            _UL(0x300) /* Reserved for Software */
*/

#[cfg(target_pointer_width = "32")]
pub mod page {
    pub const RISCV_PGLEVEL_MASK: usize = 0x3ff;
    pub const RISCV_PGTABLE_HIGHEST_BIT: usize = 0x300;
    pub const RISCV_PGLEVEL_BITS: usize = 10;
}

#[cfg(target_pointer_width = "64")]
pub mod page {
    pub const RISCV_PGLEVEL_MASK: usize = 0x1ff;
    pub const RISCV_PGTABLE_HIGHEST_BIT: usize = 0x100;
    pub const RISCV_PGLEVEL_BITS: usize = 9;
}

pub const PTE_PPN_SHIFT: usize = 10;

pub const VA_BITS: usize = 39;
pub const RISCV_PGLEVEL_TOP: usize = (VA_BITS - RISCV_PGSHIFT) / page::RISCV_PGLEVEL_BITS;

