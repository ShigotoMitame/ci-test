//! # MPSSE-rs
//! A builder for [FTDI's MPSSE commands](https://www.ftdichip.com/Support/Documents/AppNotes/AN_108_Command_Processor_for_MPSSE_and_MCU_Host_Bus_Emulation_Modes.pdf).
//!
//! ```
//! use mpsse::{Builder, ClockDirection};
//!
//! pub fn main() {
//!     let commands = Builder::new()
//!         .set_frequency(100_000.0)
//!         .then()
//!         .read_data(15)
//!         .with_clock_direction(ClockDirection::Rising)
//!         .build();
//!
//!
//!     assert_eq!(
//!         vec![0x86, 0x3B, 0x00, 0x20, 0x0E, 0x00],
//!         commands
//!     );
//! }
//! ```

pub mod builder;
pub mod command;

pub use command::{
    BitDirection, ClockDirection, PinDirection, PinDirectionArray, PinRange, PinValue,
    PinValueArray,
};

pub use builder::Builder;
