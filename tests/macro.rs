const INPUT: &str = r#"
@macro card($title, $content) {{
    <div class="card">
        <h2>$title</h2>
        <div class="card-body">
            $content
        </div>
    </div>
}}

@card({{Welcome}}, {{<p>Lorem ipsum dolor, sit amet consectetur.</p>}})
"#;

const EXPECTED_OUTPUT: &str = r#"
    <div class="card">
        <h2>Welcome</h2>
        <div class="card-body">
            <p>Lorem ipsum dolor, sit amet consectetur.</p>
        </div>
    </div>
"#;

#[test]
fn main() {
    let tokens = w1::token::tokenize(INPUT);
    let ast = w1::ast::parse(tokens);
    let output = w1::eval::eval(ast);

    similar_asserts::assert_eq!(EXPECTED_OUTPUT, output);
}
