use crate::optimize::IrLine;
use std::collections::HashMap;
use std::fs;
use std::io::Write;

/// MIPS register allocator and code emitter.
/// Reads the IR (code.txt or opt_code.txt) and emits MIPS assembly to mips_out.asm.
pub fn mips(optimize_on: bool) {
    let ir_file = if optimize_on { "opt_code.txt" } else { "code.txt" };
    let raw = fs::read_to_string(ir_file).unwrap_or_default();
    let lines: Vec<IrLine> = raw.lines()
        .map(|l| IrLine { text: l.to_string() })
        .collect();

    let mut emitter = MipsEmitter::new();
    emitter.emit(&lines);
    emitter.write("mips_out.asm");
}

struct MipsEmitter {
    output: Vec<String>,
    reg_map: HashMap<String, String>,
    reg_pool: Vec<String>,
    next_reg: usize,
}

impl MipsEmitter {
    fn new() -> Self {
        let regs = (8..=25_usize).map(|i| format!("$t{}", i - 8)).collect::<Vec<_>>();
        MipsEmitter {
            output: Vec::new(),
            reg_map: HashMap::new(),
            reg_pool: regs,
            next_reg: 0,
        }
    }

    fn alloc_reg(&mut self, var: &str) -> String {
        if let Some(r) = self.reg_map.get(var) {
            return r.clone();
        }
        let reg = if self.next_reg < self.reg_pool.len() {
            self.reg_pool[self.next_reg].clone()
        } else {
            // Spill: reuse $t9 as spill register (simplified)
            "$t9".to_string()
        };
        self.next_reg += 1;
        self.reg_map.insert(var.to_string(), reg.clone());
        reg
    }

    fn emit_line(&mut self, s: impl Into<String>) {
        self.output.push(s.into());
    }

    fn emit(&mut self, lines: &[IrLine]) {
        self.emit_line(".data");
        self.emit_line(".text");

        for line in lines {
            let trimmed = line.text.trim();
            if trimmed.is_empty() { continue; }

            if trimmed.starts_with("FUNCTION ") {
                // FUNCTION foo:
                let name = trimmed["FUNCTION ".len()..].trim_end_matches(':');
                if name == "main" {
                    self.emit_line("main:");
                } else {
                    self.emit_line(format!("{}:", name));
                }
            } else if trimmed == "RET" {
                self.emit_line("\tjr $ra");
            } else if trimmed.starts_with("RET ") {
                let var = &trimmed["RET ".len()..];
                let reg = self.alloc_reg(var);
                self.emit_line(format!("\tmove $v0, {}", reg));
                self.emit_line("\tjr $ra");
            } else if trimmed.starts_with("READ ") {
                let var = &trimmed["READ ".len()..];
                let reg = self.alloc_reg(var);
                self.emit_line("\tli $v0, 5");
                self.emit_line("\tsyscall");
                self.emit_line(format!("\tmove {}, $v0", reg));
            } else if trimmed.starts_with("PRINT ") {
                self.emit_line("\tli $v0, 4");
                self.emit_line("\tsyscall");
            } else if trimmed.starts_with("CALL ") {
                let fname = &trimmed["CALL ".len()..].trim();
                self.emit_line(format!("\tjal {}", fname));
            } else if trimmed.starts_with("PUSH ") {
                let var = &trimmed["PUSH ".len()..];
                let reg = self.alloc_reg(var);
                self.emit_line("\taddiu $sp, $sp, -4");
                self.emit_line(format!("\tsw {}, 0($sp)", reg));
            } else if trimmed.starts_with("PARAM ") {
                let var = &trimmed["PARAM ".len()..];
                let reg = self.alloc_reg(var);
                self.emit_line(format!("\tlw {}, 0($sp)", reg));
                self.emit_line("\taddiu $sp, $sp, 4");
            } else if trimmed.starts_with("GOTO ") {
                let lbl = &trimmed["GOTO ".len()..];
                self.emit_line(format!("\tj {}", lbl));
            } else if trimmed.starts_with("IF ") && trimmed.contains("GOTO") {
                // IF lhs OP rhs GOTO label
                if let Some(goto_pos) = trimmed.find("GOTO") {
                    let cond_str = trimmed["IF ".len()..goto_pos].trim();
                    let label    = trimmed[goto_pos + 4..].trim();
                    self.emit_condition_branch(cond_str, label);
                }
            } else if trimmed.ends_with(':') {
                // label definition
                self.emit_line(format!("{}:", trimmed.trim_end_matches(':')));
            } else if let Some(eq_pos) = trimmed.find(" = ") {
                // Assignment: dest = expr
                let dest = trimmed[..eq_pos].trim();
                let rhs  = trimmed[eq_pos + 3..].trim();
                self.emit_assignment(dest, rhs);
            }
        }
    }

    fn emit_assignment(&mut self, dest: &str, rhs: &str) {
        // Try arithmetic: "a OP b"
        let ops = [("+", "add"), ("-", "sub"), ("*", "mul"), ("/", "div")];
        for (sym, mips_op) in &ops {
            // Find operator with space context to avoid partial match on negative nums
            if let Some(pos) = rhs.find(&format!(" {} ", sym)) {
                let lhs_s = rhs[..pos].trim();
                let rhs_s = rhs[pos + sym.len() + 2..].trim();
                let dest_reg = self.alloc_reg(dest);
                let lhs_r = self.get_operand(lhs_s);
                let rhs_r = self.get_operand(rhs_s);
                self.emit_line(format!("\t{} {}, {}, {}", mips_op, dest_reg, lhs_r, rhs_r));
                return;
            }
        }
        // Simple assignment: dest = val
        let dest_reg = self.alloc_reg(dest);
        if let Ok(n) = rhs.parse::<i64>() {
            self.emit_line(format!("\tli {}, {}", dest_reg, n));
        } else if rhs.starts_with("0x") {
            self.emit_line(format!("\tli {}, {}", dest_reg, rhs));
        } else {
            let src_reg = self.alloc_reg(rhs);
            self.emit_line(format!("\tmove {}, {}", dest_reg, src_reg));
        }
    }

    fn get_operand(&mut self, s: &str) -> String {
        if let Ok(n) = s.parse::<i64>() {
            // Load immediate into a fresh temp
            let tmp = format!("__imm_{}", s);
            let r = self.alloc_reg(&tmp);
            self.emit_line(format!("\tli {}, {}", r, n));
            r
        } else {
            self.alloc_reg(s)
        }
    }

    fn emit_condition_branch(&mut self, cond: &str, label: &str) {
        let ops = [
            (">=", "bge"), (">", "bgt"),
            ("<=", "ble"), ("<", "blt"),
            ("==", "beq"), ("!=", "bne"),
        ];
        for (sym, mips_br) in &ops {
            if let Some(pos) = cond.find(sym) {
                let lhs = cond[..pos].trim();
                let rhs = cond[pos + sym.len()..].trim();
                let lr = self.alloc_reg(lhs);
                let rr = self.get_operand(rhs);
                self.emit_line(format!("\t{} {}, {}, {}", mips_br, lr, rr, label));
                return;
            }
        }
        // No operator — branch if non-zero
        let r = self.alloc_reg(cond);
        self.emit_line(format!("\tbne {}, $zero, {}", r, label));
    }

    fn write(&self, path: &str) {
        if let Ok(mut f) = fs::File::create(path) {
            for line in &self.output {
                writeln!(f, "{}", line).ok();
            }
        }
        eprintln!("MIPS assembly written to {}", path);
    }
}
