use volatile::Volatile;

#[repr(C, packed)]
pub struct HpetTimer {
    configuration_and_capability_register: u64,
    comparator_value_register: u64,
    fsb_interrupt_route_register: u64,
    _reserved: u64,
}

#[allow(unaligned_references)]
impl HpetTimer {
    pub fn configuration_and_capability_register(&self) -> u64 {
        Volatile::new_read_only(&self.configuration_and_capability_register).read()
    }

    pub fn set_configuration_and_capability_register(&mut self, value: u64) {
        Volatile::new(&mut self.configuration_and_capability_register).write(value)
    }

    pub fn comparator_value_register(&self) -> u64 {
        Volatile::new_read_only(&self.comparator_value_register).read()
    }

    pub fn set_comparator_value_register(&mut self, value: u64) {
        Volatile::new(&mut self.comparator_value_register).write(value)
    }

    pub fn fsb_interrupt_route_register(&self) -> u64 {
        Volatile::new_read_only(&self.fsb_interrupt_route_register).read()
    }

    pub fn set_fsb_interrupt_route_register(&mut self, value: u64) {
        Volatile::new(&mut self.fsb_interrupt_route_register).write(value)
    }
}
