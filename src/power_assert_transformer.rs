use std::sync::Arc;

use flatten_member_exprs::flatten_member_exprs;
use power_assert_recorder::{
    new_power_assert_recorder_stmt, power_assert_recorder_definition, wrap_in_record,
    RecorderContext,
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

mod capture_expr;
mod expr_tests;
mod flatten_member_exprs;
mod power_assert_recorder;
/// This is the equivalent of babel-plugin-espower - transforms plain assertion calls into power-assert calls
pub struct PowerAssertTransformerVisitor {
    found_assertion: bool,
    needs_recorder: bool,

    file_name: String,
    source_map: Arc<dyn SourceMapper>,

    recorder_context: RecorderContext,
}

impl PowerAssertTransformerVisitor {
    pub fn new(file_name: String, source_map: Arc<dyn SourceMapper>) -> Self {
        Self {
            found_assertion: false,
            needs_recorder: false,
            file_name,
            source_map,
            recorder_context: <_>::default(),
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
        let line_num = {
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
        *node = wrap_in_record(
            expr,
            &self.file_name,
            &source_code,
            line_num,
            self.recorder_context,
        );
        Ok(())
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
        let orig_recorder_context = visitor.recorder_context;
        visitor.recorder_context = self.recorder_context();
        self.visit_mut_children_with(visitor);
        if visitor.needs_recorder {
            self.insert_recorder(new_power_assert_recorder_stmt());
        }
        visitor.recorder_context = orig_recorder_context;
        visitor.needs_recorder = orig_needs_recorder;
    }
    fn insert_recorder(&mut self, recorder_stmt: Stmt);
    fn recorder_context(&mut self) -> RecorderContext;
}

macro_rules! normal_container_impl {
    () => {
        fn insert_recorder(&mut self, recorder_stmt: Stmt) {
            if let Some(ref mut body) = self.body {
                body.stmts.insert(0, recorder_stmt);
            }
            // it'd be very weird if we didn't have a body but somehow tripped 'needs_recorder'
        }
    };
}
macro_rules! normal_recorder_context_impl {
    () => {
        fn recorder_context(&mut self) -> RecorderContext {
            RecorderContext {
                is_async: self.is_async,
                is_generator: self.is_generator,
            }
        }
    };
}

impl PowerAssertContainer for Function {
    normal_container_impl!();
    normal_recorder_context_impl!();
}
impl PowerAssertContainer for Constructor {
    normal_container_impl!();
    fn recorder_context(&mut self) -> RecorderContext {
        <_>::default()
    }
}

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
    normal_recorder_context_impl!();
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

    // async
    async function f5() {
        assert(a);
    }

    // async arrow
    const f6 = async () => assert(a);

    // nested
    function outer() {
        assert(a);
        function inner() {
            assert(a);
        }
    }

    // generator
    function *gen() {
        assert(a);
    }

    // async generator
    async function *async_gen() {
        assert(a);
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
