use memory::page_table::LockedPageTable;

// #[cfg(target_arch = "x86_64")]
// pub fn get_root_table() -> &'static LockedPageTable {
//     use x86_64::registers::control::Cr3;

//     let (table, _) = Cr3::read();
// }
