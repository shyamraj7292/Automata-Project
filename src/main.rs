mod token;
mod lexer;
mod symtable;
mod parser;
mod gencode;
mod optimize;
mod mips;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Step 1: Lexical analysis
    let source_file = if args.len() >= 2 {
        args[1].as_str()
    } else {
        "testfile.txt"
    };

    let should_optimize = args.len() >= 3;

    println!("==> Lexing '{}'...", source_file);
    let tokens = lexer::lexer(source_file);

    // Step 2: Parsing
    println!("==> Parsing...");
    let mut parser = parser::Parser::new(&tokens);
    parser.parse();

    // Step 3: Intermediate code generation
    println!("==> Generating IR...");
    let mut codegen = gencode::Codegen::new(&tokens);
    codegen.generate();

    // Step 4: Optimization (optional)
    if should_optimize {
        println!("==> Optimizing...");
        let optimized = optimize::optimize(true);
        optimize::write_optimized(&optimized);
    }

    // Step 5: MIPS code generation
    println!("==> Generating MIPS assembly...");
    mips::mips(should_optimize);

    println!("==> Done! Output written to mips_out.asm");
}
