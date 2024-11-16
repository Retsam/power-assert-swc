use swc_core::ecma::ast::*;
use swc_core::ecma::visit::VisitMut;

/// This is the equivalent of babel-plugin-empower-assert - replaces imports for 'assert' with imports for 'power-assert'
pub struct ReplaceImportsVisitor;

impl VisitMut for ReplaceImportsVisitor {
    fn visit_mut_import_decl(&mut self, node: &mut ImportDecl) {
        if node.src.value == "assert" {
            *node.src = Str {
                value: "power-assert".into(),
                span: node.src.span,
                raw: None,
            };
        }
    }
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        let mut inner = || {
            let expr = node.callee.as_mut_expr()?;
            let call_id = expr.as_mut_ident()?;
            if call_id.sym != "require" {
                return None;
            }

            let call_arg = node.args.get_mut(0)?;
            let call_lit = call_arg.expr.as_mut_lit()?;
            if let Lit::Str(ref mut s) = call_lit {
                if s.value == "assert" {
                    *s = Str {
                        value: "power-assert".into(),
                        span: s.span,
                        raw: None,
                    };
                }
            }
            Some(())
        };
        inner();
    }
}

#[allow(unused)]
mod tests {
    use swc_core::ecma::{transforms::testing::test, visit::visit_mut_pass};

    use super::*;

    test!(
        Default::default(),
        |_| visit_mut_pass(ReplaceImportsVisitor {}),
        import_replace_test,
        // Input codes
        r#"
import assert from "assert";
import otherThing from "otherThing";

const assert2 = require("assert");
const otherThing2 = require("otherThing");
        "#
    );
}
