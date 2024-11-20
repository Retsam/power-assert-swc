use std::sync::Arc;

use flatten_member_exprs::flatten_member_exprs;
use power_assert_recorder::{
    new_power_assert_recorder_stmt, power_assert_recorder_definition, wrap_in_capture,
    wrap_in_record,
};
use swc_core::{
    common::{util::take::Take, Mark, SourceMapper, Span, Spanned},
    ecma::{
        ast::*,
        transforms::{
            base::resolver,
            testing::{test, Tester},
        },
        visit::{visit_mut_pass, VisitMut, VisitMutWith},
    },
};

mod expr_tests;
mod flatten_member_exprs;
mod power_assert_recorder;
/// This is the equivalent of babel-plugin-espower - transforms plain assertion calls into power-assert calls
pub struct PowerAssertTransformerVisitor {
    found_assertion: bool,
    needs_recorder: bool,

    file_name: String,
    source_map: Arc<dyn SourceMapper>,
}

#[derive(PartialEq)]
enum CaptureExprContext {
    InsideCallee,
}

impl PowerAssertTransformerVisitor {
    pub fn new(file_name: String, source_map: Arc<dyn SourceMapper>) -> Self {
        Self {
            found_assertion: false,
            needs_recorder: false,
            file_name,
            source_map,
        }
    }

    fn is_assertion_call(&self, node: &CallExpr) -> bool {
        node.callee
            .as_expr()
            .and_then(|callee| flatten_member_exprs(callee))
            .map(|callee_name| callee_name == "assert")
            .unwrap_or(false)
    }

    fn transform_assert_call(&mut self, node: &mut Expr, assert_span: Span) -> Result<(), String> {
        let expr = self.capture_expr(node.take(), vec!["arguments/0".into()], None);
        if !self.needs_recorder {
            *node = expr;
            return Ok(());
        }
        let source_code = self
            .source_map
            .span_to_snippet(assert_span)
            .map_err(|_| "Failed to get source_code for expr")?;

        // Currently the span_to_lines logic is panicking in WASM - until I can figure that out, just hard-code a line
        let line_num = if cfg!(target_family = "wasm") {
            4
        } else {
            self.source_map
                .span_to_lines(assert_span)
                .map_err(|_| "Failed to get line for expr")
                .and_then(|lines| {
                    if lines.lines.is_empty() {
                        return Err("Failed to get line for expr: empty lines list");
                    }
                    Ok(lines.lines[0].line_index)
                })?
        };
        *node = wrap_in_record(expr, &self.file_name, &source_code, line_num);
        Ok(())
    }

