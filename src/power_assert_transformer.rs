use flatten_member_exprs::flatten_member_exprs;
use power_assert_recorder::power_assert_recorder_definition;
use swc_core::ecma::{
    ast::*,
    transforms::testing::test,
    visit::{visit_mut_pass, VisitMut, VisitMutWith},
};

mod flatten_member_exprs;
mod power_assert_recorder;
/// This is the equivalent of babel-plugin-espower - transforms plain assertion calls into power-assert calls
pub struct PowerAssertTransformerVisitor {
    found_assertion: bool,
}

impl PowerAssertTransformerVisitor {
    pub fn new() -> Self {
        Self {
            found_assertion: false,
        }
    }

    fn is_assertion_call(&self, node: &CallExpr) -> bool {
        node.callee
            .as_expr()
            .and_then(|callee| flatten_member_exprs(callee))
            .map(|callee_name| callee_name == "assert")
            .unwrap_or(false)
    }
}

impl VisitMut for PowerAssertTransformerVisitor {
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        if !self.is_assertion_call(node) {
            return;
        }
        self.found_assertion = true;
    }

    fn visit_mut_program(&mut self, node: &mut Program) {
        node.visit_mut_children_with(self);
        if self.found_assertion {
            let stmt = power_assert_recorder_definition();
            match node {
                Program::Module(m) => {
                    m.body.push(ModuleItem::Stmt(stmt));
                }
                Program::Script(script) => script.body.push(stmt),
            }
        }
    }
}

test!(
    Default::default(),
    |_| visit_mut_pass(PowerAssertTransformerVisitor::new()),
    inserts_recorder,
    r#"
    import assert from 'assert';
    assert(true);
    "#
);

test!(
    Default::default(),
    |_| visit_mut_pass(PowerAssertTransformerVisitor::new()),
    doesnt_insert_recorder,
    r#"
    import notAssert from 'assert';
    notAssert(true);
    "#
);
