use flatten_member_exprs::flatten_member_exprs;
use swc_core::ecma::{ast::*, visit::VisitMut};

mod flatten_member_exprs;
mod power_assert_recorder;
/// This is the equivalent of babel-plugin-espower - transforms plain assertion calls into power-assert calls
pub struct PowerAssertTransformerVisitor;

impl VisitMut for PowerAssertTransformerVisitor {
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        let inner = || {
            let _ = flatten_member_exprs(node.callee.as_expr()?);
            Some(())
        };
        inner();
    }
}
