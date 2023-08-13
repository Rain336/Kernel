use core::ptr;

#[repr(C, packed)]
pub struct HpetTimer {
    configuration_and_capability_register: u64,
    comparator_value_register: u64,
    fsb_interrupt_route_register: u64,
    _reserved: u64,
}

impl HpetTimer {
    pub fn configuration_and_capability_register(&self) -> u64 {
        unsafe { ptr::read_volatile(ptr::addr_of!(self.configuration_and_capability_register)) }
    }

    pub fn set_configuration_and_capability_register(&mut self, value: u64) {
        unsafe {
            ptr::write_volatile(
                ptr::addr_of_mut!(self.configuration_and_capability_register),
                value,
            )
        };
    }

    pub fn comparator_value_register(&self) -> u64 {
        unsafe { ptr::read_volatile(ptr::addr_of!(self.comparator_value_register)) }
    }

    pub fn set_comparator_value_register(&mut self, value: u64) {
        unsafe { ptr::write_volatile(ptr::addr_of_mut!(self.comparator_value_register), value) };
    }

    pub fn fsb_interrupt_route_register(&self) -> u64 {
        unsafe { ptr::read_volatile(ptr::addr_of!(self.fsb_interrupt_route_register)) }
    }

    pub fn set_fsb_interrupt_route_register(&mut self, value: u64) {
        unsafe { ptr::write_volatile(ptr::addr_of_mut!(self.fsb_interrupt_route_register), value) };
    }
}
