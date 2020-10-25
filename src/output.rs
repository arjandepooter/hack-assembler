use crate::symbols::SymbolTable;
use crate::types::*;

fn instruction_to_int(instruction: &Instruction, symbols: &SymbolTable) -> Option<u16> {
    match instruction {
        Instruction::AInstruction(value) => {
            let address = match value {
                AValue::Label(label) => *symbols.get(label).unwrap_or(&0),
                AValue::Value(address) => *address,
            };
            Some(address)
        }
        Instruction::CInstruction {
            destination,
            instruction,
            jump,
        } => {
            let mut instr: u16 = 0b111;
            instr = (instr << 7) + *instruction as u16;
            instr = (instr << 3) + *destination as u16;
            instr = (instr << 3) + *jump as u16;
            Some(instr)
        }
        _ => None,
    }
}

fn format_machine_instruction(instr_int: u16) -> String {
    format!("{:016b}", instr_int)
}

pub fn to_machine_instructions(
    instructions: &Vec<Instruction>,
    symbols: SymbolTable,
) -> Vec<String> {
    instructions
        .into_iter()
        .map(|instruction| instruction_to_int(instruction, &symbols))
        .filter(|instr_int| instr_int.is_some())
        .map(|instr_int| instr_int.unwrap())
        .map(format_machine_instruction)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn formatter() {
        assert_eq!(format_machine_instruction(0), "0000000000000000");
        assert_eq!(format_machine_instruction(0x40A0), "0100000010100000");
    }

    #[test]
    fn a_instruction_to_int() {
        assert_eq!(
            instruction_to_int(
                &Instruction::AInstruction(AValue::Value(0x0100)),
                &HashMap::new()
            ),
            Some(0x0100)
        )
    }

    #[test]
    fn a_instruction_to_int_with_label() {
        let mut symbols: SymbolTable = HashMap::new();
        symbols.insert(String::from("HOI"), 0x0400);

        assert_eq!(
            instruction_to_int(
                &Instruction::AInstruction(AValue::Label(String::from("HOI"))),
                &symbols
            ),
            Some(0x0400)
        )
    }

    #[test]
    fn c_instruction() {
        assert_eq!(
            instruction_to_int(
                &Instruction::CInstruction {
                    destination: 0b100,
                    jump: 0b011,
                    instruction: 0b0110000
                },
                &HashMap::new()
            ),
            Some(0b1110110000100011)
        )
    }
}
