// Token types corresponding to the original C++ TokenTag enum
#[derive(Debug, Clone, PartialEq)]
pub enum TokenTag {
    // Literals
    NUM,
    ID,
    STR,
    CHR,
    // Keywords
    Const,
    Int,
    Char,
    Void,
    If,
    Else,
    Do,
    While,
    For,
    Scanf,
    Printf,
    Return,
    // Operators
    GE,   // >=
    GT,   // >
    LE,   // <=
    LT,   // <
    EQ,   // ==
    NE,   // !=
    Assign, // =
    Add,
    Sub,
    Mul,
    Div,
    // Punctuation
    Semicn,  // ;
    Comma,   // ,
    LParent, // (
    RParent, // )
    LBrack,  // [
    RBrack,  // ]
    LBrace,  // {
    RBrace,  // }
    LineEnd, // newline
}

impl TokenTag {
    pub fn name(&self) -> &'static str {
        match self {
            TokenTag::NUM => "NUM",
            TokenTag::ID => "ID",
            TokenTag::STR => "STR",
            TokenTag::CHR => "CHR",
            TokenTag::Const => "CONST",
            TokenTag::Int => "INT",
            TokenTag::Char => "CHAR",
            TokenTag::Void => "VOID",
            TokenTag::If => "IF",
            TokenTag::Else => "ELSE",
            TokenTag::Do => "DO",
            TokenTag::While => "WHILE",
            TokenTag::For => "FOR",
            TokenTag::Scanf => "SCANF",
            TokenTag::Printf => "PRINTF",
            TokenTag::Return => "RETURN",
            TokenTag::GE => "GE",
            TokenTag::GT => "GT",
            TokenTag::LE => "LE",
            TokenTag::LT => "LT",
            TokenTag::EQ => "EQ",
            TokenTag::NE => "NE",
            TokenTag::Assign => "ASSIGN",
            TokenTag::Add => "ADD",
            TokenTag::Sub => "SUB",
            TokenTag::Mul => "MUL",
            TokenTag::Div => "DIV",
            TokenTag::Semicn => "SEMICN",
            TokenTag::Comma => "COMMA",
            TokenTag::LParent => "LPARENT",
            TokenTag::RParent => "RPARENT",
            TokenTag::LBrack => "LBRACK",
            TokenTag::RBrack => "RBRACK",
            TokenTag::LBrace => "LBRACE",
            TokenTag::RBrace => "RBRACE",
            TokenTag::LineEnd => "LINEEND",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tag: TokenTag,
    pub value: String,
}

impl Token {
    pub fn new(tag: TokenTag, value: impl Into<String>) -> Self {
        Token { tag, value: value.into() }
    }
}
