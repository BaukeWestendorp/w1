pub mod token;

fn main() {
    let src = std::fs::read_to_string(
        std::env::args()
            .nth(1)
            .expect("Please provide a source file path"),
    )
    .expect("Failed to read source file");

    let tokens = token::tokenize(&src).collect::<Vec<_>>();

    dbg!(tokens);

    // let parse_result = chumsky::Parser::parse(&parse::w1(), &source).into_result();

    // match parse_result {
    //     Ok(ast) => {
    //         for node in ast {
    //             println!("{node}");
    //         }
    //     }
    //     Err(errors) => {
    //         for error in errors {
    //             ariadne::Report::build(ariadne::ReportKind::Error, error.span().into_range())
    //                 .with_label(
    //                     ariadne::Label::new(error.span().into_range())
    //                         .with_message(error.reason().to_string()),
    //                 )
    //                 .with_message(error.to_string())
    //                 .finish()
    //                 .print(ariadne::Source::from(&source))
    //                 .unwrap();
    //         }
    //     }
    // }
}
