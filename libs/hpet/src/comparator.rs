use crate::timer::HpetTimer;

/// A comparator is value that is compared against the counter register and if it matches, an interrupt is generated.
/// 
/// ## Periodic vs One-shot Mode
/// The comparator can run in either periodic or one-shot mode.
/// In one-shot mode, an interrupt is triggered once [`value`] is reached and nothing more.
/// In periodic mode, every time [`values`] is reached, it is increased by the amount last written to it.
/// Periodic mode is not implemented for every comparator. Use [`supports_periodic_mode`] to see it it's avilable.
/// 
/// ### Example: 
/// One-shot mode:
/// ```text
/// set_value(123)
///     ~~~ Interrupt Happens at 123 ~~~
/// value() == 123
/// ```
/// 
/// Periodic mode:
/// ```text
/// set_value(123)
///     ~~~ Interrupt Happens at 123 ~~~
/// value() == 246
///     ~~~ Interrupt Happens at 246 ~~~
/// value() == 369
/// ```
/// 
/// ## Comparator Size
/// A comparator can either be 32-bit or 64-bit.
/// In both cases, [`value`] will be 8 bytes long, but a 32-bit comparator only compares the lower 32 bits of [`value`] against the counter.
/// 
/// ## Interrupt Routing
/// The interrupts generated from a comparator can either be routed through an IOAPIC or using FSB Messaging.
/// FSB Messaging takes priority over I/O routing, but is also not required to be implemented, so check using [`supports_fsb_interrupt`].
/// For I/O routing, only the interrupts specified by [`supported_io_interrupt_routes`] can be used.
pub struct HpetComparator {
    timer: &'static mut HpetTimer,
    index: u8
}

impl HpetComparator {
    pub(crate) fn new(timer: &'static mut HpetTimer, index: u8) -> Self {
        HpetComparator { timer, index }
    }

    /// Gets the index of the comparator.
    pub fn index(&self) -> u8 {
        self.index
    }

    /// Returns whenever the interrupts are level-triggered or edge-triggered.
    /// TODO: Example with the differance
    pub fn is_level_triggered(&self) -> bool {
        (self.timer.configuration_and_capability_register() & 0b10) == 0b10
    }

    /// Makes the interrupts level-triggered.
    pub fn set_level_triggered(&mut self) {
        let value = self.timer.configuration_and_capability_register() | 0b10;
        self.timer.set_configuration_and_capability_register(value)
    }

    /// Makes the interrupts edge-triggered.
    pub fn set_edge_triggered(&mut self) {
        let value = self.timer.configuration_and_capability_register() & (!0b10);
        self.timer.set_configuration_and_capability_register(value)
    }

    /// Returns whenever the comparator is enabled.
    /// A disabled comparator doesn't generate interrupts.
    pub fn is_enabled(&self) -> bool {
        (self.timer.configuration_and_capability_register() & 0b100) == 0b100
    }

    /// Enables the comparator and starts generating interrupts.
    pub fn enable(&mut self) {
        let value = self.timer.configuration_and_capability_register() | 0b100;
        self.timer.set_configuration_and_capability_register(value)
    }

    /// Disables the comparator and stops generating interrupts.
    pub fn disable(&mut self) {
        let value = self.timer.configuration_and_capability_register() & (!0b100);
        self.timer.set_configuration_and_capability_register(value)
    }

    /// Returns whenever the comparator is in periodic mode or one-shot mode.
    pub fn is_periodic_mode(&self) -> bool {
        (self.timer.configuration_and_capability_register() & 0b1000) == 0b1000
    }

    /// Sets the comparator into periodic mode.
    /// Check [`supports_periodic_mode`] before using. It has no effect otherwise.
    pub fn set_periodic_mode(&mut self) {
        let value = self.timer.configuration_and_capability_register() | 0b1000;
        self.timer.set_configuration_and_capability_register(value)
    }

    /// Sets the comparator into one-shot mode.
    pub fn set_one_shot_mode(&mut self) {
        let value = self.timer.configuration_and_capability_register() & (!0b1000);
        self.timer.set_configuration_and_capability_register(value)
    }

    /// Returns whenever periodic mode is supported by this comparator.
    pub fn supports_periodic_mode(&self) -> bool {
        (self.timer.configuration_and_capability_register() & 0b10000) == 0b10000
    }

    /// Returns whenever this comparator supports 64-bit [`value`]s.
    /// If not supported. Only the lower 32-bits of [`value`] are compared against the couter register.
    pub fn is_64_bit(&self) -> bool {
        (self.timer.configuration_and_capability_register() & 0b100000) == 0b100000
    }

