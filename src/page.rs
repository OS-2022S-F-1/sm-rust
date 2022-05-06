use crate::opensbi;

pub const RISCV_PGSIZE: usize =  opensbi::PAGE_SIZE; //opensbi
pub const RISCV_PGSHIFT: usize =  opensbi::PAGE_SHIFT;


/* page table entry (PTE) fields */
pub const PTE_V: usize = 0x001; /* Valid */
pub const PTE_R: usize = 0x002; /* Read */
pub const PTE_W: usize = 0x004; /* Write */
pub const PTE_X: usize = 0x008; /* Execute */
pub const PTE_U: usize = 0x010; /* User */
pub const PTE_G: usize = 0x020; /* Global */
pub const PTE_A: usize = 0x040; /* Accessed */
pub const PTE_D: usize = 0x080; /* Dirty */
pub const PTE_SOFT: usize = 0x300; /* Reserved for Software */


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

