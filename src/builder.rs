/// Simple buidlers for MPSSE commands
use crate::command::{Command, CommandList, DataShiftOptions};

pub use crate::command::{
    BitDirection, ClockDirection, PinDirection, PinDirectionArray, PinRange, PinValue,
    PinValueArray,
};

/// Internal macro for repetitive builder methods
macro_rules! builder_funcs {
    () => {
        /// Same as commit, but more readable in a chain.
        pub fn then(self) -> Builder {
            self.commit()
        }

        /// Commit this command to the parent Builder, then get the entire command list as a byte sequence.
        pub fn build(self) -> Vec<u8> {
            self.commit().build()
        }
    };
}

/// Builder for MPSSE commands.
#[derive(Debug)]
pub struct Builder {
    commands: Vec<Command>,
}

impl Builder {
    /// Create a new command builder with.
    pub fn new() -> Self {
        Builder {
            commands: Vec::new(),
        }
    }

    /// Write bytes of data, one bit at a time, on a single pin.
    ///
    /// This will generate a Data Shifting Command with the appropriate bits set to
    /// write to TDO with the appropriate parameters
    ///
    /// * `data` - The data to write out.
    ///
    /// ```
    /// use mpsse::{Builder, ClockDirection, BitDirection};
    ///
    /// let commands = Builder::new()
    ///     .write_data(vec![0xD, 0xEC, 0xAF])
    ///     .with_clock_direction(ClockDirection::Rising)
    ///     .with_bit_direction(BitDirection::MsbFirst)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x10, 0x02, 0x00, 0xD, 0xEC, 0xAF])
    /// ```
    pub fn write_data(self, data: Vec<u8>) -> WriteBuilder {
        WriteBuilder {
            parent: self,
            data,
            clock_direction: ClockDirection::Rising,
            bit_direction: BitDirection::MsbFirst,
        }
    }

    /// Read bytes of data, one bit at a time, on a single pin.
    ///
    /// This will generate a Data Shifting Command with the appropriate bits set to
    /// read from TDI with the appropriate parameters
    ///
    /// * `length` - The number of bytes to read out.
    ///
    /// ```
    /// use mpsse::{Builder, ClockDirection, BitDirection};
    ///
    /// let commands = Builder::new()
    ///     .read_data(328)
    ///     .with_clock_direction(ClockDirection::Rising)
    ///     .with_bit_direction(BitDirection::MsbFirst)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x20, 0x47, 0x01])
    /// ```
    pub fn read_data(self, length: u16) -> ReadBuilder {
        ReadBuilder {
            parent: self,
            length,
            clock_direction: ClockDirection::Rising,
            bit_direction: BitDirection::MsbFirst,
        }
    }

    /// Set the pins of the interface directly to the given values, and configure their direction.
    ///
    /// This will generate a Set Data Bits command of the appropriate type
    ///
    /// * `range` - Set either the high or low byte to be set
    /// * `direction` - The direction to set the pins, as either inputs or outputs.
    /// * `value` - The value to set the pins to, high or low (only applies to output pins)
    ///
    /// ```
    /// use mpsse::{Builder, PinRange};
    ///
    /// let commands = Builder::new()
    ///     .set_pins(PinRange::Low, 0b00110100, 0b11001100)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x80, 0xCC, 0x34])
    /// ```
    pub fn set_pins<D, V>(self, range: PinRange, direction: D, value: V) -> SetPinsBuilder
    where
        D: Into<PinDirectionArray>,
        V: Into<PinValueArray>,
    {
        SetPinsBuilder {
            parent: self,
            range,
            direction: direction.into(),
            value: value.into(),
        }
    }

    /// Read the value of input pins of the interface directly.
    ///
    /// This will generate a Read Data Bits command of the appropriate type
    ///
    /// * `range` - Read either the high or low byte to be set
    ///
    /// ```
    /// use mpsse::{Builder, PinRange};
    ///
    /// let commands = Builder::new()
    ///     .read_pins(PinRange::High)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x83])
    /// ```
    pub fn read_pins(self, range: PinRange) -> ReadPinsBuilder {
        ReadPinsBuilder {
            parent: self,
            range,
        }
    }

