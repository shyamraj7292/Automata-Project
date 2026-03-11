use std::fs;
use std::io::Write;

/// A single IR instruction line  
#[derive(Debug, Clone)]
pub struct IrLine {
    pub text: String,
}

/// Reads code.txt and performs basic optimizations:
/// - Constant folding (e.g., `t0 = 3 + 2`  →  `t0 = 5`)
/// - Dead code elimination (removes assignments to temps never read)
pub fn optimize(optimize_on: bool) -> Vec<IrLine> {
    let raw = fs::read_to_string("code.txt").unwrap_or_default();
    let lines: Vec<IrLine> = raw.lines()
        .map(|l| IrLine { text: l.to_string() })
        .collect();

    if !optimize_on {
        return lines;
    }

    let folded = constant_fold(lines);
    let result = dead_code_elim(folded);
    result
}

/// Constant folding: evaluate simple arithmetic on integer literals at compile time.
fn constant_fold(lines: Vec<IrLine>) -> Vec<IrLine> {
    lines.into_iter().map(|line| {
        // Pattern: "  dest = lhs OP rhs"  where lhs and rhs are integer literals
        let trimmed = line.text.trim();
        if let Some(eq_pos) = trimmed.find(" = ") {
            let dest = trimmed[..eq_pos].trim();
            let rhs_expr = trimmed[eq_pos + 3..].trim();
            // Try to parse "NUM OP NUM"
            for op in &["+", "-", "*", "/"] {
                if let Some(op_pos) = rhs_expr.find(op) {
                    // Make sure it's surrounded by spaces (avoid partial matches like ">=")
                    let before = rhs_expr[..op_pos].trim();
                    let after  = rhs_expr[op_pos + op.len()..].trim();
                    if let (Ok(a), Ok(b)) = (before.parse::<i64>(), after.parse::<i64>()) {
                        let result = match *op {
                            "+" => a + b,
                            "-" => a - b,
                            "*" => a * b,
                            "/" if b != 0 => a / b,
                            _ => break,
                        };
                        let leading = &line.text[..line.text.len() - trimmed.len()];
                        return IrLine {
                            text: format!("{}{} = {}", leading, dest, result),
                        };
                    }
                }
            }
        }
        line
    }).collect()
}

/// Dead code elimination: remove assignments to temporaries (t*) that are never used.
fn dead_code_elim(lines: Vec<IrLine>) -> Vec<IrLine> {
    // Collect all temporaries that are defined but never used on the RHS
    let all_text: String = lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n");

    lines.into_iter().filter(|line| {
        let trimmed = line.text.trim();
        if let Some(eq_pos) = trimmed.find(" = ") {
            let dest = trimmed[..eq_pos].trim();
            // Only remove pure temporary vars like t0, t1, ...
            if dest.starts_with('t') && dest[1..].parse::<u32>().is_ok() {
                // Count how often this temp appears in the whole IR
                let occurrences = all_text.matches(dest).count();
                // If only 1 occurrence it's only the definition — eliminate
                return occurrences > 1;
            }
        }
        true
    }).collect()
}

/// Write optimized IR back to opt_code.txt
pub fn write_optimized(lines: &[IrLine]) {
    if let Ok(mut f) = fs::File::create("opt_code.txt") {
        for line in lines {
            writeln!(f, "{}", line.text).ok();
        }
    }
}
