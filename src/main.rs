use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "A text processing macro language")]
struct Args {
    /// Path to the source file to parse. If omitted or "-", reads from stdin.
    #[arg(value_name = "INPUT")]
    input: Option<String>,

    /// Path to the output file. If omitted, writes to stdout.
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Skip building AST evaluation and output the parsed tokens for debugging.
    #[arg(long)]
    show_tokens: bool,

    /// Skip evaluation and output the parsed AST for debugging.
    #[arg(long)]
    show_ast: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut src = String::new();
    match args.input.as_deref() {
        Some("-") | None => {
            io::stdin().read_to_string(&mut src)?;
        }
        Some(path) => {
            src = fs::read_to_string(path)?;
        }
    }

    if args.show_tokens {
        for token in w1::token::tokenize(&src) {
            println!("{:?}", token);
        }

        return Ok(());
    }

    if args.show_ast {
        let tokens = w1::token::tokenize(&src);
        let ast = w1::ast::parse(tokens);
        println!("{:#?}", ast);
        return Ok(());
    }

    let output = w1::eval(&src);

    if let Some(output_path) = args.output {
        fs::write(output_path, output)?;
    } else {
        io::stdout().write_all(output.as_bytes())?;
    }

    Ok(())
}
