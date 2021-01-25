#[derive(Debug)]
pub enum ClockDirection {
    Rising,
    Falling,
}

#[derive(Debug)]
pub enum BitDirection {
    LsbFirst,
    MsbFirst,
}

#[derive(Debug)]
struct FullDataShiftOptions {
    read_clock_direction: ClockDirection,
    write_clock_direction: ClockDirection,
    bit_direction: BitDirection,
    write_tdi: bool,
    read_tdo: bool,
    write_tms: bool,
}

impl Default for FullDataShiftOptions {
    fn default() -> Self {
        FullDataShiftOptions {
            read_clock_direction: ClockDirection::Rising,
            write_clock_direction: ClockDirection::Rising,
            bit_direction: BitDirection::MsbFirst,
            write_tdi: false,
            read_tdo: false,
            write_tms: false,
        }
    }
}

impl Into<u8> for FullDataShiftOptions {
    fn into(self) -> u8 {
        let mut byte = 0;
        byte |= match self.write_clock_direction {
            ClockDirection::Rising => 0x00,
            ClockDirection::Falling => 0x01,
        };
        byte |= match self.read_clock_direction {
            ClockDirection::Rising => 0x00,
            ClockDirection::Falling => 0x04,
        };
        byte |= match self.bit_direction {
            BitDirection::MsbFirst => 0x00,
            BitDirection::LsbFirst => 0x08,
        };
        byte |= match self.write_tdi {
            false => 0x00,
            true => 0x10,
        };
        byte |= match self.read_tdo {
            false => 0x00,
            true => 0x20,
        };
        byte |= match self.write_tms {
            false => 0x00,
            true => 0x40,
        };

        byte
    }
}

#[derive(Debug)]
pub struct DataShiftOptions {
    pub clock_direction: ClockDirection,
    pub bit_direction: BitDirection,
}

impl Into<FullDataShiftOptions> for DataShiftOptions {
    fn into(self) -> FullDataShiftOptions {
        FullDataShiftOptions {
            read_clock_direction: self.clock_direction,
            bit_direction: self.bit_direction,
            ..Default::default()
        }
    }
}

