use volatile::Volatile;

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

#[allow(unaligned_references)]
impl HpetRegisters {
    pub fn general_capabilitis_and_id_register(&self) -> u64 {
        Volatile::new_read_only(&self.general_capabilitis_and_id_register).read()
    }

    pub fn general_configuration_register(&self) -> u64 {
        Volatile::new_read_only(&self.general_configuration_register).read()
    }

    pub fn set_general_configuration_register(&mut self, value: u64) {
        Volatile::new(&mut self.general_configuration_register).write(value)
    }

    pub fn main_counter_value_register(&self) -> u64 {
        Volatile::new_read_only(&self.main_counter_value_register).read()
    }

    pub fn set_main_counter_value_register(&mut self, value: u64) {
        Volatile::new(&mut self.main_counter_value_register).write(value)
    }
}