    /// Returns whenever this 64-bit comparator is in 32-bit mode.
    /// A 64-bit comparator in 32-bit mode acts like a 32-bit counter. See [`is_64_bit`].
    /// 
    /// ## Note
    /// A 32-bit comparator (`is_64_bit() == false`) returns false for this function,
    /// since it's not in 32-bit **mode**, but an actual 32-bit comparator.
    pub fn is_32_bit_mode(&self) -> bool {
        (self.timer.configuration_and_capability_register() & 0b100000000) == 0b100000000
    }

    /// Sets this 64-bit comparator in 32-bit mode. It has no effect on a 32-bit comparator.
    pub fn set_32_bit_mode(&mut self) {
        let value = self.timer.configuration_and_capability_register() | 0b100000000;
        self.timer.set_configuration_and_capability_register(value)
    }

    /// Deactivates the 32-bit mode of this 64-bit comparator. It has no effect on a 32-bit comparator.
    pub fn unset_32_bit_mode(&mut self) {
        let value = self.timer.configuration_and_capability_register() & (!0b100000000);
        self.timer.set_configuration_and_capability_register(value)
    }

    /// Gets which interrupt should be triggered at the IOAPIC.
    /// Can only be one of [`supported_io_interrupt_routes`].
    /// Will be ignored if [`is_fsb_interrupt_enabled`].
    pub fn io_interrupt_route(&self) -> u8 {
        ((self.timer.configuration_and_capability_register() >> 9) & 0b11111) as u8
    }

    /// Sets which interrupt should be triggered at the IOAPIC.
    /// Can only be one of [`supported_io_interrupt_routes`].
    /// Will be ignored if [`is_fsb_interrupt_enabled`].
    pub fn set_io_interrupt_route(&mut self, value: u8) {
        let value = self.timer.configuration_and_capability_register() | ((value as u64 & 0b11111) << 9);
        self.timer.set_configuration_and_capability_register(value)
    }

    /// Returns whenever FSB interrupt routing is enabled on this comparator.
    /// FSB interrupt routing overrides the standard I/O interrupt routing.
    /// Check [`supports_fsb_interrupt`] before using. It has no effect otherwise.
    pub fn is_fsb_interrupt_enabled(&self) -> bool {
        (self.timer.configuration_and_capability_register() & 0b100000000000000) == 0b100000000000000
    }

    /// Enables FSB interrupt routing for this comparator.
    /// FSB interrupt routing overrides the standard I/O interrupt routing.
    /// Check [`supports_fsb_interrupt`] before using. It has no effect otherwise.
    pub fn enable_fsb_interrupt(&mut self) {
        if self.supports_fsb_interrupt() {
            let value = self.timer.configuration_and_capability_register() | 0b100000000000000;
            self.timer.set_configuration_and_capability_register(value)
        }
    }

    /// Disables FSB interrupt routing for this comparator.
    /// FSB interrupt routing overrides the standard I/O interrupt routing.
    /// Check [`supports_fsb_interrupt`] before using. It has no effect otherwise.
    pub fn disable_fsb_interrupt(&mut self) {
        if self.supports_fsb_interrupt() {
            let value = self.timer.configuration_and_capability_register() & (!0b100000000);
            self.timer.set_configuration_and_capability_register(value)
        }
    }

    /// Returns whenever FSB interrupt routing is supported by this comparator.
    pub fn supports_fsb_interrupt(&self) -> bool {
        (self.timer.configuration_and_capability_register() & 0b1000000000000000) == 0b1000000000000000
    }

    /// Returns a bitmask of which interrupts of the IOAPIC this comparator can route to.
    pub fn supported_io_interrupt_routes(&self) -> u32 {
        (self.timer.configuration_and_capability_register() >> 32) as u32
    }

    /// Returns the current value of the comparator.
    /// The value at which an interrupt should triggered.
    pub fn value(&self) -> u64 {
        self.timer.comparator_value_register()
    }

    /// Sets the value of the comparator.
    /// The value at which an interrupt should triggered.
    pub fn set_value(&mut self, value: u64) {
        self.timer.set_comparator_value_register(value)
    }

    pub fn fsb_interrupt_address(&self) -> u32 {
        (self.timer.fsb_interrupt_route_register() >> 32) as u32
    }

    pub fn set_fsb_interrupt_address(&mut self, value: u32) {
        let value = (self.timer.fsb_interrupt_route_register() as u32) as u64 | ((value as u64) << 32);
        self.timer.set_fsb_interrupt_route_register(value)
    }

    pub fn fsb_interrupt_value(&self) -> u32 {
        self.timer.fsb_interrupt_route_register() as u32
    }

    pub fn set_fsb_interrupt_value(&mut self, value: u32) {
        let value = (self.timer.fsb_interrupt_route_register() & (!(u32::MAX as u64))) | value as u64;
        self.timer.set_fsb_interrupt_route_register(value)
    }
}

impl Drop for HpetComparator {
    fn drop(&mut self) {
        self.disable();
        self.set_value(u64::MAX)
    }
}
