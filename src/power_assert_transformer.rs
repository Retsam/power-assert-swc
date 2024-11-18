use flatten_member_exprs::flatten_member_exprs;
use power_assert_recorder::{
    new_power_assert_recorder_stmt, power_assert_recorder_definition, wrap_in_capture,
    wrap_in_record,
};
use swc_core::{
    common::{util::take::Take, Mark},
    ecma::{
        ast::*,
        transforms::{base::resolver, testing::test},
        visit::{visit_mut_pass, VisitMut, VisitMutWith},
    },
};

mod flatten_member_exprs;
mod power_assert_recorder;
/// This is the equivalent of babel-plugin-espower - transforms plain assertion calls into power-assert calls
pub struct PowerAssertTransformerVisitor {
    found_assertion: bool,
    needs_recorder: bool,
}

impl PowerAssertTransformerVisitor {
    pub fn new() -> Self {
        Self {
            found_assertion: false,
            needs_recorder: false,
        }
    }

    fn is_assertion_call(&self, node: &CallExpr) -> bool {
        node.callee
            .as_expr()
            .and_then(|callee| flatten_member_exprs(callee))
            .map(|callee_name| callee_name == "assert")
            .unwrap_or(false)
    }

    fn transform_assert_call(&self, node: &mut Expr) {
        *node = wrap_in_record(capture_expr(node.take(), vec!["arguments/0".into()]));
    }
}

fn capture_expr(mut node: Expr, path: Vec<String>) -> Expr {
    macro_rules! append_path {
        ($str: expr) => {{
            let mut x = path.clone();
            x.push($str.to_string());
            x
        }};
    }

    let mut inner_expr = node.take();
    inner_expr = match inner_expr {
        Expr::This(_) => inner_expr,
        // Expr::Array(array_lit) => todo!(),
        // Expr::Object(object_lit) => todo!(),
        // Expr::Fn(fn_expr) => todo!(),
        Expr::Unary(unary_expr) => UnaryExpr {
            arg: Box::new(capture_expr(*unary_expr.arg, append_path!("argument"))),
            ..unary_expr
        }
        .into(),
        // Expr::Update(update_expr) => todo!(),
        // Expr::Bin(bin_expr) => todo!(),
        // Expr::Assign(assign_expr) => todo!(),
        // Expr::Member(member_expr) => todo!(),
        // Expr::SuperProp(super_prop_expr) => todo!(),
        // Expr::Cond(cond_expr) => todo!(),
        Expr::Call(call_expr) => CallExpr {
            args: call_expr
                .args
                .into_iter()
                .enumerate()
                .map(|(i, x)| ExprOrSpread {
                    expr: Box::new(capture_expr(
                        *x.expr,
                        append_path!(format!("arguments/{i}")),
                    )),
                    ..x
                })
                .collect(),
            ..call_expr
        }
        .into(),
        // Expr::New(new_expr) => todo!(),
        // Expr::Seq(seq_expr) => todo!(),
        // Expr::Ident(ident) => todo!(),
        // Expr::Lit(lit) => todo!(),
        // Expr::Tpl(tpl) => todo!(),
        // Expr::TaggedTpl(tagged_tpl) => todo!(),
        // Expr::Arrow(arrow_expr) => todo!(),
        // Expr::Class(class_expr) => todo!(),
        // Expr::Yield(yield_expr) => todo!(),
        // Expr::MetaProp(meta_prop_expr) => todo!(),
        // Expr::Await(await_expr) => todo!(),
        // Expr::Paren(paren_expr) => todo!(),
        // Expr::JSXMember(jsxmember_expr) => todo!(),
        // Expr::JSXNamespacedName(jsxnamespaced_name) => todo!(),
        // Expr::JSXEmpty(jsxempty_expr) => todo!(),
        // Expr::JSXElement(jsxelement) => todo!(),
        // Expr::JSXFragment(jsxfragment) => todo!(),
        // Expr::TsTypeAssertion(ts_type_assertion) => todo!(),
        // Expr::TsConstAssertion(ts_const_assertion) => todo!(),
        // Expr::TsNonNull(ts_non_null_expr) => todo!(),
        // Expr::TsAs(ts_as_expr) => todo!(),
        // Expr::TsInstantiation(ts_instantiation) => todo!(),
        // Expr::TsSatisfies(ts_satisfies_expr) => todo!(),
        // Expr::PrivateName(private_name) => todo!(),
        // Expr::OptChain(opt_chain_expr) => todo!(),
        // Expr::Invalid(invalid) => todo!(),
        _ => inner_expr,
    };
    wrap_in_capture(inner_expr, path.join("/"))
}

