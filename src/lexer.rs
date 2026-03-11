use crate::token::{Token, TokenTag};
use std::fs;

/// Tokenizes the given source file into a Vec of Tokens.
/// Equivalent to the C++ `lexer(string file)` function.
pub fn lexer(file: &str) -> Vec<Token> {
    let source = fs::read_to_string(file)
        .unwrap_or_else(|e| panic!("Failed to open source file '{}': {}", file, e));

    let mut tokens: Vec<Token> = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '\n' {
            tokens.push(Token::new(TokenTag::LineEnd, ""));
            i += 1;
        } else if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() {
            // Integer literal
            let mut num = String::new();
            while i < chars.len() && chars[i].is_ascii_digit() {
                num.push(chars[i]);
                i += 1;
            }
            tokens.push(Token::new(TokenTag::NUM, num));
        } else if c.is_alphabetic() || c == '_' {
            // Identifier or keyword
            let mut id = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                id.push(chars[i]);
                i += 1;
            }
            let tag = match id.as_str() {
                "const"  => TokenTag::Const,
                "int"    => TokenTag::Int,
                "char"   => TokenTag::Char,
                "void"   => TokenTag::Void,
                "if"     => TokenTag::If,
                "else"   => TokenTag::Else,
                "do"     => TokenTag::Do,
                "while"  => TokenTag::While,
                "for"    => TokenTag::For,
                "scanf"  => TokenTag::Scanf,
                "printf" => TokenTag::Printf,
                "return" => TokenTag::Return,
                _        => TokenTag::ID,
            };
            tokens.push(Token::new(tag, id));
        } else if c == '"' {
            // String literal
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != '"' {
                s.push(chars[i]);
                i += 1;
            }
            i += 1; // consume closing "
            tokens.push(Token::new(TokenTag::STR, s));
        } else if c == '\'' {
            // Character literal
            i += 1;
            let ch = chars[i];
            i += 1;
            i += 1; // consume closing '
            tokens.push(Token::new(TokenTag::CHR, ch.to_string()));
        } else if c == '>' {
            if i + 1 < chars.len() && chars[i + 1] == '=' {
                tokens.push(Token::new(TokenTag::GE, ">="));
                i += 2;
            } else {
                tokens.push(Token::new(TokenTag::GT, ">"));
                i += 1;
            }
        } else if c == '<' {
            if i + 1 < chars.len() && chars[i + 1] == '=' {
                tokens.push(Token::new(TokenTag::LE, "<="));
                i += 2;
            } else {
                tokens.push(Token::new(TokenTag::LT, "<"));
                i += 1;
            }
        } else if c == '=' {
            if i + 1 < chars.len() && chars[i + 1] == '=' {
                tokens.push(Token::new(TokenTag::EQ, "=="));
                i += 2;
            } else {
                tokens.push(Token::new(TokenTag::Assign, "="));
                i += 1;
            }
        } else if c == '!' {
            // Assume !=
            tokens.push(Token::new(TokenTag::NE, "!="));
            i += 2;
        } else {
            let (tag, val) = match c {
                ';' => (TokenTag::Semicn,  ";"),
                ',' => (TokenTag::Comma,   ","),
                '+' => (TokenTag::Add,     "+"),
                '-' => (TokenTag::Sub,     "-"),
                '*' => (TokenTag::Mul,     "*"),
                '/' => (TokenTag::Div,     "/"),
                '(' => (TokenTag::LParent, "("),
                ')' => (TokenTag::RParent, ")"),
                '[' => (TokenTag::LBrack,  "["),
                ']' => (TokenTag::RBrack,  "]"),
                '{' => (TokenTag::LBrace,  "{"),
                '}' => (TokenTag::RBrace,  "}"),
                _   => { i += 1; continue; }
            };
            tokens.push(Token::new(tag, val));
            i += 1;
        }
    }

    tokens
}
