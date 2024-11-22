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
                $($field: ident $(as $path: expr)?),+
            }) => {
                $expr_kind { $($field: Box::new($self.capture_expr(*$orig.$field, append_path(path_from_field!($field $($path)?)), None))),+, ..$orig }
            };
        }
        macro_rules! path_from_field {
            ($field: ident) => {
                stringify!($field)
            };
            ($field: ident $path: expr) => {
                $path
            };
        }

        // Captures a single sub expr, with no change in the path
        macro_rules! capture_sub_expr {
            ($self: ident, $orig: ident.$field: ident) => {{
                let mut new_expr = $orig.clone();
                *new_expr.$field = $self.capture_expr(*$orig.$field, path.clone(), None);
                new_expr.into()
            }};
        }

        // Common logic shared between Expr::Call and Expr::New
        macro_rules! capture_args {
            ($self: ident, $args: expr) => {
                $args
                    .into_iter()
                    .enumerate()
                    .map(|(i, arg)| capture_subs_exprs!(self, arg, ExprOrSpread {
                        expr as &format!("arguments/{i}")
                    }))
                    .collect::<Vec<_>>()
            };
        }

        match node.take() {
            Expr::Array(array_lit) => capt!(self, ArrayLit {
                elems: array_lit.elems.into_iter()
                .enumerate()
                .map(|(i, maybe_el)| maybe_el.map(|el| capture_subs_exprs!(self, el, ExprOrSpread { expr as &format!("elements/{i}") }))).collect(),
                ..array_lit
            }),
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
                    args: capture_args!(self, call_expr.args),
                    ..call_expr
                }
            ),
            // Same as Expr::Call, except callee only has the Expr case and args is Optional
            Expr::New(new_expr) => capt!(
                self,
                NewExpr {
                    callee: Box::new(self.capture_expr(*new_expr.callee, append_path("callee"), Some(CaptureExprContext::InsideCallee))),
                    args: new_expr.args.map(|args| capture_args!(self, args)),
                    ..new_expr
                }
            ),
            Expr::Seq(seq_expr) => SeqExpr {
                exprs: seq_expr.exprs.into_iter()
                .enumerate()
                .map(|(i, expr)| Box::new(self.capture_expr(*expr, append_path(&format!("expressions/{i}")), None))).collect(),
                ..seq_expr
            }.into(),
            expr @ (Expr::Ident(_) | Expr::SuperProp(_)) => capt!(self, expr),
            // "Boring" cases where the expr is just a thin wrapper and we just want to recurse into the wrapper
            Expr::Paren(paren_expr) => capture_sub_expr!(self, paren_expr.expr),
            Expr::TsAs(ts_as_expr) => capture_sub_expr!(self, ts_as_expr.expr),
            Expr::TsInstantiation(ts_instantiation) => capture_sub_expr!(self, ts_instantiation.expr),

            // Exprs that are just ignored, neither captured nor recursed into
            expr @ (Expr::Lit(_) | Expr::This(_) | Expr::Class(_) | Expr::Invalid(_)) => expr,
            // Expr::Tpl(tpl) => todo!(),
            // Expr::TaggedTpl(tagged_tpl) => todo!(),
            // Expr::Arrow(arrow_expr) => todo!(),
            // Expr::Yield(yield_expr) => todo!(),
            // Expr::MetaProp(meta_prop_expr) => todo!(),
            // Expr::Await(await_expr) => todo!(),
            // Expr::JSXMember(jsxmember_expr) => todo!(),
            // Expr::JSXNamespacedName(jsxnamespaced_name) => todo!(),
            // Expr::JSXEmpty(jsxempty_expr) => todo!(),
            // Expr::JSXElement(jsxelement) => todo!(),
            // Expr::JSXFragment(jsxfragment) => todo!(),
            // Expr::TsTypeAssertion(ts_type_assertion) => todo!(),
            // Expr::TsConstAssertion(ts_const_assertion) => todo!(),
            // Expr::TsNonNull(ts_non_null_expr) => todo!(),
            // Expr::TsSatisfies(ts_satisfies_expr) => todo!(),
            // Expr::PrivateName(private_name) => todo!(),
            // Expr::OptChain(opt_chain_expr) => todo!(),
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
        node.power_assert(self)
    }
    fn visit_mut_constructor(&mut self, node: &mut Constructor) {
        node.power_assert(self)
    }
    fn visit_mut_arrow_expr(&mut self, node: &mut ArrowExpr) {
        node.power_assert(self);
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

/// Trait used by things that contain power asserts and need to inject the recorder element: functions, arrow functions, constructors, generators, etc
trait PowerAssertContainer: VisitMutWith<PowerAssertTransformerVisitor> {
    fn power_assert(&mut self, visitor: &mut PowerAssertTransformerVisitor) {
        let orig_needs_recorder = visitor.needs_recorder;
        self.visit_mut_children_with(visitor);
        if visitor.needs_recorder {
            self.insert_recorder(new_power_assert_recorder_stmt());
            visitor.needs_recorder = orig_needs_recorder;
        }
    }
    fn insert_recorder(&mut self, recorder_stmt: Stmt);
}

macro_rules! normal_container_impl {
    ($kind:ident) => {
        impl PowerAssertContainer for $kind {
            fn insert_recorder(&mut self, recorder_stmt: Stmt) {
                if let Some(ref mut body) = self.body {
                    body.stmts.insert(0, recorder_stmt);
                }
                // it'd be very weird if we didn't have a body but somehow tripped 'needs_recorder'
            }
        }
    };
}

normal_container_impl!(Function);
normal_container_impl!(Constructor);

impl PowerAssertContainer for ArrowExpr {
    fn insert_recorder(&mut self, recorder_stmt: Stmt) {
        match &mut *self.body {
            BlockStmtOrExpr::BlockStmt(block) => {
                block.stmts.insert(0, recorder_stmt);
            }
            // Need to 'un-shorthand' the shorthand form
            BlockStmtOrExpr::Expr(expr) => {
                *self.body = BlockStmt {
                    stmts: vec![
                        recorder_stmt,
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
    }
}

#[allow(unused)]
fn tr(tester: &mut Tester) -> impl Pass {
    (
        resolver(Mark::new(), Mark::new(), false),
        visit_mut_pass(PowerAssertTransformerVisitor::new(
            "input/test.js".into(),
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
    assert_inside_method,
    r#"
class C {
    constructor() {
        assert(a)
    }
    m1() {
        assert(a);
    }
    #m2() {
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
