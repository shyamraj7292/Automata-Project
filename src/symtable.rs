use std::collections::HashMap;

/// Corresponds to C++ enum Type
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    Int,
    Char,
}

impl Type {
    pub fn name(&self) -> &'static str {
        match self {
            Type::Void => "void",
            Type::Int  => "int",
            Type::Char => "char",
        }
    }
}

/// Corresponds to C++ enum Qual
#[derive(Debug, Clone, PartialEq)]
pub enum Qual {
    None,
    Const,
}

/// Variable attribute (VarAttr in C++)
#[derive(Debug, Clone)]
pub struct VarAttr {
    pub ty: Type,
    pub qual: Qual,
    pub value: i32,
    pub var: String,
}

impl Default for VarAttr {
    fn default() -> Self {
        VarAttr { ty: Type::Void, qual: Qual::None, value: 0, var: String::new() }
    }
}

impl VarAttr {
    pub fn to_string(&self) -> String {
        let mut s = self.ty.name().to_string();
        if self.qual == Qual::Const {
            s += &format!("\tconst\t{}\t{}", self.value, self.var);
        }
        s
    }
}

/// Function attribute (FunAttr in C++)
#[derive(Debug, Clone)]
pub struct FunAttr {
    pub ty: Type,
    pub args: Vec<Type>,
}

impl FunAttr {
    pub fn to_string(&self) -> String {
        let mut s = self.ty.name().to_string();
        for arg in &self.args {
            s += &format!("\t{}", arg.name());
        }
        s
    }
}

/// Variable symbol table with scope support
pub struct VarTable {
    pub allsyms: HashMap<String, VarAttr>,
    pub cursyms: HashMap<String, VarAttr>,
    // scope stack: None = scope boundary, Some = (name, old_value)
    modify: Vec<Option<(String, Option<VarAttr>)>>,
}

impl VarTable {
    pub fn new() -> Self {
        VarTable {
            allsyms: HashMap::new(),
            cursyms: HashMap::new(),
            modify: Vec::new(),
        }
    }

    pub fn addsym(&mut self, name: String, attr: VarAttr) {
        let old = self.allsyms.get(&name).cloned();
        self.modify.push(Some((name.clone(), old)));
        self.allsyms.insert(name.clone(), attr.clone());
        self.cursyms.insert(name, attr);
    }

    pub fn hassym(&self, name: &str) -> bool {
        self.allsyms.contains_key(name)
    }

    pub fn getattr(&self, name: &str) -> Option<&VarAttr> {
        self.allsyms.get(name)
    }

    pub fn enter_scope(&mut self) {
        self.cursyms.clear();
        self.modify.push(None); // scope boundary
    }

    pub fn exit_scope(&mut self) {
        // Unwind until scope boundary
        loop {
            match self.modify.pop() {
                None => break, // no more stack
                Some(None) => break, // found scope boundary
                Some(Some((name, old))) => {
                    match old {
                        None => { self.allsyms.remove(&name); }
                        Some(v) => { self.allsyms.insert(name, v); }
                    }
                }
            }
        }
        self.cursyms.clear();
        // Rebuild cursyms from remaining modify stack (current scope)
        for entry in self.modify.iter().rev() {
            match entry {
                None => break,
                Some((name, _)) => {
                    if let Some(a) = self.allsyms.get(name) {
                        self.cursyms.insert(name.clone(), a.clone());
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.allsyms.clear();
        self.cursyms.clear();
        self.modify.clear();
    }
}

/// Function symbol table
pub struct FunTable {
    pub syms: HashMap<String, FunAttr>,
}

impl FunTable {
    pub fn new() -> Self {
        FunTable { syms: HashMap::new() }
    }

    pub fn addsym(&mut self, name: String, attr: FunAttr) {
        self.syms.insert(name, attr);
    }

    pub fn hassym(&self, name: &str) -> bool {
        self.syms.contains_key(name)
    }

    pub fn getattr(&self, name: &str) -> Option<&FunAttr> {
        self.syms.get(name)
    }
}
