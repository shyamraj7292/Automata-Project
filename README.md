# Automata Mini-Compiler

A mini-compiler written entirely in **Rust**, built as a compiler design course project.
It implements a complete compilation pipeline — from lexical analysis all the way through
to MIPS assembly output — with an optional optimisation pass.

---

## Table of Contents

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

This project is a from-scratch mini-compiler targeting a simple custom source language,
producing MIPS assembly as output. Every phase of compilation — tokenising, parsing,
symbol-table management, IR generation, optimisation, and MIPS emission — lives in its
own focused Rust module.

---

## Features

- Lexical analysis (hand-written tokeniser)
- Recursive-descent parser with full grammar coverage
- Scoped symbol table for variables and functions
- Intermediate representation (IR) code generation
- Constant-folding and dead-code-elimination optimisation pass
- MIPS assembly backend with basic register allocation
- Descriptive parse / codegen error reporting

---

## Project Structure

```
automata-mini-compiler/
├── src/
│   ├── main.rs        # Entry point — wires the pipeline together
│   ├── token.rs       # TokenTag enum + Token struct
│   ├── lexer.rs       # Tokeniser / lexer
│   ├── parser.rs      # Recursive-descent parser → syntaxtree.txt
│   ├── symtable.rs    # Symbol table (VarTable + FunTable) with scopes
│   ├── gencode.rs     # IR code generator → data.txt + code.txt
│   ├── optimize.rs    # Constant folding + dead-code elimination
│   └── mips.rs        # MIPS assembly emitter → mips_out.asm
├── tests/
│   └── integration_tests.rs
├── tools/
│   └── test.sh        # Batch test runner (uses MARS simulator)
├── test/
│   ├── in/            # Sample source files
│   └── out/           # Expected MIPS outputs
├── Cargo.toml
├── grammars.pdf
└── README.md
```

---

## Grammar

The formal grammar for the supported source language is documented in
[`grammars.pdf`](./grammars.pdf).

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) stable (1.75+)
- Cargo (bundled with Rust)

Install via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Building

```bash
# Clone the repository
git clone https://github.com/shyamraj7292/Automata-Project.git
cd Automata-Project

# Debug build
cargo build

# Release build (optimised)
cargo build --release
```

### Running

```bash
# Compile a source file (no optimisation)
cargo run --release -- <source_file>

# Compile with the optimisation pass enabled
cargo run --release -- <source_file> opt

# Examples
cargo run --release -- test/in/fact.txt
cargo run --release -- test/in/merge_sort.txt opt
```

After a successful run the MIPS assembly is written to `mips_out.asm`.

### Running Tests

```bash
cargo test
```

---

## Pipeline

```
Source File
    │
    ▼
┌──────────┐
│  Lexer   │  token.rs / lexer.rs
│          │  Converts raw source into a token stream
└────┬─────┘
     │
     ▼
┌──────────┐
│  Parser  │  parser.rs
│          │  Recursive-descent; validates structure; writes syntaxtree.txt
└────┬─────┘
     │
     ▼
┌────────────────┐
│  Symbol Table  │  symtable.rs
│                │  Scoped variable + function tables
└──────┬─────────┘
       │
       ▼
┌──────────────────┐
│  Code Generation │  gencode.rs
│                  │  Emits three-address IR to code.txt / data.txt
└───────┬──────────┘
        │
        ▼
┌────────────┐
│  Optimiser │  optimize.rs
│            │  Constant folding + dead-code elimination → opt_code.txt
└─────┬──────┘
      │
      ▼
┌──────────────┐
│ MIPS Backend │  mips.rs
│              │  Register allocation + MIPS assembly → mips_out.asm
└──────────────┘
```

---

## License

This project is licensed under the [MIT License](./LICENSE).