use std::collections::VecDeque;
use std::fs::File;
use std::io::{stdin, ErrorKind, Read};

fn main() -> Result<(), UMError> {
    let args: Vec<_> = std::env::args().collect();
    let filename = &args[1];
    let program = read_program(filename)?;

    let mut reg = [0u32; 8];
    let mut platter_arrays = Vec::from([program]);
    let mut finger = 0usize;
    let mut abandoned = VecDeque::new();

    loop {
        let platter = platter_arrays[0][finger];
        let op = platter >> 28;
        let a = (platter >> 6 & 0b111) as usize;
        let b = (platter >> 3 & 0b111) as usize;
        let c = (platter & 0b111) as usize;
        // eprintln!(
        //     "Platter {:0<32b}, Operator {}, a: {}, b: {}, c: {}, registries: {:?}",
        //     platter, op, a, b, c, reg
        // );

        match op {
            //ConditionalMove
            0 => {
                if reg[c] != 0 {
                    reg[a] = reg[b]
                }
            }
            //ArrayIndex
            1 => reg[a] = platter_arrays[reg[b] as usize][reg[c] as usize],
            //ArrayAmendment
            2 => platter_arrays[reg[a] as usize][reg[b] as usize] = reg[c],
            // Addition
            3 => reg[a] = reg[b].wrapping_add(reg[c]),
            // Multiplication
            4 => reg[a] = reg[b].wrapping_mul(reg[c]),
            // Division
            5 => reg[a] = reg[b] / reg[c],
            // NotAnd
            6 => reg[a] = !(reg[b] & reg[c]),
            // Halt
            7 => break,
            // Allocation
            8 => {
                let new_index = abandoned
                    .pop_front()
                    .unwrap_or_else(|| platter_arrays.len());
                if new_index < platter_arrays.len() {
                    platter_arrays[new_index] = vec![0; reg[c] as usize];
                } else {
                    platter_arrays.insert(new_index, vec![0; reg[c] as usize]);
                }
                reg[b] = new_index as u32;
            }
            // Abandonment
            9 => {
                platter_arrays[reg[c] as usize] = Vec::new();
                abandoned.push_back(reg[c] as usize);
            }
            // Output
            10 => {
                print!(
                    "{}",
                    char::from_u32(reg[c]).ok_or_else(|| UMError(format!(
                        "invalid character in register C: {}",
                        reg[c]
                    )))?
                )
            }
            // Input
            11 => {
                let mut char = [0u8; 1];
                match stdin().read_exact(&mut char) {
                    Ok(_) => reg[c] = char[0] as u32,
                    Err(err) if err.kind() == ErrorKind::UnexpectedEof => reg[c] = !0,
                    Err(err) => return Err(UMError(err.to_string())),
                }
            }
            // Load
            12 => {
                if reg[b] != 0 {
                    platter_arrays[0] = platter_arrays[reg[b] as usize].clone();
                }
                finger = reg[c] as usize;
                continue;
            }
            // Orthography
            13 => {
                let reg_index = (platter >> 25) & 0b111;
                let val = platter & 0b1_1111_1111_1111_1111_1111_1111;
                reg[reg_index as usize] = val;
            }
            op => return Err(UMError(format!("Unknown operator: {}", op))),
        }

        finger += 1;
    }

    Ok(())
}

fn read_program(filename: &str) -> Result<Vec<u32>, UMError> {
    let mut file = File::open(filename)?;
    let mut program_bytes = Vec::new();
    file.read_to_end(&mut program_bytes)?;

    let program: Vec<u32> = program_bytes
        .chunks_exact(4)
        .map(|bytes| u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect();
    Ok(program)
}

#[derive(Debug)]
struct UMError(String);

impl From<std::io::Error> for UMError {
    fn from(err: std::io::Error) -> Self {
        Self(err.to_string())
    }
}