    /// Set the clock frequency of the interface.
    ///
    /// This will calculate the closest clock divisor to acheive the given frequency and generate a
    /// Set Clock Divisor command
    ///
    /// * `frequency` - The *target* frequency to set the clock to in hz. *Note*: this is a target
    ///     frequency that may not be met due to MPSSE internals. If you need more definite control
    ///     over the clock speed, use `.set_divisor()` instead.
    ///
    /// ```
    /// use mpsse::Builder;
    ///
    /// let commands = Builder::new()
    ///     .set_frequency(1_000_000.0)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x86, 0x05, 0x00])
    /// ```
    pub fn set_frequency<F>(self, frequency: F) -> SetFrequencyBuilder
    where
        F: Into<f64>,
    {
        SetFrequencyBuilder {
            parent: self,
            frequency: frequency.into(),
        }
    }

    pub fn set_divisor(self, _divisor: u16) -> ! {
        todo!()
    }

    /// Wait for IO on pin 1.
    ///
    /// This will send a Set Clock Frequency command
    ///
    /// * `value` - Whether to wait for a High or Low state on the pin.
    ///
    /// ```
    /// use mpsse::{Builder, PinValue};
    ///
    /// let commands = Builder::new()
    ///     .wait_for_io(PinValue::High)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x88])
    /// ```
    pub fn wait_for_io(self, value: PinValue) -> WaitForIoBuilder {
        WaitForIoBuilder {
            parent: self,
            value,
        }
    }

    /// Build the current command list into a sequence of bytes.
    pub fn build(self) -> Vec<u8> {
        CommandList(self.commands).into()
    }
}

/// Build a Data Shifting Command set to read bytes.
#[derive(Debug)]
pub struct ReadBuilder {
    parent: Builder,
    length: u16,
    clock_direction: ClockDirection,
    bit_direction: BitDirection,
}

impl ReadBuilder {
    /// Set this command to read the bits on a specific edge of the clock.
    ///
    /// By default, the ReadBuilder will build the command with the clock direction set Rising (meaning read on the rising clock).
    ///
    /// ```
    /// use mpsse::{Builder, ClockDirection};
    ///
    /// let commands = Builder::new()
    ///     .read_data(1)
    ///     .with_clock_direction(ClockDirection::Rising)
    ///     .then()
    ///     .read_data(1)
    ///     .with_clock_direction(ClockDirection::Falling)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x20, 0x00, 0x00, 0x21, 0x00, 0x00])
    /// ```
    pub fn with_clock_direction(self, direction: ClockDirection) -> Self {
        ReadBuilder {
            clock_direction: direction,
            ..self
        }
    }

    /// Set this command to read the bits in a specific direction
    ///
    /// By default, the ReadBuilder will build the command with the bit direction set MsbFirst.
    ///
    /// ```
    /// use mpsse::{Builder, BitDirection};
    ///
    /// let commands = Builder::new()
    ///     .read_data(1)
    ///     .with_bit_direction(BitDirection::MsbFirst)
    ///     .then()
    ///     .read_data(1)
    ///     .with_bit_direction(BitDirection::LsbFirst)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x20, 0x00, 0x00, 0x28, 0x00, 0x00])
    /// ```
    pub fn with_bit_direction(self, direction: BitDirection) -> Self {
        ReadBuilder {
            bit_direction: direction,
            ..self
        }
    }

    /// Commit this command to the parent Builder.
    fn commit(mut self) -> Builder {
        self.parent.commands.push(Command::ReadDataShiftBytes {
            options: DataShiftOptions {
                clock_direction: self.clock_direction,
                bit_direction: self.bit_direction,
            },
            length: self.length,
        });

        self.parent
    }

    builder_funcs!();
}

/// Build a Data Shifting Command set to write bytes.
#[derive(Debug)]
pub struct WriteBuilder {
    parent: Builder,
    data: Vec<u8>,
    clock_direction: ClockDirection,
    bit_direction: BitDirection,
}

impl WriteBuilder {
    /// Set this command to write the bits on a specific clock edge.
    ///
    /// By default, the WriteBuilder will build the command with the clock direction set Rising (meaning read on the rising clock).
    ///
    /// ```
    /// use mpsse::{Builder, ClockDirection};
    ///
    /// let commands = Builder::new()
    ///     .write_data(vec![0x01])
    ///     .with_clock_direction(ClockDirection::Rising)
    ///     .then()
    ///     .write_data(vec![0x01])
    ///     .with_clock_direction(ClockDirection::Falling)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x10, 0x00, 0x00, 0x01, 0x11, 0x00, 0x00, 0x01])
    /// ```
    pub fn with_clock_direction(self, direction: ClockDirection) -> Self {
        WriteBuilder {
            clock_direction: direction,
            ..self
        }
    }

