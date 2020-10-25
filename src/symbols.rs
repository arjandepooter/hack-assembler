use crate::types::*;
use std::collections::HashMap;

type SymbolTable = HashMap<String, u16>;

const SYMBOLS: [(&str, u16); 23] = [
    ("SP", 0x0000),
    ("LCL", 0x0001),
    ("ARG", 0x0002),
    ("THIS", 0x0003),
    ("THAT", 0x0004),
    ("R0", 0x0000),
    ("R1", 0x0001),
    ("R2", 0x0002),
    ("R3", 0x0003),
    ("R4", 0x0004),
    ("R5", 0x0005),
    ("R6", 0x0006),
    ("R7", 0x0007),
    ("R8", 0x0008),
    ("R9", 0x0009),
    ("R10", 0x000a),
    ("R11", 0x000b),
    ("R12", 0x000c),
    ("R13", 0x000d),
    ("R14", 0x000e),
    ("R15", 0x000f),
    ("SCREEN", 0x4000),
    ("KBD", 0x6000),
];

fn get_default_symbols() -> SymbolTable {
    Vec::from(SYMBOLS)
        .into_iter()
        .map(|(key, address)| (key.into(), address))
        .collect()
}

fn add_label_symbols(
    symbols: SymbolTable,
    instructions: &Vec<Instruction>,
) -> Result<SymbolTable, String> {
    instructions
        .iter()
        .fold(
            Ok::<(u16, SymbolTable), String>((0u16, symbols.clone())),
            |acc, instruction| match acc {
                Ok((instruction_pointer, mut symbols)) => match instruction {
                    Instruction::Label(label) => {
                        if symbols.contains_key(label) {
                            Err(format!(
                                "Error while adding {} to symbol table as it already exisits",
                                label
                            ))
                        } else {
                            symbols.insert(label.clone(), instruction_pointer);
                            Ok((instruction_pointer, symbols))
                        }
                    }
                    _ => Ok((instruction_pointer + 1, symbols)),
                },
                Err(_) => acc,
            },
        )
        .map(|(_, symbols)| symbols)
}

fn add_variable_symbols(
    symbols: SymbolTable,
    instructions: &Vec<Instruction>,
) -> Result<SymbolTable, String> {
    Err("Invalid variable".to_string())
}

pub fn process_symbols(instructions: Vec<Instruction>) -> Result<SymbolTable, String> {
    let symbols = get_default_symbols();
    add_label_symbols(symbols, &instructions)
}
