use range_check::{Check, OutOfRangeError};
use x86_64::instructions::interrupts::without_interrupts;

use crate::{COMMAND_PORT, DATA_PORT};

// TODO: Use the proper names for these registers
const OSDEV_REGISTER_A: u8 = 0x8A;
const OSDEV_REGISTER_B: u8 = 0x8B;
const OSDEV_REGISTER_C: u8 = 0x0C;

pub fn enable() {
    // Based on https://wiki.osdev.org/RTC#Turning_on_IRQ_8
    without_interrupts(|| {
        // select register B, and disable NMI
        unsafe { COMMAND_PORT.write(OSDEV_REGISTER_B) };
        // read the current value of register B
        let previous_value_of_register_b = unsafe { DATA_PORT.read() };
        // set the index again (a read will reset the index to register D)
        unsafe { COMMAND_PORT.write(OSDEV_REGISTER_B) };
        // write the previous value ORed with 0x40. This turns on bit 6 of register B
        let new_value_of_register_b = previous_value_of_register_b | 0x40;
        unsafe { DATA_PORT.write(new_value_of_register_b) };
    })
}

pub fn disable() {
    // Based on https://wiki.osdev.org/RTC#Turning_on_IRQ_8
    without_interrupts(|| {
        // select register B, and disable NMI
        unsafe { COMMAND_PORT.write(OSDEV_REGISTER_B) };
        // read the current value of register B
        let previous_value_of_register_b = unsafe { DATA_PORT.read() };
        // set the index again (a read will reset the index to register D)
        unsafe { COMMAND_PORT.write(OSDEV_REGISTER_B) };
        // This turn off bit 6 of register B
        let new_value_of_register_b = previous_value_of_register_b & 0b10111111;
        unsafe { DATA_PORT.write(new_value_of_register_b) };
    })
}

/// Useful for throwing away contents of Register C to receive more interrupts, as described in https://wiki.osdev.org/RTC#Interrupts_and_Register_C
pub fn read_register_c() -> u8 {
    unsafe { COMMAND_PORT.write(OSDEV_REGISTER_C) };
    unsafe { DATA_PORT.read() }
}

pub struct DividerValue(u8);

impl DividerValue {
    pub fn new(divider_value: u8) -> Result<Self, OutOfRangeError<u8>> {
        divider_value
            .check_range({
                // rate must be above 2 and not over 15
                3..16
            })
            .map(|checked_divider_value| Self(checked_divider_value))
    }
}

/// https://wiki.osdev.org/RTC#Changing_Interrupt_Rate
pub fn set_divider_value(divider_value: DividerValue) {
    without_interrupts(|| {
        // set index to register A, disable NMI
        unsafe { COMMAND_PORT.write(OSDEV_REGISTER_A) };
        // get initial value of register A
        let previous_register_a_value = unsafe { DATA_PORT.read() };
        // reset index to A
        unsafe { COMMAND_PORT.write(OSDEV_REGISTER_A) };
        //write only our rate to A. Note, rate is the bottom 4 bits.
        let rate = divider_value.0;
        let new_register_a_value = (previous_register_a_value & 0xF0) | rate;
        unsafe { DATA_PORT.write(new_register_a_value) };
    })
}
