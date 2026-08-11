use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

pub mod ast;
pub mod token;

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

    let tokens = token::tokenize(&src);

    if args.show_tokens {
        for token in tokens {
            println!("{:?}", token);
        }
        return Ok(());
    }

    let ast = ast::parse(tokens);

    if args.show_ast {
        println!("{:#?}", ast);
    }

    let evaluated_output = "FIXME: EVALUATE OUTPUT";

    if let Some(output_path) = args.output {
        fs::write(output_path, evaluated_output)?;
    } else {
        io::stdout().write_all(evaluated_output.as_bytes())?;
    }

    Ok(())
}
