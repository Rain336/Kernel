use log::debug;
use common::sync::SyncOnceCell;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{Segment, CS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

const SECOND_STACK_SIZE: usize = 4096 * 4;
static SECOND_STACK: [u8; SECOND_STACK_SIZE] = [0; SECOND_STACK_SIZE];

static TASK_STATE_SEGMENT: SyncOnceCell<TaskStateSegment> = SyncOnceCell::new();

static GLOBAL_DISCRIPTOR_TABLE: SyncOnceCell<GlobalDescriptorTable> = SyncOnceCell::new();

static KERNEL_CODE_SEGMENT: SegmentSelector = SegmentSelector::new(1, PrivilegeLevel::Ring0);
//static KERNEL_DATA_SEGMENT: SegmentSelector = SegmentSelector::new(2, PrivilegeLevel::Ring0);
//static USER_CODE_SEGMENT: SegmentSelector = SegmentSelector::new(3, PrivilegeLevel::Ring3);
//static USER_DATA_SEGMENT: SegmentSelector = SegmentSelector::new(4, PrivilegeLevel::Ring3);
static TSS_SEGMENT: SegmentSelector = SegmentSelector::new(5, PrivilegeLevel::Ring0);

pub fn load(stack_end: u64) {
    let tss = TASK_STATE_SEGMENT.get_or_init(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[0] = VirtAddr::new_truncate(SECOND_STACK.as_ptr() as u64 + SECOND_STACK_SIZE as u64);
        tss.privilege_stack_table[0] = VirtAddr::new_truncate(stack_end);
        tss
    });

    let gdt = GLOBAL_DISCRIPTOR_TABLE.get_or_init(|| {
        let mut gdt = GlobalDescriptorTable::new();
        gdt.add_entry(Descriptor::kernel_code_segment());
        gdt.add_entry(Descriptor::kernel_data_segment());
        gdt.add_entry(Descriptor::user_code_segment());
        gdt.add_entry(Descriptor::user_data_segment());
        gdt.add_entry(Descriptor::tss_segment(tss));
        gdt
    });

    debug!("Loading Global Descriptor Table...");
    gdt.load();
    unsafe {
        CS::set_reg(KERNEL_CODE_SEGMENT);
        load_tss(TSS_SEGMENT);
    };
}
