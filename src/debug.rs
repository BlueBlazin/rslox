use crate::chunk::Chunk;
use crate::gc::Heap;
use crate::opcodes::OpCode;
use crate::value::Value;
use std::fmt;

macro_rules! simple_instr {
    ($output:expr, $i:expr, $opcode:expr) => {{
        $output.push_str(&format!("{:10}\n", $opcode));
        $i += 1;
    }};
}

macro_rules! const_instr {
    ($output:expr, $i:expr, $opcode:expr, $chunk:expr) => {{
        let constant = $chunk.code[$i + 1] as usize;
        let handle = $chunk.constants[constant];

        // // It's either have an unsafe here or make debug a separate function
        // // which gets passed the `heap` to get values with further checks.
        // // Is the convenience of implementing fmt::Debug worth the 'unsafety'?
        // let value = unsafe { &*handle.ptr };

        $output.push_str(&format!("{:12} {:4} '{:?}'\n", $opcode, constant, handle));

        $i += 2;
    }};
}

macro_rules! byte_instr {
    ($output:expr, $i:expr, $opcode:expr, $chunk:expr) => {{
        let idx = $chunk.code[$i + 1] as usize;

        $output.push_str(&format!("{:12} {:4}\n", $opcode, idx));

        $i += 2;
    }};
}

macro_rules! jump_instr {
    ($output:expr, $i:expr, $opcode:expr, $sign:expr, $chunk:expr) => {{
        let mut jump = ($chunk.code[$i + 1] as u16) << 8;
        jump |= $chunk.code[$i + 2] as u16;

        $output.push_str(&format!(
            "{:12} {:4} -> {}\n",
            $opcode,
            $i,
            $i + 3 + ($sign * jump) as usize
        ));

        $i += 3;
    }};
}

// pub fn debug(chunk: Chunk, heap: Heap<Value>) -> String {
//     let mut output = format!("=== {} ===\n", chunk.name);

//     let mut i = 0;
//     let mut num = 0;

//     while i < chunk.code.len() {
//         let opcode = format!("{:?}", OpCode::from(chunk.code[i]));
//         let line = chunk.lines[i];

//         if i > 0 && line == chunk.lines[i - 1] {
//             output.push_str(&format!("{:04}    | ", num));
//         } else {
//             output.push_str(&format!("{:04} {:4} ", num, line));
//         }

//         match OpCode::from(chunk.code[i]) {
//             OpCode::Return => simple_instr!(output, i, opcode),
//             OpCode::Constant => const_instr!(output, i, opcode, chunk),
//             OpCode::Negate => simple_instr!(output, i, opcode),
//             OpCode::Add | OpCode::Subtract => simple_instr!(output, i, opcode),
//             OpCode::Multiply | OpCode::Divide => simple_instr!(output, i, opcode),
//             OpCode::Nil | OpCode::True | OpCode::False => simple_instr!(output, i, opcode),
//             OpCode::Not => simple_instr!(output, i, opcode),
//             OpCode::Equal | OpCode::Greater | OpCode::Less => simple_instr!(output, i, opcode),
//             OpCode::Print => simple_instr!(output, i, opcode),
//             OpCode::Pop => simple_instr!(output, i, opcode),
//             OpCode::DefineGlobal => const_instr!(output, i, opcode, chunk),
//             OpCode::GetGlobal => const_instr!(output, i, opcode, chunk),
//             OpCode::SetGlobal => const_instr!(output, i, opcode, chunk),
//             OpCode::GetLocal => byte_instr!(output, i, opcode, chunk),
//             OpCode::SetLocal => byte_instr!(output, i, opcode, chunk),
//             OpCode::JumpIfFalse => jump_instr!(output, i, opcode, 1, chunk),
//             OpCode::Jump => jump_instr!(output, i, opcode, 1, chunk),
//             OpCode::Loop => jump_instr!(output, i, opcode, 1, chunk),
//             OpCode::Call => byte_instr!(output, i, opcode, chunk),
//             OpCode::Closure => {
//                 let constant = chunk.code[i + 1] as usize;
//                 let handle = chunk.constants[constant];

//                 i += 2;

//                 let value = unsafe { &*handle.ptr };