    fn capture_expr(
        &mut self,
        mut node: Expr,
        path: Vec<String>,
        ctx: Option<CaptureExprContext>,
    ) -> Expr {
        let append_path = |str: &str| {
            let mut x = Vec::with_capacity(path.len() + 1);
            x.clone_from(&path);
            x.push(str.to_string());
            x
        };
        macro_rules! capt {
            ($self: ident, $expr: expr) => {{
                let e = $expr.into();
                if ctx == Some(CaptureExprContext::InsideCallee) {
                    e
                } else {
                    self.found_assertion = true;
                    self.needs_recorder = true;
                    wrap_in_capture(e, path.join("/"))
                }
            }};
        }

        /// This macro is a shortcut for the common pattern of recursing `capture_expr` into 'sub expressions', e.g. the left and right side of BinExpr
        macro_rules! capture_subs_exprs {
            ($self: ident, $orig: ident, $expr_kind: ident {
                $($field: ident $(as $path: literal)?),+
            }) => {
                $expr_kind { $($field: Box::new($self.capture_expr(*$orig.$field, append_path(path_from_field!($field $($path)?)), None))),+, ..$orig }
            };
        }
        macro_rules! path_from_field {
            ($field: ident) => {
                stringify!($field)
            };
            ($field: ident $path: literal) => {
                $path
            };
        }

        match node.take() {
            // Expr::Array(array_lit) => todo!(),
            // Expr::Object(object_lit) => todo!(),
            // Expr::Fn(fn_expr) => todo!(),
            Expr::Unary(unary_expr) => capt!(
                self,
                capture_subs_exprs!(self, unary_expr, UnaryExpr { arg })
            ),
            // Expr::Update(update_expr) => todo!(),
            Expr::Bin(bin_expr) => capt!(
                self,
                capture_subs_exprs!(self, bin_expr, BinExpr { left, right })
            ),
            // Expr::Assign(assign_expr) => todo!(),
            Expr::Member(member_expr) => capt!(
                self,
                capture_subs_exprs!(self, member_expr, MemberExpr { obj as "object" })
            ),
            // Expr::SuperProp(super_prop_expr) => todo!(),
            Expr::Cond(cond_expr) => {
                capture_subs_exprs!(self, cond_expr, CondExpr { test, cons as "consequent", alt as "alternate" })
                    .into()
            }
            Expr::Call(call_expr) => capt!(
                self,
                CallExpr {
                    callee: match call_expr.callee {
                        Callee::Expr(expr) => {
                            // We call self.capture_expr here, but not capt! - we don't necessarily capture the callee, but might capture parts of it
                            Callee::Expr(Box::new(self.capture_expr(
                                *expr,
                                append_path("callee"),
                                // We don't directly capture the callee (it's a function, not interesting to print out)
                                //  ... but there might be sub-expressions that are capture-able
                                // This flag will skip the capture in the recursive call
                                Some(CaptureExprContext::InsideCallee),
                            )))
                        }
                        // Nothing to capture in these cases
                        Callee::Super(_) | Callee::Import(_) => call_expr.callee,
                    },
                    args: call_expr
                        .args
                        .into_iter()
                        .enumerate()
                        .map(|(i, x)| ExprOrSpread {
                            expr: Box::new(self.capture_expr(
                                *x.expr,
                                append_path(&format!("arguments/{i}")),
                                None
                            ),),
                            ..x
                        })
                        .collect(),
                    ..call_expr
                }
            ),
            // Expr::New(new_expr) => todo!(),
            // Expr::Seq(seq_expr) => todo!(),
            expr @ Expr::Ident(_) => capt!(self, expr),
            expr @ (Expr::Lit(_) | Expr::This(_)) => expr,
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
            expr => expr,
        }
    }
}

impl VisitMut for PowerAssertTransformerVisitor {
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        if !self.is_assertion_call(node) {
            node.visit_mut_children_with(self);
            return;
        }
        // Used to output the input source code into the data passed to power-assert,
        //  including the assert function call, which is why the span is captured here, not inside transform_assert_call
        let assert_span = node.span();
        if let [ExprOrSpread { expr, .. }] = &mut node.args[..] {
            let _ = self.transform_assert_call(expr, assert_span);
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
fn tr(tester: &mut Tester) -> impl Pass {
    (
        resolver(Mark::new(), Mark::new(), false),
        visit_mut_pass(PowerAssertTransformerVisitor::new(
            "test.js".into(),
            tester.cm.clone(),
        )),
    )
}

test!(
    Default::default(),
    tr,
    assert_inside_func,
    r#"
    function f1() {
        assert(a);
    }
    // expr
    const f2 = function foo() {
        assert(a);
    }
    // arrow
    const f3 = () => {
        assert(a);
    }
    // arrow shorthand
    const f4 = () => assert(a);

    // nested
    function outer() {
        assert(true);
        function inner() {
            assert(a);
        }
    }
    "#
);

test!(
    Default::default(),
    tr,
    top_level_assert,
    r#"
    import assert from 'assert';
    assert(a)
    "#
);

// Shouldn't replace assert(true) with any sort of power-assert stuff
test!(
    Default::default(),
    tr,
    boring_assert,
    r#"
    import assert from 'assert';

    assert(true);
    "#
);

test!(
    Default::default(),
    tr,
    avoids_name_conflict_with_local,
    r#"
    import { assert } from 'assert';
    const _powerAssertRecorder = "name taken";
    assert(a);
    function f() {
        assert(_powerAssertRecorder);
    }
    "#
);
test!(
    Default::default(),
    tr,
    avoids_name_conflict_with_import,
    r#"
    import { _powerAssertRecorder } from "somewhere-else";
    import { assert } from 'assert';
    assert(a);
    function f() {
        assert(a);
    }
    "#
);

test!(
    Default::default(),
    tr,
    no_assert,
    r#"
    import notAssert from 'assert';
    notAssert(true);
    "#
);
