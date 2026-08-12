#[test]
fn basic() {
    const INPUT: &str = r#"
@define $posts = [
    [ $id = {{1}}, $title = {{Post 1}} ],
    [ $id = {{2}}, $title = {{Post 2}} ],
    [ $id = {{3}}, $title = {{Post 3}} ],
    [ $id = {{4}}, $title = {{Post 4}} ],
    [ $id = {{5}}, $title = {{Post 5}} ],
]

@for [$id, $title] in $posts {{
    <a href="/posts/$id">$title</a>
}}
"#;

    const EXPECTED_OUTPUT: &str = r#"



    <a href="/posts/1">Post 1</a>

    <a href="/posts/2">Post 2</a>

    <a href="/posts/3">Post 3</a>

    <a href="/posts/4">Post 4</a>

    <a href="/posts/5">Post 5</a>

"#;

    let tokens = w1::token::tokenize(INPUT);
    let ast = w1::ast::parse(INPUT, tokens);
    let output = w1::eval::eval(&ast);
    similar_asserts::assert_eq!(EXPECTED_OUTPUT, output);
}
