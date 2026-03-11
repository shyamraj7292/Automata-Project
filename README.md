# 🦀 Automata Mini-Compiler

A mini-compiler built entirely in **Rust**, developed as a compiler design course project. It implements a full compilation pipeline — from lexical analysis all the way to MIPS assembly code generation — with an integrated optimizer pass.

---

## 📚 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Project Structure](#project-structure)
- [Grammar](#grammar)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Building](#building)
  - [Running](#running)
  - [Running Tests](#running-tests)
- [Pipeline](#pipeline)
- [License](#license)

---

## Overview

This project is a from-scratch mini-compiler implemented in **Rust**. It takes a custom source language as input and produces MIPS assembly output. The compiler is split across clean, well-separated modules covering each phase of compilation.

---

## Features

- ✅ Lexical analysis (tokenizer / lexer)
- ✅ Recursive-descent parser
- ✅ Symbol table with scope management
- ✅ Intermediate code generation
- ✅ Optimization pass (constant folding, dead code elimination)
- ✅ MIPS assembly code generation
- ✅ Error handling with descriptive messages
- ✅ Full unit test coverage

---

## Project Structure

```
automata-mini-compiler/
├── src/
│   ├── main.rs           # Entry point
│   ├── lexer.rs          # Tokenizer / Lexer
│   ├── parser.rs         # Recursive-descent parser
│   ├── symtable.rs       # Symbol table
│   ├── gencode.rs        # Intermediate code generation
│   ├── optimize.rs       # Optimization pass
│   ├── mips.rs           # MIPS backend / code emitter
│   └── error_handle.rs   # Error handling utilities
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
├── grammars.pdf
└── README.md
```

---

## Grammar

The formal grammar for the supported source language is documented in [`grammars.pdf`](./grammars.pdf).

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain, 1.75+)
- Cargo (bundled with the Rust installation)

Install Rust via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building

```bash
# Clone the repository
git clone https://github.com/compiler-design-projects/minicompiler.git
cd minicompiler

# Build in release mode
cargo build --release
```

### Running

```bash
# Compile a source file
cargo run --release -- <source_file>

# Example
cargo run --release -- tests/sample.src
```

The compiled MIPS assembly will be printed to stdout (or written to an output file depending on flags).

### Running Tests

```bash
cargo test
```

---

## Pipeline

The compilation pipeline proceeds through the following stages:

```
Source Code
    │
    ▼
┌─────────┐
│  Lexer  │  ── Tokenizes raw input into a stream of tokens
└────┬────┘
     │
     ▼
┌─────────┐
│ Parser  │  ── Builds an AST via recursive descent
└────┬────┘
     │
     ▼
┌──────────────┐
│ Symbol Table │  ── Tracks identifiers, types, and scopes
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│ Code Generation  │  ── Emits intermediate representation (IR)
└───────┬──────────┘
        │
        ▼
┌────────────┐
│  Optimize  │  ── Applies IR-level optimizations
└─────┬──────┘
      │
      ▼
┌──────────────┐
│ MIPS Backend │  ── Translates IR to MIPS assembly
└──────────────┘
```

---

## License

This project is licensed under the [MIT License](./LICENSE).