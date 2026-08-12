#[test]
fn basic() {
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

    let tokens = w1::token::tokenize(INPUT);
    let ast = w1::ast::parse(INPUT, tokens);
    let output = w1::eval::eval(&ast);
    similar_asserts::assert_eq!(EXPECTED_OUTPUT, output);
}

#[test]
fn nested_invokation() {
    const INPUT: &str = r#"
@macro button($label, $action) {{<button onclick="$action">$label</button>}}

@macro modal($title, $content, $button_label) {{
<div class="modal">
    <h2>$title</h2>
    <div class="modal-content">
        $content
    </div>
    <div class="modal-footer">
        @button({{$button_label}}, {{closeModal()}})
    </div>
</div>
}}

@modal({{Warning}}, {{<p>Are you sure you want to delete this?</p>}}, {{Confirm}})
"#;

    const EXPECTED_OUTPUT: &str = r#"





<div class="modal">
    <h2>Warning</h2>
    <div class="modal-content">
        <p>Are you sure you want to delete this?</p>
    </div>
    <div class="modal-footer">
        <button onclick="closeModal()">Confirm</button>
    </div>
</div>

"#;

    let tokens = w1::token::tokenize(INPUT);
    let ast = w1::ast::parse(INPUT, tokens);
    let output = w1::eval::eval(&ast);
    similar_asserts::assert_eq!(EXPECTED_OUTPUT, output);
}

#[test]
fn nested_macro_definition() {
    const INPUT: &str = r#"
@macro outer($prefix) {{
@macro inner($text) {{$prefix: $text}}
@inner({{Hello World}})
}}

@outer({{INFO}})
"#;

    const EXPECTED_OUTPUT: &str = r#"




INFO: Hello World

"#;

    let tokens = w1::token::tokenize(INPUT);
    let ast = w1::ast::parse(INPUT, tokens);
    let output = w1::eval::eval(&ast);
    similar_asserts::assert_eq!(EXPECTED_OUTPUT, output);
}
