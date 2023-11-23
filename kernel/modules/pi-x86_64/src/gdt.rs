// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use common::sync::SyncOnceCell;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{Segment, CS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{PrivilegeLevel, VirtAddr};

static TASK_STATE_SEGMENT: SyncOnceCell<TaskStateSegment> = SyncOnceCell::new();

static GLOBAL_DISCRIPTOR_TABLE: SyncOnceCell<GlobalDescriptorTable> = SyncOnceCell::new();

static KERNEL_CODE_SEGMENT: SegmentSelector = SegmentSelector::new(1, PrivilegeLevel::Ring0);
static KERNEL_DATA_SEGMENT: SegmentSelector = SegmentSelector::new(2, PrivilegeLevel::Ring0);
static USER_CODE_SEGMENT: SegmentSelector = SegmentSelector::new(3, PrivilegeLevel::Ring3);
static USER_DATA_SEGMENT: SegmentSelector = SegmentSelector::new(4, PrivilegeLevel::Ring3);
static TSS_SEGMENT: SegmentSelector = SegmentSelector::new(5, PrivilegeLevel::Ring0);

pub fn load(primary_stack: u64, secondary_stack: u64) {
    let tss = TASK_STATE_SEGMENT.get_or_init(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[0] = VirtAddr::new_truncate(secondary_stack);
        tss.privilege_stack_table[0] = VirtAddr::new_truncate(primary_stack);
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

    log::info!("Loading Global Descriptor Table...");
    gdt.load();
    unsafe {
        CS::set_reg(KERNEL_CODE_SEGMENT);
        load_tss(TSS_SEGMENT);
    };
}
