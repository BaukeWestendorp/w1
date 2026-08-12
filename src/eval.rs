use std::collections::HashMap;

use crate::ast;

pub fn eval<'src>(ast: &'src ast::Ast<'src>) -> String {
    Evaluator::new(ast).eval()
}

struct Evaluator<'src> {
    scope: Scope<'src>,
}

impl<'src> Evaluator<'src> {
    pub fn new(ast: &'src ast::Ast<'src>) -> Self {
        Self {
            scope: Scope::new_root(&ast.root),
        }
    }

    pub fn eval(mut self) -> String {
        let mut output = String::new();
        self.scope.eval(&mut output);
        output
    }
}

struct Scope<'src> {
    parent: Option<&'src Scope<'src>>,
    block: &'src ast::Block<'src>,
    definitions: HashMap<&'src ast::Variable<'src>, &'src ast::ListDefinition<'src>>,
    macros: HashMap<&'src str, &'src ast::Macro<'src>>,
    variables: HashMap<&'src ast::Variable<'src>, &'src ast::Block<'src>>,
}

impl<'src> Scope<'src> {
    pub fn new(block: &'src ast::Block<'src>, parent: &'src Self) -> Self {
        let mut this = Self::new_root(block);
        this.parent = Some(parent);
        this
    }

    pub fn new_root(block: &'src ast::Block<'src>) -> Self {
        Self {
            parent: None,
            block,
            definitions: HashMap::new(),
            macros: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    pub fn eval(&mut self, output: &mut String) {
        let block = self.block;
        for node in &block.nodes {
            self.eval_node(node, output);
        }
    }

    fn eval_node(&mut self, node: &'src ast::Node<'src>, output: &mut String) {
        match node {
            ast::Node::Define(define) => self.eval_define(define),
            ast::Node::For(r#for) => self.eval_for(r#for, output),
            ast::Node::Macro(r#macro) => self.eval_macro(r#macro),
            ast::Node::Call(call) => self.eval_call(call, output),
            ast::Node::Variable(variable) => self.eval_variable(variable, output),
            ast::Node::Text(text) => output.push_str(text),
            _ => todo!(),
        }
    }

    fn eval_define(&mut self, define: &'src ast::Define<'src>) {
        self.definitions.insert(&define.name, &define.list);
    }

    fn eval_for(&mut self, r#for: &'src ast::For<'src>, output: &mut String) {
        let list = self.get_definition(&r#for.list);

        for object in &list.objects {
            let mut scope = Scope::new(&r#for.template, self);

            for field_name in &r#for.fields {
                let field = object
                    .fields
                    .iter()
                    .find(|field| field.name == *field_name)
                    .unwrap_or_else(|| {
                        error(&format!(
                            "missing field '{}' in list '{}'",
                            field_name.name, r#for.list.name
                        ))
                    });

                scope.variables.insert(field_name, &field.value);
            }

            scope.eval(output);
        }
    }

    fn eval_macro(&mut self, r#macro: &'src ast::Macro<'src>) {
        self.macros.insert(r#macro.name, r#macro);
    }

    fn eval_call(&mut self, call: &ast::Call<'src>, output: &mut String) {
        let r#macro = self.get_macro(call.name);

        let mut call_output = String::new();
        let mut scope = Scope::new(&r#macro.template, self);

        for (param, arg) in r#macro.parameters.iter().zip(call.arguments.iter()) {
            scope.variables.insert(param, arg);
        }

        scope.eval(&mut call_output);
        output.push_str(&call_output);
    }

    fn eval_variable(&mut self, variable: &ast::Variable<'src>, output: &mut String) {
        let block_to_expand = self.get_variable(variable);
        let mut scope = Scope::new(&block_to_expand, self);
        scope.eval(output);
    }

    pub fn get_definition(&self, name: &ast::Variable<'src>) -> &ast::ListDefinition<'src> {
        if let Some(definition) = self.definitions.get(name) {
            definition
        } else if let Some(parent) = self.parent {
            parent.get_definition(name)
        } else {
            error(&format!("undefined list: {}", name.name));
        }
    }

    pub fn get_macro(&self, name: &str) -> &ast::Macro<'src> {
        if let Some(r#macro) = self.macros.get(name) {
            r#macro
        } else if let Some(parent) = self.parent {
            parent.get_macro(name)
        } else {
            error(&format!("undefined macro: {}", name));
        }
    }

    pub fn get_variable(&self, variable: &ast::Variable<'src>) -> &ast::Block<'src> {
        if let Some(block) = self.variables.get(variable) {
            block
        } else if let Some(parent) = self.parent {
            parent.get_variable(variable)
        } else {
            error(&format!("undefined variable: {}", variable.name));
        }
    }
}

fn error(message: &str) -> ! {
    panic!("evaluation error: {}", message);
}
