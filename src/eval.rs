use std::collections::HashMap;

use crate::ast::{self};

pub fn eval<'src>(ast: ast::Ast<'src>) -> String {
    Evaluator::new(ast).eval()
}

struct Evaluator<'src> {
    ast: ast::Ast<'src>,

    macros: HashMap<&'src str, ast::Macro<'src>>,
}

impl<'src> Evaluator<'src> {
    pub fn new(ast: ast::Ast<'src>) -> Self {
        Self {
            ast,
            macros: HashMap::new(),
        }
    }

    pub fn eval(mut self) -> String {
        self.collect_macros();

        let mut output = String::new();
        for node in &self.ast.nodes {
            self.eval_node(node, &mut output);
        }
        output
    }

    fn eval_invokation(&self, invokation: &ast::Invokation<'src>, output: &mut String) {
        if let Some(r#macro) = self.macros.get(invokation.name) {
            let mut invokation_output = String::new();

            for node in &r#macro.template.nodes {
                match node {
                    ast::Node::Text(text) => invokation_output.push_str(text),
                    ast::Node::Variable(ast::Variable { name }) => {
                        let Some(param_ix) =
                            r#macro.params.iter().position(|param| param.name == *name)
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
                    ast::Node::Macro(_) => todo!("FIXME: implement nested macros"),
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
            ast::Node::Macro(_) => {}
            ast::Node::Invokation(invokation) => self.eval_invokation(invokation, output),
            ast::Node::Variable(variable) => {
                panic!("unexpected variable node at top level: {variable:?}")
            }
            ast::Node::Text(text) => output.push_str(text),
        }
    }

    fn collect_macros(&mut self) {
        self.macros.clear();

        // FIXME: Macros should be scoped.
        for node in &self.ast.nodes {
            if let ast::Node::Macro(r#macro) = node {
                if self.macros.contains_key(r#macro.name) {
                    panic!("duplicate macro: {}", r#macro.name);
                } else {
                    self.macros.insert(r#macro.name, r#macro.clone());
                }
            }
        }
    }
}
