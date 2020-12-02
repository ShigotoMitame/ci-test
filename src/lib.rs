mod command;

use self::command::{Command, ClockDirection, BitDirection, DataShiftOptions, CommandList};

#[derive(Debug)]
pub struct Builder {
    commands: Vec<Command>
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            commands: vec!()
        }
    }

    pub fn write_data(self, data: Vec<u8>) -> WriteBuilder {
        WriteBuilder {
            parent: self,
            data,
            clock_direction: ClockDirection::Rising,
            bit_direction: BitDirection::MsbFirst,
        }
    }

    pub fn read_data(self, length: u16) -> ReadBuilder {
        ReadBuilder {
            parent: self,
            length,
            clock_direction: ClockDirection::Rising,
            bit_direction: BitDirection::MsbFirst,
        }
    }
        
    pub fn build(self) -> CommandList {
        CommandList(self.commands)
    }
}

#[derive(Debug)]
pub struct ReadBuilder {
    parent: Builder,
    length: u16,
    clock_direction: ClockDirection,
    bit_direction: BitDirection,
}

impl ReadBuilder {
    pub fn with_clock_direction(self, direction: ClockDirection) -> Self {
        ReadBuilder {
            clock_direction: direction,
            ..self
        }
    }

    pub fn with_bit_direction(self, direction: BitDirection) -> Self {
        ReadBuilder {
            bit_direction: direction,
            ..self
        }
    }

    fn commit(mut self) -> Builder {
        self.parent.commands.push(
            Command::ReadDataShiftBytes {
                options: DataShiftOptions {
                    clock_direction: self.clock_direction,
                    bit_direction: self.bit_direction,
                },
                length: self.length,
            }
        );

        self.parent
    }

    pub fn then(self) -> Builder { 
        self.commit()
    }

    pub fn build(self) -> CommandList {
        self.commit().build()
    }
}

#[derive(Debug)]
pub struct WriteBuilder {
    parent: Builder,
    data: Vec<u8>,
    clock_direction: ClockDirection,
    bit_direction: BitDirection,
}

impl WriteBuilder {
    pub fn with_clock_direction(self, direction: ClockDirection) -> Self {
        WriteBuilder {
            clock_direction: direction,
            ..self
        }
    }

    pub fn with_bit_direction(self, direction: BitDirection) -> Self {
        WriteBuilder {
            bit_direction: direction,
            ..self
        }
    }

    fn commit(mut self) -> Builder {
        self.parent.commands.push(
            Command::WriteDataShiftBytes {
                options: DataShiftOptions {
                    clock_direction: self.clock_direction,
                    bit_direction: self.bit_direction,
                },
                bytes: self.data,
            }
        );

        self.parent
    }

    pub fn then(self) -> Builder { 
        self.commit()
    }

    pub fn build(self) -> CommandList {
        self.commit().build()
    }
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
        
        assert_eq!(command_bytes, vec![0x10, 0x03, 0x00, 0x10, 0x01, 0x20, 0x01]);
    }
}
