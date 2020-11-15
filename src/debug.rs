use crate::chunk::Chunk;
use crate::opcodes::OpCode;
use std::fmt;

macro_rules! simple_instr {
    ($output:expr, $i:expr, $opcode:expr) => {{
        $output.push_str(&format!("{:10}\n", $opcode));
        $i += 1;
    }};
}

macro_rules! const_instr {
    ($output:expr, $i:expr, $opcode:expr, $self:expr) => {{
        let constant = $self.code[$i + 1] as usize;

        $output.push_str(&format!(
            "{:12} {:4} '{:?}'\n",
            $opcode, constant, $self.constants[constant]
        ));

        $i += 2;
    }};
}

macro_rules! byte_instr {
    ($output:expr, $i:expr, $opcode:expr, $self:expr) => {{
        let idx = $self.code[$i + 1] as usize;

        $output.push_str(&format!("{:12} {:4}\n", $opcode, idx));

        $i += 2;
    }};
}

impl fmt::Debug for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = format!("=== {} ===\n", self.name);

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
            }

            num += 1;
        }

        write!(f, "{}", output)
    }
}
