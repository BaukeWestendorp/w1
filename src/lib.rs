pub mod ast;
pub mod token;

pub fn eval<'src>(src: &'src str) -> String {
    let tokens = token::tokenize(&src);
    let _ast = ast::parse(tokens);
    "FIXME: EVALUATE".to_string()
}