    /// Set this command to write the bits in a specific direction
    ///
    /// By default, the WriteBuilder will build the command with the bit direction set MsbFirst.
    ///
    /// ```
    /// use mpsse::{Builder, BitDirection};
    ///
    /// let commands = Builder::new()
    ///     .write_data(vec![0x01])
    ///     .with_bit_direction(BitDirection::MsbFirst)
    ///     .then()
    ///     .write_data(vec![0x01])
    ///     .with_bit_direction(BitDirection::LsbFirst)
    ///     .build();
    ///
    /// assert_eq!(commands, vec![0x10, 0x00, 0x00, 0x01, 0x18, 0x00, 0x00, 0x01])
    /// ```
    pub fn with_bit_direction(self, direction: BitDirection) -> Self {
        WriteBuilder {
            bit_direction: direction,
            ..self
        }
    }

    /// Commit this command to the parent Builder.
    fn commit(mut self) -> Builder {
        self.parent.commands.push(Command::WriteDataShiftBytes {
            options: DataShiftOptions {
                clock_direction: self.clock_direction,
                bit_direction: self.bit_direction,
            },
            bytes: self.data,
        });

        self.parent
    }

    builder_funcs!();
}


/// Build a Set Pins command
#[derive(Debug)]
pub struct SetPinsBuilder {
    parent: Builder,
    range: PinRange,
    direction: PinDirectionArray,
    value: PinValueArray,
}

impl SetPinsBuilder {
    /// Commit this command to the parent Builder.
    fn commit(mut self) -> Builder {
        self.parent.commands.push(Command::SetBits {
            range: self.range,
            value: self.value,
            direction: self.direction,
        });

        self.parent
    }

    builder_funcs!();
}

/// Build a Set Divisor command using the given frequency.
#[derive(Debug)]
pub struct SetFrequencyBuilder {
    parent: Builder,
    frequency: f64,
}

impl SetFrequencyBuilder {
    /// Commit this command to the parent Builder.
    fn commit(mut self) -> Builder {
        self.parent.commands.push(Command::SetClockDivisor {
            divisor: (6_000_000f64 / self.frequency - 0.5).floor() as u16,
        });

        self.parent
    }

    builder_funcs!();
}

#[derive(Debug)]
pub struct WaitForIoBuilder {
    parent: Builder,
    value: PinValue,
}

impl WaitForIoBuilder {
    /// Commit this command to the parent Builder.
    fn commit(mut self) -> Builder {
        self.parent
            .commands
            .push(Command::WaitForIo { value: self.value });

        self.parent
    }

    builder_funcs!();
}

#[derive(Debug)]
pub struct ReadPinsBuilder {
    parent: Builder,
    range: PinRange,
}

impl ReadPinsBuilder {
    /// Commit this command to the parent Builder.
    fn commit(mut self) -> Builder {
        self.parent
            .commands
            .push(Command::ReadBits { range: self.range });

        self.parent
    }

    builder_funcs!();
}

#[cfg(test)]
mod write_builder_tests {
    use super::*;

    #[test]
    fn syntax_test() {
        let data = vec![0x10, 0x01, 0x20, 0x01];

        let commands = Builder::new()
            .write_data(data)
            .with_clock_direction(ClockDirection::Rising)
            .with_bit_direction(BitDirection::MsbFirst)
            .build();

        let command_bytes: Vec<u8> = commands.into_iter().collect();

        assert_eq!(
            command_bytes,
            vec![0x10, 0x03, 0x00, 0x10, 0x01, 0x20, 0x01]
        );
    }
}

#[cfg(test)]
mod read_builder_tests {
    use super::*;

    #[test]
    fn syntax_test() {
        let commands = Builder::new()
            .read_data(15)
            .with_clock_direction(ClockDirection::Rising)
            .with_bit_direction(BitDirection::MsbFirst)
            .build();

        let command_bytes: Vec<u8> = commands.into_iter().collect();

        assert_eq!(command_bytes, vec![0x20, 0x0e, 0x00]);
    }
}

#[cfg(test)]
mod set_freq_tests {
    use super::*;

    #[test]
    fn syntax_test() {
        let commands = Builder::new().set_frequency(5000.0).build();

        let command_bytes: Vec<u8> = commands.into_iter().collect();

        assert_eq!(command_bytes, vec![0x86, 0xAF, 0x04]);
    }
}