//                 match value {
//                     Value::Closure(closure) => {
//                         let upvalue_count = closure.upvalue_count;

//                         for _ in 0..upvalue_count {
//                             let is_local = chunk.code[i] != 0;
//                             let index = chunk.code[i + 1];
//                             i += 2;

//                             output.push_str(&format!(
//                                 "{:04}      |                     {} {}\n",
//                                 i - 2,
//                                 is_local,
//                                 index
//                             ));
//                         }
//                     }
//                     _ => panic!("Unexpected type in debug. Expected Closure."),
//                 }
//             }
//             OpCode::GetUpvalue => const_instr!(output, i, opcode, chunk),
//             OpCode::SetUpvalue => const_instr!(output, i, opcode, chunk),
//         }

//         num += 1;
//     }

//     output
// }

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let mut output = format!("=== {} ===\n", self.name);
        let mut output = String::from("");

        let mut i = 0;
        let mut num = 0;

        while i < self.code.len() {
            let opcode = format!("{:?}", OpCode::from(self.code[i]));
            let line = self.lines[i];

            if i > 0 && line == self.lines[i - 1] {
                output.push_str(&format!("{:04}    | ", num));
            } else {
                output.push_str(&format!("{:04} {:4} ", num, line));
            }

            match OpCode::from(self.code[i]) {
                OpCode::Return => simple_instr!(output, i, opcode),
                OpCode::Constant => const_instr!(output, i, opcode, self),
                OpCode::Negate => simple_instr!(output, i, opcode),
                OpCode::Add | OpCode::Subtract => simple_instr!(output, i, opcode),
                OpCode::Multiply | OpCode::Divide => simple_instr!(output, i, opcode),
                OpCode::Nil | OpCode::True | OpCode::False => simple_instr!(output, i, opcode),
                OpCode::Not => simple_instr!(output, i, opcode),
                OpCode::Equal | OpCode::Greater | OpCode::Less => simple_instr!(output, i, opcode),
                OpCode::Print => simple_instr!(output, i, opcode),
                OpCode::Pop => simple_instr!(output, i, opcode),
                OpCode::DefineGlobal => const_instr!(output, i, opcode, self),
                OpCode::GetGlobal => const_instr!(output, i, opcode, self),
                OpCode::SetGlobal => const_instr!(output, i, opcode, self),
                OpCode::GetLocal => byte_instr!(output, i, opcode, self),
                OpCode::SetLocal => byte_instr!(output, i, opcode, self),
                OpCode::JumpIfFalse => jump_instr!(output, i, opcode, 1, self),
                OpCode::Jump => jump_instr!(output, i, opcode, 1, self),
                OpCode::Loop => jump_instr!(output, i, opcode, 1, self),
                OpCode::Call => byte_instr!(output, i, opcode, self),
                OpCode::Closure => {
                    let constant = self.code[i + 1] as usize;
                    let handle = self.constants[constant];

                    output.push_str(&format!("{:12} {:4} {:?}\n", opcode, constant, handle));

                    i += 2;

                    // Justification for unsafe: At this point we're debugging,
                    // so our program has already failed making using unsafe slightly less worse
                    // than otherwise. The other, more important, reason is that without it
                    // we need a separate function which takes the heap as an argument.
                    let value = unsafe { &*handle.ptr };

                    // output.push_str(&format!("{:?}\n", value));

                    match value {
                        Value::Closure(closure) => {
                            let upvalue_count = closure.upvalue_count;

                            for _ in 0..upvalue_count {
                                let is_local = self.code[i] != 0;
                                let index = self.code[i + 1];
                                i += 2;

                                output.push_str(&format!(
                                    "{:04}    |                 {} {}\n",
                                    i - 2,
                                    is_local,
                                    index
                                ));
                            }
                            output.push_str(&format!("----End {:?}----\n", &closure.name.unwrap()));
                        }
                        _ => panic!("Unexpected type in debug. Expected Closure."),
                    }
                }
                OpCode::GetUpvalue => byte_instr!(output, i, opcode, self),
                OpCode::SetUpvalue => byte_instr!(output, i, opcode, self),
            }

            num += 1;
        }

        write!(f, "{}", &output[..output.len() - 1])
    }
}
