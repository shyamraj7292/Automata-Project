use crate::token::{Token, TokenTag};
use crate::symtable::{FunAttr, FunTable, Qual, Type, VarAttr, VarTable};
use std::fs::File;
use std::io::Write;

/// Recursive-descent parser — translates the token stream into a syntax tree
/// log file (syntaxtree.txt) and validates the program structure.
pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    cursor: usize,
    numlines: usize,
    output: File,
    pub vtable: VarTable,
    pub ftable: FunTable,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        let output = File::create("syntaxtree.txt").expect("Cannot create syntaxtree.txt");
        Parser {
            tokens,
            cursor: 0,
            numlines: 1,
            output,
            vtable: VarTable::new(),
            ftable: FunTable::new(),
        }
    }

    pub fn parse(&mut self) {
        self.program();
    }

    // --- Token helpers ---

    fn match_tag(&mut self, tag: &TokenTag) -> bool {
        self.match_offset(0, tag)
    }

    fn match_offset(&mut self, offset: usize, tag: &TokenTag) -> bool {
        let pos = self.cursor + offset;
        if pos < self.tokens.len() && self.tokens[pos].tag == TokenTag::LineEnd {
            self.cursor += 1;
            self.numlines += 1;
            return self.match_tag(tag);
        }
        pos < self.tokens.len() && self.tokens[pos].tag == *tag
    }

    fn eat(&mut self, tag: &TokenTag) -> String {
        // Skip line ends
        while self.cursor < self.tokens.len()
            && self.tokens[self.cursor].tag == TokenTag::LineEnd
        {
            self.cursor += 1;
            self.numlines += 1;
        }
        if self.cursor >= self.tokens.len() {
            eprintln!("parse error at line {}", self.numlines);
            std::process::exit(1);
        }
        let tok = &self.tokens[self.cursor];
        let val = tok.value.clone();
        writeln!(self.output, "\t{}\t{}", val, tag.name()).ok();
        if tok.tag != *tag {
            eprintln!("parse error at line {}: expected {:?}, got {:?}", self.numlines, tag, tok.tag);
            std::process::exit(1);
        }
        self.cursor += 1;
        val
    }

    // --- Grammar productions ---

    fn program(&mut self) {
        writeln!(self.output, "enter program").ok();
        self.const_data();
        self.variable_data();
        while (self.match_tag(&TokenTag::Void)
            || self.match_tag(&TokenTag::Int)
            || self.match_tag(&TokenTag::Char))
            && self.match_offset(1, &TokenTag::ID)
        {
            // Peek at next ID value; stop at "main"
            let next_val = self.peek_value(1);
            if next_val == "main" {
                break;
            }
            self.function();
        }
        self.main_function();
        writeln!(self.output, "exit program").ok();
    }

    fn peek_value(&self, offset: usize) -> String {
        let pos = self.cursor + offset;
        if pos < self.tokens.len() {
            self.tokens[pos].value.clone()
        } else {
            String::new()
        }
    }

    fn function(&mut self) {
        writeln!(self.output, "enter function").ok();
        let tp = if self.match_tag(&TokenTag::Void) {
            self.eat(&TokenTag::Void);
            Type::Void
        } else {
            self.parse_type()
        };
        let id = self.eat(&TokenTag::ID);
        let attr = FunAttr { ty: tp, args: vec![] };
        self.ftable.addsym(id, attr);

        self.vtable.enter_scope();
        writeln!(self.output, "enter a new scope").ok();

        self.eat(&TokenTag::LParent);
        self.args();
        self.eat(&TokenTag::RParent);
        self.block(true);
        writeln!(self.output, "exit function").ok();
    }

    fn main_function(&mut self) {
        writeln!(self.output, "enter main_function").ok();
        self.eat(&TokenTag::Void);
        self.eat(&TokenTag::ID);

        let attr = FunAttr { ty: Type::Void, args: vec![] };
        self.ftable.addsym("main".to_string(), attr);

        self.eat(&TokenTag::LParent);
        self.eat(&TokenTag::RParent);
        self.block(false);
        writeln!(self.output, "exit main_function").ok();
    }

    fn block(&mut self, is_scoped: bool) {
        writeln!(self.output, "enter block").ok();
        self.eat(&TokenTag::LBrace);
        if !is_scoped {
            self.vtable.enter_scope();
            writeln!(self.output, "enter a new scope").ok();
        }
        self.const_data();
        self.variable_data();
        self.stmts();
        self.eat(&TokenTag::RBrace);
        self.vtable.exit_scope();
        writeln!(self.output, "exit the current scope").ok();
        writeln!(self.output, "exit block").ok();
    }

    fn const_data(&mut self) {
        writeln!(self.output, "enter const_data").ok();
        while self.match_tag(&TokenTag::Const) {
            self.const_declaration();
            self.eat(&TokenTag::Semicn);
        }
        writeln!(self.output, "exit const_data").ok();
    }

    fn const_declaration(&mut self) {
        writeln!(self.output, "enter const_declaration").ok();
        self.eat(&TokenTag::Const);
        let tp = self.parse_type();
        let id = self.eat(&TokenTag::ID);
        self.eat(&TokenTag::Assign);
        let value = self.atom_value();
        let attr = VarAttr { ty: tp.clone(), qual: Qual::Const, value, var: id.clone() };
        self.vtable.addsym(id, attr);

        while self.match_tag(&TokenTag::Comma) {
            self.eat(&TokenTag::Comma);
            let id2 = self.eat(&TokenTag::ID);
            self.eat(&TokenTag::Assign);
            let val2 = self.atom_value();
            let attr2 = VarAttr { ty: tp.clone(), qual: Qual::Const, value: val2, var: id2.clone() };
            self.vtable.addsym(id2, attr2);
        }
        writeln!(self.output, "exit const_declaration").ok();
    }

    fn variable_data(&mut self) {
        writeln!(self.output, "enter variable_data").ok();
        while (self.match_tag(&TokenTag::Int) || self.match_tag(&TokenTag::Char))
            && !self.match_offset(2, &TokenTag::LParent)
        {
            self.variable_declaration();
            self.eat(&TokenTag::Semicn);
        }
        writeln!(self.output, "exit variable_data").ok();
    }

    fn variable_declaration(&mut self) {
        writeln!(self.output, "enter variable_declaration").ok();
        let tp = self.parse_type();
        let id = self.eat(&TokenTag::ID);
        let attr = VarAttr { ty: tp.clone(), qual: Qual::None, value: 0, var: id.clone() };
        self.vtable.addsym(id, attr);

        while self.match_tag(&TokenTag::Comma) {
            self.eat(&TokenTag::Comma);
            let id2 = self.eat(&TokenTag::ID);
            let attr2 = VarAttr { ty: tp.clone(), qual: Qual::None, value: 0, var: id2.clone() };
            self.vtable.addsym(id2, attr2);
        }
        writeln!(self.output, "exit variable_declaration").ok();
    }

    fn stmts(&mut self) {
        writeln!(self.output, "enter stmts").ok();
        loop {
            if self.match_tag(&TokenTag::RBrace) {
                break;
            }
            if self.cursor >= self.tokens.len() {
                break;
            }
            self.stmt();
        }
        writeln!(self.output, "exit stmts").ok();
    }

    fn stmt(&mut self) {
        writeln!(self.output, "enter stmt").ok();
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
            self.call();
            self.eat(&TokenTag::Semicn);
        } else if self.match_tag(&TokenTag::ID) {
            self.assign();
            self.eat(&TokenTag::Semicn);
        }
        writeln!(self.output, "exit stmt").ok();
    }

    fn assign(&mut self) {
        writeln!(self.output, "enter assign").ok();
        self.eat(&TokenTag::ID);
        self.eat(&TokenTag::Assign);
        self.expr();
        writeln!(self.output, "exit assign").ok();
    }

    fn call(&mut self) -> Type {
        writeln!(self.output, "enter call").ok();
        let id = self.eat(&TokenTag::ID);
        self.eat(&TokenTag::LParent);
        self.values();
        self.eat(&TokenTag::RParent);
        let ret_type = self.ftable.getattr(&id)
            .map(|f| f.ty.clone())
            .unwrap_or(Type::Void);
        writeln!(self.output, "exit call").ok();
        ret_type
    }

    fn if_condition(&mut self) {
        writeln!(self.output, "enter if_condition").ok();
        self.eat(&TokenTag::If);
        self.eat(&TokenTag::LParent);
        self.condition();
        self.eat(&TokenTag::RParent);
        self.block(false);
        if self.match_tag(&TokenTag::Else) {
            self.eat(&TokenTag::Else);
            self.block(false);
        }
        writeln!(self.output, "exit if_condition").ok();
    }

    fn for_loop(&mut self) {
        writeln!(self.output, "enter for_loop").ok();
        self.eat(&TokenTag::For);
        self.eat(&TokenTag::LParent);
        self.eat(&TokenTag::ID);
        self.eat(&TokenTag::Assign);
        self.atom_value();
        self.eat(&TokenTag::Semicn);
        self.condition();
        self.eat(&TokenTag::Semicn);
        self.eat(&TokenTag::ID);
        self.eat(&TokenTag::Assign);
        self.expr();
        self.eat(&TokenTag::RParent);
        self.block(false);
        writeln!(self.output, "exit for_loop").ok();
    }

    fn while_loop(&mut self) {
        writeln!(self.output, "enter while_loop").ok();
        self.eat(&TokenTag::While);
        self.eat(&TokenTag::LParent);
        self.condition();
        self.eat(&TokenTag::RParent);
        self.block(false);
        writeln!(self.output, "exit while_loop").ok();
    }

    fn dowhile_loop(&mut self) {
        writeln!(self.output, "enter dowhile_loop").ok();
        self.eat(&TokenTag::Do);
        self.block(false);
        self.eat(&TokenTag::While);
        self.eat(&TokenTag::LParent);
        self.condition();
        self.eat(&TokenTag::RParent);
        self.eat(&TokenTag::Semicn);
        writeln!(self.output, "exit dowhile_loop").ok();
    }

    fn read(&mut self) {
        writeln!(self.output, "enter read").ok();
        self.eat(&TokenTag::Scanf);
        self.eat(&TokenTag::LParent);
        self.eat(&TokenTag::STR);
        self.eat(&TokenTag::Comma);
        self.eat(&TokenTag::ID);
        self.eat(&TokenTag::RParent);
        writeln!(self.output, "exit read").ok();
    }

    fn print(&mut self) {
        writeln!(self.output, "enter print").ok();
        self.eat(&TokenTag::Printf);
        self.eat(&TokenTag::LParent);
        self.eat(&TokenTag::STR);
        while self.match_tag(&TokenTag::Comma) {
            self.eat(&TokenTag::Comma);
            self.expr();
        }
        self.eat(&TokenTag::RParent);
        writeln!(self.output, "exit print").ok();
    }

    fn ret(&mut self) {
        writeln!(self.output, "enter return").ok();
        self.eat(&TokenTag::Return);
        if !self.match_tag(&TokenTag::Semicn) {
            self.expr();
        }
        writeln!(self.output, "exit return").ok();
    }

    fn expr(&mut self) -> Type {
        let mut tp = self.term();
        while self.match_tag(&TokenTag::Add) || self.match_tag(&TokenTag::Sub) {
            if self.match_tag(&TokenTag::Add) {
                self.eat(&TokenTag::Add);
            } else {
                self.eat(&TokenTag::Sub);
            }
            self.term();
            tp = Type::Int;
        }
        tp
    }

    fn term(&mut self) -> Type {
        let mut tp = self.factor();
        while self.match_tag(&TokenTag::Mul) || self.match_tag(&TokenTag::Div) {
            if self.match_tag(&TokenTag::Mul) {
                self.eat(&TokenTag::Mul);
            } else {
                self.eat(&TokenTag::Div);
            }
            self.factor();
            tp = Type::Int;
        }
        tp
    }

    fn factor(&mut self) -> Type {
        if self.match_tag(&TokenTag::LParent) {
            self.eat(&TokenTag::LParent);
            let tp = self.expr();
            self.eat(&TokenTag::RParent);
            tp
        } else if self.match_tag(&TokenTag::ID) && self.match_offset(1, &TokenTag::LParent) {
            self.call()
        } else if self.match_tag(&TokenTag::ID) {
            let id = self.eat(&TokenTag::ID);
            self.vtable.getattr(&id).map(|v| v.ty.clone()).unwrap_or(Type::Int)
        } else if self.match_tag(&TokenTag::NUM) {
            self.eat(&TokenTag::NUM);
            Type::Int
        } else if self.match_tag(&TokenTag::CHR) {
            self.eat(&TokenTag::CHR);
            Type::Char
        } else {
            self.eat(&TokenTag::NUM); // consume whatever is next
            Type::Int
        }
    }

    fn args(&mut self) -> Vec<Type> {
        writeln!(self.output, "enter args").ok();
        let mut arg_types = vec![];
        if !self.match_tag(&TokenTag::RParent) {
            let tp = self.parse_type();
            let id = self.eat(&TokenTag::ID);
            let attr = VarAttr { ty: tp.clone(), qual: Qual::None, value: 0, var: id.clone() };
            self.vtable.addsym(id, attr);
            arg_types.push(tp);
            while self.match_tag(&TokenTag::Comma) {
                self.eat(&TokenTag::Comma);
                let tp2 = self.parse_type();
                let id2 = self.eat(&TokenTag::ID);
                let attr2 = VarAttr { ty: tp2.clone(), qual: Qual::None, value: 0, var: id2.clone() };
                self.vtable.addsym(id2, attr2);
                arg_types.push(tp2);
            }
        }
        writeln!(self.output, "exit args").ok();
        arg_types
    }

    fn values(&mut self) {
        writeln!(self.output, "enter values").ok();
        if !self.match_tag(&TokenTag::RParent) {
            self.expr();
            while self.match_tag(&TokenTag::Comma) {
                self.eat(&TokenTag::Comma);
                self.expr();
            }
        }
        writeln!(self.output, "exit values").ok();
    }

    fn condition(&mut self) {
        writeln!(self.output, "enter condition").ok();
        self.expr();
        let cmp_tags = [
            TokenTag::GE, TokenTag::GT, TokenTag::LE,
            TokenTag::LT, TokenTag::EQ, TokenTag::NE,
        ];
        let found = cmp_tags.iter().any(|t| self.match_tag(t));
        if found {
            // eat whichever comparator is present
            for t in &cmp_tags {
                if self.match_tag(t) {
                    self.eat(t);
                    break;
                }
            }
            self.expr();
        }
        writeln!(self.output, "exit condition").ok();
    }

    fn parse_type(&mut self) -> Type {
        if self.match_tag(&TokenTag::Int) {
            self.eat(&TokenTag::Int);
            Type::Int
        } else if self.match_tag(&TokenTag::Char) {
            self.eat(&TokenTag::Char);
            Type::Char
        } else {
            self.eat(&TokenTag::Void);
            Type::Void
        }
    }

    fn atom_value(&mut self) -> i32 {
        if self.match_tag(&TokenTag::Add) {
            self.eat(&TokenTag::Add);
        } else if self.match_tag(&TokenTag::Sub) {
            self.eat(&TokenTag::Sub);
            let s = self.eat(&TokenTag::NUM);
            return -s.parse::<i32>().unwrap_or(0);
        }
        if self.match_tag(&TokenTag::NUM) {
            let s = self.eat(&TokenTag::NUM);
            s.parse::<i32>().unwrap_or(0)
        } else if self.match_tag(&TokenTag::CHR) {
            let s = self.eat(&TokenTag::CHR);
            s.chars().next().map(|c| c as i32).unwrap_or(0)
        } else {
            0
        }
    }
}