impl VisitMut for PowerAssertTransformerVisitor {
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        if !self.is_assertion_call(node) {
            node.visit_mut_children_with(self);
            return;
        }
        if let [ExprOrSpread { expr, .. }] = &mut node.args[..] {
            self.found_assertion = true;
            self.needs_recorder = true;
            self.transform_assert_call(expr);
        }
    }

    fn visit_mut_function(&mut self, node: &mut Function) {
        let orig_needs_recorder = self.needs_recorder;
        node.visit_mut_children_with(self);
        if self.needs_recorder {
            if let Some(ref mut body) = node.body {
                self.needs_recorder = orig_needs_recorder;
                body.stmts.insert(0, new_power_assert_recorder_stmt());
            }
        }
    }
    fn visit_mut_arrow_expr(&mut self, node: &mut ArrowExpr) {
        let orig_needs_recorder = self.needs_recorder;
        node.visit_mut_children_with(self);
        if self.needs_recorder {
            let stmt = new_power_assert_recorder_stmt();
            match &mut *node.body {
                BlockStmtOrExpr::BlockStmt(block) => {
                    block.stmts.insert(0, stmt);
                }
                // Need to 'un-shorthand' the shorthand form
                BlockStmtOrExpr::Expr(expr) => {
                    *node.body = BlockStmt {
                        stmts: vec![
                            stmt,
                            Stmt::Expr(ExprStmt {
                                expr: expr.take(),
                                ..Default::default()
                            }),
                        ],
                        ..Default::default()
                    }
                    .into();
                }
            };
            self.needs_recorder = orig_needs_recorder;
        }
    }

    fn visit_mut_program(&mut self, node: &mut Program) {
        node.visit_mut_children_with(self);
        if self.found_assertion {
            let mut prepend_stmt = |stmt: Stmt| match node {
                Program::Module(m) => {
                    // In case of a module, try to put statements after the imports
                    let pos = m
                        .body
                        .iter()
                        .position(|mod_item| mod_item.is_stmt())
                        .unwrap_or(0);
                    m.body.insert(pos, stmt.into());
                }

                Program::Script(script) => script.body.insert(0, stmt),
            };

            // If needs_recorder is true, program has top-level asserts so we need a top-level recorder.
            // Order doesn't matter between this and the class definition, but defining the class first (so prepending it after this) looks nicer.
            if self.needs_recorder {
                prepend_stmt(new_power_assert_recorder_stmt());
            }
            prepend_stmt(power_assert_recorder_definition());
        }
    }
}
#[allow(unused)]
fn tr() -> impl Pass {
    (
        resolver(Mark::new(), Mark::new(), false),
        visit_mut_pass(PowerAssertTransformerVisitor::new()),
    )
}

test!(
    Default::default(),
    |_| tr(),
    assert_inside_func,
    r#"
    function f1() {
        assert(true);
    }
    // expr
    const f2 = function foo() {
        assert(true);
    }
    // arrow
    const f3 = () => {
        assert(true);
    }
    // arrow shorthand
    const f4 = () => assert(true);

    // nested
    function outer() {
        assert(true);
        function inner() {
            assert(true);
        }
    }
    "#
);

test!(
    Default::default(),
    |_| tr(),
    top_level_assert,
    r#"
    import assert from 'assert';
    assert(true);
    "#
);

test!(
    Default::default(),
    |_| tr(),
    expr_test,
    r#"
    import assert from 'assert';

    assert(!isNaN(a))
    "#
);

test!(
    Default::default(),
    |_| tr(),
    avoids_name_conflict_with_local,
    r#"
    import { assert } from 'assert';
    const _powerAssertRecorder = "name taken";
    assert(true);
    function f() {
        assert(_powerAssertRecorder);
    }
    "#
);
test!(
    Default::default(),
    |_| tr(),
    avoids_name_conflict_with_import,
    r#"
    import { _powerAssertRecorder } from "somewhere-else";
    import { assert } from 'assert';
    assert(true);
    function f() {
        assert(true);
    }
    "#
);

test!(
    Default::default(),
    |_| tr(),
    no_assert,
    r#"
    import notAssert from 'assert';
    notAssert(true);
    "#
);
