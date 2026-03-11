use crate::token::{Token, TokenTag};
use crate::symtable::{FunAttr, FunTable, Qual, Type, VarAttr, VarTable};
use std::fs::File;
use std::io::Write;

/// Walks the token stream again and emits intermediate representation (IR)
/// to data.txt and code.txt, matching the original C++ gencode.cpp behaviour.
pub struct Codegen<'a> {
    tokens: &'a Vec<Token>,
    cursor: usize,
    hp: String,
    outdata: File,
    outcode: File,
    t_index: usize,
    a_index: usize,
    g_index: usize,
    s_index: usize,
    if_index: usize,
    for_index: usize,
    while_index: usize,
    dowhile_index: usize,
    indent: String,
    vtable: VarTable,
    ftable: FunTable,
}

impl<'a> Codegen<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        let outdata = File::create("data.txt").expect("Cannot create data.txt");
        let outcode = File::create("code.txt").expect("Cannot create code.txt");
        Codegen {
            tokens,
            cursor: 0,
            hp: String::new(),
            outdata,
            outcode,
            t_index: 0,
            a_index: 0,
            g_index: 0,
            s_index: 0,
            if_index: 0,
            for_index: 0,
            while_index: 0,
            dowhile_index: 0,
            indent: String::new(),
            vtable: VarTable::new(),
            ftable: FunTable::new(),
        }
    }

    pub fn generate(&mut self) {
        self.vtable.clear();
        self.program();
    }

    // --- Helpers ---

    fn newvar(&mut self, prefix: &str) -> String {
        match prefix {
            "g" => { let s = format!("g{}", self.g_index); self.g_index += 1; s }
            "a" => { let s = format!("a{}", self.a_index); self.a_index += 1; s }
            "s" => { let s = format!("s{}", self.s_index); self.s_index += 1; s }
            _   => { let s = format!("t{}", self.t_index); self.t_index += 1; s }
        }
    }

    fn resetvar(&mut self, prefix: &str) {
        match prefix {
            "g" => self.g_index = 0,
            "a" => self.a_index = 0,
            "t" => self.t_index = 0,
            "s" => self.s_index = 0,
            _ => {}
        }
    }

    fn newlabel(&mut self, prefix: &str) -> String {
        match prefix {
            "if"      => { let s = format!("if{}", self.if_index); self.if_index += 1; s }
            "for"     => { let s = format!("for{}", self.for_index); self.for_index += 1; s }
            "while"   => { let s = format!("while{}", self.while_index); self.while_index += 1; s }
            _         => { let s = format!("dowhile{}", self.dowhile_index); self.dowhile_index += 1; s }
        }
    }

    fn add_indent(&mut self) { self.indent.push('\t'); }
    fn sub_indent(&mut self) { if !self.indent.is_empty() { self.indent.pop(); } }

    fn match_tag(&mut self, tag: &TokenTag) -> bool {
        self.match_offset(0, tag)
    }

    fn match_offset(&mut self, offset: usize, tag: &TokenTag) -> bool {
        let pos = self.cursor + offset;
        if pos < self.tokens.len() && self.tokens[pos].tag == TokenTag::LineEnd {
            self.cursor += 1;
            return self.match_tag(tag);
        }
        pos < self.tokens.len() && self.tokens[pos].tag == *tag
    }

    fn eat(&mut self, tag: &TokenTag) -> String {
        while self.cursor < self.tokens.len()
            && self.tokens[self.cursor].tag == TokenTag::LineEnd
        {
            self.cursor += 1;
        }
        if self.cursor >= self.tokens.len() {
            eprintln!("codegen error: unexpected end of tokens");
            std::process::exit(1);
        }
        let val = self.tokens[self.cursor].value.clone();
        self.cursor += 1;
        val
    }

    fn peek_value(&self, offset: usize) -> String {
        let pos = self.cursor + offset;
        if pos < self.tokens.len() { self.tokens[pos].value.clone() } else { String::new() }
    }

    // --- IR emission helpers ---

    fn emit_data(&mut self, s: &str)  { writeln!(self.outdata, "{}{}", self.indent, s).ok(); }
    fn emit_code(&mut self, s: &str)  { writeln!(self.outcode, "{}{}", self.indent, s).ok(); }

    // --- Grammar / IR generation ---

    fn program(&mut self) {
        self.hp = self.newvar("g");
        self.emit_data(&format!("{} = 0x10040000", self.hp));
        self.const_data(true);
        self.variable_data(true);
        while (self.match_tag(&TokenTag::Void)
            || self.match_tag(&TokenTag::Int)
            || self.match_tag(&TokenTag::Char))
            && self.match_offset(1, &TokenTag::ID)
            && self.peek_value(1) != "main"
        {
            self.function();
        }
        self.main_function();
    }

    fn function(&mut self) {
        self.resetvar("a");
        self.resetvar("t");
        self.resetvar("s");

        if self.match_tag(&TokenTag::Void) {
            self.eat(&TokenTag::Void);
        } else {
            self.parse_type();
        }
        let id = self.eat(&TokenTag::ID);
        self.emit_code(&format!("FUNCTION {}:", id));
        self.add_indent();
        self.vtable.enter_scope();
        self.eat(&TokenTag::LParent);
        self.parse_args();
        self.eat(&TokenTag::RParent);
        self.block(true);
        self.emit_code("RET\n");
        self.sub_indent();
    }

    fn main_function(&mut self) {
        self.resetvar("a");
        self.resetvar("t");
        self.resetvar("s");
        self.emit_code("FUNCTION main:");
        self.add_indent();
        self.eat(&TokenTag::Void);
        self.eat(&TokenTag::ID);
        self.eat(&TokenTag::LParent);
        self.eat(&TokenTag::RParent);
        self.block(false);
        self.emit_code("RET\n");
        self.sub_indent();
    }

    fn block(&mut self, is_scoped: bool) {
        self.eat(&TokenTag::LBrace);
        if !is_scoped { self.vtable.enter_scope(); }
        self.const_data(false);
        self.variable_data(false);
        self.stmts();
        self.eat(&TokenTag::RBrace);
        self.vtable.exit_scope();
    }

    fn const_data(&mut self, is_global: bool) {
        while self.match_tag(&TokenTag::Const) {
            self.const_declaration(is_global);
            self.eat(&TokenTag::Semicn);
        }
    }

    fn const_declaration(&mut self, is_global: bool) {
        self.eat(&TokenTag::Const);
        let tp = self.parse_type();
        let id = self.eat(&TokenTag::ID);
        self.eat(&TokenTag::Assign);
        let (val_str, val_int) = self.atom_str();
        let var_name = if is_global {
            let v = self.newvar("g");
            self.emit_data(&format!("{} = {}", v, val_str));
            v
        } else {
            let v = self.newvar("s");
            self.emit_code(&format!("{} = {}", v, val_str));
            v
        };
        let attr = VarAttr { ty: tp, qual: Qual::Const, value: val_int, var: var_name };
        self.vtable.addsym(id, attr);

        while self.match_tag(&TokenTag::Comma) {
            self.eat(&TokenTag::Comma);
            let id2 = self.eat(&TokenTag::ID);
            self.eat(&TokenTag::Assign);
            let (vs2, vi2) = self.atom_str();
            let var2 = if is_global {
                let v = self.newvar("g"); self.emit_data(&format!("{} = {}", v, vs2)); v
            } else {
                let v = self.newvar("s"); self.emit_code(&format!("{} = {}", v, vs2)); v
            };
            let attr2 = VarAttr { ty: Type::Int, qual: Qual::Const, value: vi2, var: var2 };
            self.vtable.addsym(id2, attr2);
        }
    }

    fn variable_data(&mut self, is_global: bool) {
        while (self.match_tag(&TokenTag::Int) || self.match_tag(&TokenTag::Char))
            && !self.match_offset(2, &TokenTag::LParent)
        {
            self.variable_declaration(is_global);
            self.eat(&TokenTag::Semicn);
        }
    }

    fn variable_declaration(&mut self, is_global: bool) {
        let tp = self.parse_type();
        let id = self.eat(&TokenTag::ID);
        let var_name = if is_global {
            let v = self.newvar("g"); self.emit_data(&format!("{} = 0", v)); v
        } else {
            let v = self.newvar("s"); self.emit_code(&format!("{} = 0", v)); v
        };
        let attr = VarAttr { ty: tp.clone(), qual: Qual::None, value: 0, var: var_name };
        self.vtable.addsym(id, attr);

        while self.match_tag(&TokenTag::Comma) {
            self.eat(&TokenTag::Comma);
            let id2 = self.eat(&TokenTag::ID);
            let var2 = if is_global {
                let v = self.newvar("g"); self.emit_data(&format!("{} = 0", v)); v
            } else {
                let v = self.newvar("s"); self.emit_code(&format!("{} = 0", v)); v
            };
            let attr2 = VarAttr { ty: tp.clone(), qual: Qual::None, value: 0, var: var2 };
            self.vtable.addsym(id2, attr2);
        }
    }

    fn stmts(&mut self) {
        loop {
            if self.match_tag(&TokenTag::RBrace) { break; }
            if self.cursor >= self.tokens.len() { break; }
            self.stmt();
        }
    }

    fn stmt(&mut self) {
        if self.match_tag(&TokenTag::If) {
            self.if_condition();
        } else if self.match_tag(&TokenTag::For) {
            self.for_loop();
        } else if self.match_tag(&TokenTag::While) {
            self.while_loop();
        } else if self.match_tag(&TokenTag::Do) {
            self.dowhile_loop();
        } else if self.match_tag(&TokenTag::Scanf) {
            self.read();
            self.eat(&TokenTag::Semicn);
        } else if self.match_tag(&TokenTag::Printf) {
            self.print();
            self.eat(&TokenTag::Semicn);
        } else if self.match_tag(&TokenTag::Return) {
            self.ret();
            self.eat(&TokenTag::Semicn);
        } else if self.match_tag(&TokenTag::ID) && self.match_offset(1, &TokenTag::LParent) {
            self.call_stmt();
            self.eat(&TokenTag::Semicn);
        } else if self.match_tag(&TokenTag::ID) {
            self.assign();
            self.eat(&TokenTag::Semicn);
        }
    }

    fn assign(&mut self) {
        let id = self.eat(&TokenTag::ID);
        self.eat(&TokenTag::Assign);
        let var_name = self.vtable.getattr(&id).map(|v| v.var.clone()).unwrap_or_else(|| id.clone());
        let tmp = self.newvar("t");
        self.expr_into(&tmp);
        self.emit_code(&format!("{} = {}", var_name, tmp));
    }

    fn call_stmt(&mut self) {
        let id = self.eat(&TokenTag::ID);
        self.eat(&TokenTag::LParent);
        let vals = self.values_list();
        self.eat(&TokenTag::RParent);
        for v in &vals {
            self.emit_code(&format!("PUSH {}", v));
        }
        self.emit_code(&format!("CALL {}", id));
    }

    fn call_expr(&mut self) -> String {
        let id = self.eat(&TokenTag::ID);
        self.eat(&TokenTag::LParent);
        let vals = self.values_list();
        self.eat(&TokenTag::RParent);
        for v in &vals {
            self.emit_code(&format!("PUSH {}", v));
        }
        let ret = self.newvar("t");
        self.emit_code(&format!("{} = CALL {}", ret, id));
        ret
    }

    fn if_condition(&mut self) {
        let label = self.newlabel("if");
        let else_label = format!("{}_else", label);
        let end_label  = format!("{}_end", label);

        self.eat(&TokenTag::If);
        self.eat(&TokenTag::LParent);
        let cond = self.condition_str(true);
        self.eat(&TokenTag::RParent);
        self.emit_code(&format!("IF {} GOTO {}", cond, else_label));
        self.block(false);
        self.emit_code(&format!("GOTO {}", end_label));
        self.emit_code(&format!("{}:", else_label));
        if self.match_tag(&TokenTag::Else) {
            self.eat(&TokenTag::Else);
            self.block(false);
        }
        self.emit_code(&format!("{}:", end_label));
    }

    fn for_loop(&mut self) {
        let label = self.newlabel("for");
        let start = format!("{}_start", label);
        let end   = format!("{}_end", label);

        self.eat(&TokenTag::For);
        self.eat(&TokenTag::LParent);
        let init_id = self.eat(&TokenTag::ID);
        self.eat(&TokenTag::Assign);
        let (init_val, _) = self.atom_str();
        let var_name = self.vtable.getattr(&init_id)
            .map(|v| v.var.clone()).unwrap_or_else(|| init_id.clone());
        self.emit_code(&format!("{} = {}", var_name, init_val));
        self.eat(&TokenTag::Semicn);

        self.emit_code(&format!("{}:", start));
        let cond = self.condition_str(true);
        self.eat(&TokenTag::Semicn);
        self.emit_code(&format!("IF {} GOTO {}", cond, end));

        // Save step tokens (eat them, reemit after block — simplified: just parse and emit)
        let step_id = self.eat(&TokenTag::ID);
        self.eat(&TokenTag::Assign);
        let step_tmp = self.newvar("t");
        self.expr_into(&step_tmp);
        let step_var = self.vtable.getattr(&step_id)
            .map(|v| v.var.clone()).unwrap_or_else(|| step_id.clone());
        self.eat(&TokenTag::RParent);

        self.block(false);
        self.emit_code(&format!("{} = {}", step_var, step_tmp));
        self.emit_code(&format!("GOTO {}", start));
        self.emit_code(&format!("{}:", end));
    }

    fn while_loop(&mut self) {
        let label = self.newlabel("while");
        let start = format!("{}_start", label);
        let end   = format!("{}_end", label);

        self.eat(&TokenTag::While);
        self.eat(&TokenTag::LParent);
        self.emit_code(&format!("{}:", start));
        let cond = self.condition_str(true);
        self.eat(&TokenTag::RParent);
        self.emit_code(&format!("IF {} GOTO {}", cond, end));
        self.block(false);
        self.emit_code(&format!("GOTO {}", start));
        self.emit_code(&format!("{}:", end));
    }

    fn dowhile_loop(&mut self) {
        let label = self.newlabel("dowhile");
        let start = format!("{}_start", label);

        self.emit_code(&format!("{}:", start));
        self.eat(&TokenTag::Do);
        self.block(false);
        self.eat(&TokenTag::While);
        self.eat(&TokenTag::LParent);
        let cond = self.condition_str(false);
        self.eat(&TokenTag::RParent);
        self.eat(&TokenTag::Semicn);
        self.emit_code(&format!("IF {} GOTO {}", cond, start));
    }

    fn read(&mut self) {
        self.eat(&TokenTag::Scanf);
        self.eat(&TokenTag::LParent);
        self.eat(&TokenTag::STR);
        self.eat(&TokenTag::Comma);
        let id = self.eat(&TokenTag::ID);
        self.eat(&TokenTag::RParent);
        let var_name = self.vtable.getattr(&id).map(|v| v.var.clone()).unwrap_or(id);
        self.emit_code(&format!("READ {}", var_name));
    }

    fn print(&mut self) {
        self.eat(&TokenTag::Printf);
        self.eat(&TokenTag::LParent);
        let fmt = self.eat(&TokenTag::STR);
        let mut args = vec![];
        while self.match_tag(&TokenTag::Comma) {
            self.eat(&TokenTag::Comma);
            let tmp = self.newvar("t");
            self.expr_into(&tmp);
            args.push(tmp);
        }
        self.eat(&TokenTag::RParent);
        let args_str = args.join(", ");
        self.emit_code(&format!("PRINT \"{}\", {}", fmt, args_str));
    }

    fn ret(&mut self) {
        self.eat(&TokenTag::Return);
        if !self.match_tag(&TokenTag::Semicn) {
            let tmp = self.newvar("t");
            self.expr_into(&tmp);
            self.emit_code(&format!("RET {}", tmp));
        } else {
            self.emit_code("RET");
        }
    }

    // Returns IR variable holding expr result
    fn expr_into(&mut self, dest: &str) {
        let tmp = self.expr_str();
        self.emit_code(&format!("{} = {}", dest, tmp));
    }

    fn expr_str(&mut self) -> String {
        let mut lhs = self.term_str();
        while self.match_tag(&TokenTag::Add) || self.match_tag(&TokenTag::Sub) {
            let op = if self.match_tag(&TokenTag::Add) {
                self.eat(&TokenTag::Add); "+"
            } else {
                self.eat(&TokenTag::Sub); "-"
            };
            let rhs = self.term_str();
            let tmp = self.newvar("t");
            self.emit_code(&format!("{} = {} {} {}", tmp, lhs, op, rhs));
            lhs = tmp;
        }
        lhs
    }

    fn term_str(&mut self) -> String {
        let mut lhs = self.factor_str();
        while self.match_tag(&TokenTag::Mul) || self.match_tag(&TokenTag::Div) {
            let op = if self.match_tag(&TokenTag::Mul) {
                self.eat(&TokenTag::Mul); "*"
            } else {
                self.eat(&TokenTag::Div); "/"
            };
            let rhs = self.factor_str();
            let tmp = self.newvar("t");
            self.emit_code(&format!("{} = {} {} {}", tmp, lhs, op, rhs));
            lhs = tmp;
        }
        lhs
    }

    fn factor_str(&mut self) -> String {
        if self.match_tag(&TokenTag::LParent) {
            self.eat(&TokenTag::LParent);
            let v = self.expr_str();
            self.eat(&TokenTag::RParent);
            v
        } else if self.match_tag(&TokenTag::ID) && self.match_offset(1, &TokenTag::LParent) {
            self.call_expr()
        } else if self.match_tag(&TokenTag::ID) {
            let id = self.eat(&TokenTag::ID);
            self.vtable.getattr(&id).map(|v| v.var.clone()).unwrap_or(id)
        } else if self.match_tag(&TokenTag::NUM) {
            self.eat(&TokenTag::NUM)
        } else if self.match_tag(&TokenTag::CHR) {
            self.eat(&TokenTag::CHR)
        } else {
            self.eat(&TokenTag::NUM)
        }
    }

    fn parse_args(&mut self) {
        if !self.match_tag(&TokenTag::RParent) {
            let tp = self.parse_type();
            let id = self.eat(&TokenTag::ID);
            let v = self.newvar("a");
            let attr = VarAttr { ty: tp, qual: Qual::None, value: 0, var: v.clone() };
            self.vtable.addsym(id, attr);
            self.emit_code(&format!("PARAM {}", v));
            while self.match_tag(&TokenTag::Comma) {
                self.eat(&TokenTag::Comma);
                let tp2 = self.parse_type();
                let id2 = self.eat(&TokenTag::ID);
                let v2 = self.newvar("a");
                let attr2 = VarAttr { ty: tp2, qual: Qual::None, value: 0, var: v2.clone() };
                self.vtable.addsym(id2, attr2);
                self.emit_code(&format!("PARAM {}", v2));
            }
        }
    }

    fn values_list(&mut self) -> Vec<String> {
        let mut vals = vec![];
        if !self.match_tag(&TokenTag::RParent) {
            let tmp = self.newvar("t");
            self.expr_into(&tmp);
            vals.push(tmp);
            while self.match_tag(&TokenTag::Comma) {
                self.eat(&TokenTag::Comma);
                let tmp2 = self.newvar("t");
                self.expr_into(&tmp2);
                vals.push(tmp2);
            }
        }
        vals
    }

    /// Returns condition string like "a0 < s1"; reverse=true means negate for branch
    fn condition_str(&mut self, _reverse: bool) -> String {
        let lhs = self.expr_str();
        let cmp_tags = [
            (TokenTag::GE, ">="), (TokenTag::GT, ">"),
            (TokenTag::LE, "<="), (TokenTag::LT, "<"),
            (TokenTag::EQ, "=="), (TokenTag::NE, "!="),
        ];
        for (tag, sym) in &cmp_tags {
            if self.match_tag(tag) {
                self.eat(tag);
                let rhs = self.expr_str();
                return format!("{} {} {}", lhs, sym, rhs);
            }
        }
        lhs
    }

    fn atom_str(&mut self) -> (String, i32) {
        let neg = if self.match_tag(&TokenTag::Sub) {
            self.eat(&TokenTag::Sub); true
        } else {
            if self.match_tag(&TokenTag::Add) { self.eat(&TokenTag::Add); }
            false
        };
        if self.match_tag(&TokenTag::NUM) {
            let s = self.eat(&TokenTag::NUM);
            let n: i32 = s.parse().unwrap_or(0);
            let n = if neg { -n } else { n };
            (n.to_string(), n)
        } else if self.match_tag(&TokenTag::CHR) {
            let s = self.eat(&TokenTag::CHR);
            let n = s.chars().next().map(|c| c as i32).unwrap_or(0);
            let n = if neg { -n } else { n };
            (format!("'{}'", s), n)
        } else {
            ("0".to_string(), 0)
        }
    }

    fn parse_type(&mut self) -> Type {
        if self.match_tag(&TokenTag::Int) {
            self.eat(&TokenTag::Int); Type::Int
        } else if self.match_tag(&TokenTag::Char) {
            self.eat(&TokenTag::Char); Type::Char
        } else {
            self.eat(&TokenTag::Void); Type::Void
        }
    }
}
