pub mod ast;
pub mod eval;
pub mod token;

pub fn eval<'src>(src: &'src str) -> String {
    let tokens = token::tokenize(&src);
    let ast = ast::parse(src, tokens);
    eval::eval(ast)
}
