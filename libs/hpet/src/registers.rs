use core::ptr;

#[repr(C, packed)]
pub struct HpetRegisters {
    general_capabilitis_and_id_register: u64,
    _reserved1: u64,
    general_configuration_register: u64,
    _reserved2: u64,
    general_input_status_register: u64,
    _reserved3: [u8; 200],
    main_counter_value_register: u64,
    _reserved4: u64,
}

impl HpetRegisters {
    pub fn general_capabilitis_and_id_register(&self) -> u64 {
        unsafe { ptr::read_volatile(ptr::addr_of!(self.general_capabilitis_and_id_register)) }
    }

    pub fn general_configuration_register(&self) -> u64 {
        unsafe { ptr::read_volatile(ptr::addr_of!(self.general_configuration_register)) }
    }

    pub fn set_general_configuration_register(&mut self, value: u64) {
        unsafe {
            ptr::write_volatile(
                ptr::addr_of_mut!(self.general_configuration_register),
                value,
            )
        };
    }

    pub fn main_counter_value_register(&self) -> u64 {
        unsafe { ptr::read_volatile(ptr::addr_of!(self.main_counter_value_register)) }
    }

    pub fn set_main_counter_value_register(&mut self, value: u64) {
        unsafe { ptr::write_volatile(ptr::addr_of_mut!(self.main_counter_value_register), value) };
    }
}