impl Into<u8> for DataShiftOptions {
    fn into(self) -> u8 {
        Into::<FullDataShiftOptions>::into(self).into()
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PinRange {
    High,
    Low,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PinValue {
    High,
    Low,
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PinValueArray([PinValue; 8]);

impl Into<u8> for PinValueArray {
    fn into(self) -> u8 {
        self.0
            .iter()
            .enumerate()
            .map(|(i, val)| match val {
                PinValue::High => 1 << i,
                PinValue::Low => 0 << i,
            })
            .fold(0u8, |acc, val| acc | val)
    }
}

impl From<u8> for PinValueArray {
    fn from(value: u8) -> Self {
        let mut result = [PinValue::Low; 8];
        let value = value.reverse_bits();
        for i in 0..8 {
            match (value << i & 0x80) == 0x80 {
                true => result[i] = PinValue::High,
                false => result[i] = PinValue::Low,
            }
        }
        PinValueArray(result)
    }
}

#[cfg(test)]
mod pin_value_array_tests {
    use super::*;

    #[test]
    fn test() {
        let array: PinValueArray = 0b00110011u8.into();

        assert_eq!(
            array,
            PinValueArray([
                PinValue::High,
                PinValue::High,
                PinValue::Low,
                PinValue::Low,
                PinValue::High,
                PinValue::High,
                PinValue::Low,
                PinValue::Low,
            ])
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PinDirection {
    Input,
    Output,
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PinDirectionArray([PinDirection; 8]);

impl Into<u8> for PinDirectionArray {
    fn into(self) -> u8 {
        self.0
            .iter()
            .enumerate()
            .map(|(i, val)| match val {
                PinDirection::Input => 0 << i,
                PinDirection::Output => 1 << i,
            })
            .fold(0u8, |acc, val| acc | val)
    }
}

impl From<u8> for PinDirectionArray {
    fn from(value: u8) -> Self {
        let mut result = [PinDirection::Input; 8];
        let value = value.reverse_bits();
        for i in 0..8 {
            match (value << i & 0x80) == 0x80 {
                true => result[i] = PinDirection::Output,
                false => result[i] = PinDirection::Input,
            }
        }
        PinDirectionArray(result)
    }
}

#[cfg(test)]
mod pin_direction_array_tests {
    use super::*;

    #[test]
    fn test() {
        let array: PinDirectionArray = 0b00110011u8.into();

        assert_eq!(
            array,
            PinDirectionArray([
                PinDirection::Output,
                PinDirection::Output,
                PinDirection::Input,
                PinDirection::Input,
                PinDirection::Output,
                PinDirection::Output,
                PinDirection::Input,
                PinDirection::Input,
            ])
        )
    }
}

#[derive(Debug)]
pub enum Command {
    ReadDataShiftBits {
        options: DataShiftOptions,
        length: u8,
    },
    ReadDataShiftBytes {
        options: DataShiftOptions,
        length: u16,
    },
    WriteDataShiftBits {
        options: DataShiftOptions,
        bits: u8,
        length: u8,
    },
    WriteDataShiftBytes {
        options: DataShiftOptions,
        bytes: Vec<u8>,
    },
    SetBits {
        range: PinRange,
        value: PinValueArray,
        direction: PinDirectionArray,
    },
    ReadBits {
        range: PinRange,
    },
    SetLoopback {
        enable: bool,
    },
    SetClockDivisor {
        divisor: u16,
    },
    WaitForIo {
        value: PinValue,
    },
}

impl Command {
    pub fn expected_response_length(&self) -> usize {
        match self {
            Self::ReadDataShiftBits {
                options: _,
                length: _,
            } => 1,
            Self::WriteDataShiftBits {
                options: _,
                bits: _,
                length: _,
            } => 0,
            Self::WriteDataShiftBytes {
                options: _,
                bytes: _,
            } => 0,
            Self::ReadDataShiftBytes { options: _, length } => length.to_owned() as usize,
            Self::SetBits {
                range: _,
                value: _,
                direction: _,
            } => 0,
            Self::ReadBits { range: _ } => 1,
            Self::SetLoopback { enable: _ } => 0,
            Self::SetClockDivisor { divisor: _ } => 0,
            Self::WaitForIo { value: _ } => 1,
        }
    }
}

impl Into<Vec<u8>> for Command {
    fn into(self) -> Vec<u8> {
        match self {
            Self::WriteDataShiftBits {
                options,
                bits,
                length,
            } => {
                let full_options = FullDataShiftOptions {
                    write_clock_direction: options.clock_direction,
                    bit_direction: options.bit_direction,
                    write_tdi: true,
                    ..Default::default()
                };
                let opcode: u8 = full_options.into();

                vec![opcode | 0x02, length - 1, bits]
            }
            Self::ReadDataShiftBits { options, length } => {
                let full_options = FullDataShiftOptions {
                    write_clock_direction: options.clock_direction,
                    bit_direction: options.bit_direction,
                    read_tdo: true,
                    ..Default::default()
                };
                let opcode: u8 = full_options.into();

                vec![opcode | 0x02, length - 1]
            }
            Self::WriteDataShiftBytes { options, bytes } => {
                let full_options = FullDataShiftOptions {
                    write_clock_direction: options.clock_direction,
                    bit_direction: options.bit_direction,
                    write_tdi: true,
                    ..Default::default()
                };
                let opcode: u8 = full_options.into();

                let mut result = vec![opcode];
                result.extend_from_slice(&((bytes.len() - 1) as u16).to_le_bytes());
                result.extend(bytes);

                result
            }
            Self::ReadDataShiftBytes { options, length } => {
                let full_options = FullDataShiftOptions {
                    write_clock_direction: options.clock_direction,
                    bit_direction: options.bit_direction,
                    read_tdo: true,
                    ..Default::default()
                };
                let opcode: u8 = full_options.into();

                let mut result = vec![opcode];
                result.extend_from_slice(&(length - 1).to_le_bytes());

                result
            }
            Self::SetBits {
                range,
                value,
                direction,
            } => {
                let opcode = match range {
                    PinRange::Low => 0x80,
                    PinRange::High => 0x82,
                };

                vec![opcode, value.into(), direction.into()]
            }
            Self::ReadBits { range } => {
                let opcode = match range {
                    PinRange::Low => 0x81,
                    PinRange::High => 0x83,
                };

                vec![opcode]
            }
            Self::SetLoopback { enable } => {
                let opcode = match enable {
                    true => 0x84,
                    false => 0x85,
                };

                vec![opcode]
            }
            Command::SetClockDivisor { divisor } => {
                let mut result = vec![0x86];
                result.extend_from_slice(&divisor.to_le_bytes());
                result
            }
            Self::WaitForIo { value } => match value {
                PinValue::High => vec![0x88],
                PinValue::Low => vec![0x89],
            },
        }
    }
}

impl IntoIterator for Command {
    type Item = u8;

    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        let bytes: Vec<u8> = self.into();

        bytes.into_iter()
    }
}

#[derive(Debug)]
pub struct CommandList(pub Vec<Command>);

impl IntoIterator for CommandList {
    type Item = u8;

    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        let mut result = Vec::new();
        for command in self.0 {
            result.extend(command)
        }

        result.into_iter()
    }
}

impl Into<Vec<u8>> for CommandList {
    fn into(self) -> Vec<u8> {
        self.into_iter().collect()
    }
}

impl CommandList {
    pub fn expected_response_length(self) -> usize {
        self.0
            .iter()
            .map(|cmd| cmd.expected_response_length())
            .fold(0, |acc, cur| acc + cur)
    }
}
