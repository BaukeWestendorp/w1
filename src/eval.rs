use std::collections::HashMap;

use crate::ast::{self};

pub fn eval<'src>(ast: ast::Ast<'src>) -> String {
    Evaluator::new(ast).eval()
}

struct Evaluator<'src> {
    ast: ast::Ast<'src>,

    definitions: HashMap<&'src str, ast::Definition<'src>>,
}

impl<'src> Evaluator<'src> {
    pub fn new(ast: ast::Ast<'src>) -> Self {
        Self {
            ast,
            definitions: HashMap::new(),
        }
    }

    pub fn eval(mut self) -> String {
        self.collect_definitions();

        let mut output = String::new();
        for node in &self.ast.nodes {
            self.eval_node(node, &mut output);
        }
        output
    }

    fn eval_invokation(&self, invokation: &ast::Invokation<'src>, output: &mut String) {
        if let Some(definition) = self.definitions.get(invokation.name) {
            let mut invokation_output = String::new();

            for node in &definition.template.nodes {
                match node {
                    ast::Node::Text(text) => invokation_output.push_str(text),
                    ast::Node::Variable(ast::Variable { name }) => {
                        let Some(param_ix) = definition
                            .params
                            .iter()
                            .position(|param| param.name == *name)
                        else {
                            panic!("undefined variable: {}", name);
                        };

                        let Some(arg) = invokation.args.get(param_ix) else {
                            panic!("missing argument for parameter: {}", name);
                        };

                        for node in &arg.nodes {
                            self.eval_node(node, &mut invokation_output);
                        }
                    }
                    ast::Node::Definition(_) => todo!("FIXME: implement nested definitions"),
                    ast::Node::Invokation(_) => todo!("FIXME: implement nested invokations"),
                }
            }

            output.push_str(&invokation_output);
        } else {
            panic!("undefined invokation: {}", invokation.name);
        }
    }

    fn eval_node(&self, node: &ast::Node<'src>, output: &mut String) {
        match node {
            ast::Node::Definition(_) => {}
            ast::Node::Invokation(invokation) => self.eval_invokation(invokation, output),
            ast::Node::Variable(variable) => {
                panic!("unexpected variable node at top level: {variable:?}")
            }
            ast::Node::Text(text) => output.push_str(text),
        }
    }

    fn collect_definitions(&mut self) {
        self.definitions.clear();

        // FIXME: Definitions should be scoped.
        for node in &self.ast.nodes {
            if let ast::Node::Definition(definition) = node {
                if self.definitions.contains_key(definition.name) {
                    panic!("duplicate definition: {}", definition.name);
                } else {
                    self.definitions.insert(definition.name, definition.clone());
                }
            }
        }
    }
}
