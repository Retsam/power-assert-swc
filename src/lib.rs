use swc_core::common::util::take::Take;
use swc_core::common::{Span, SyntaxContext};
use swc_core::ecma::ast::{
    ArrowExpr, BlockStmt, BlockStmtOrExpr, CallExpr, Callee, Expr, ExprStmt, MemberExpr,
};
use swc_core::ecma::visit::VisitMutWith;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};
use swc_core::{
    common::Spanned,
    ecma::{
        ast::{Program, Stmt},
        transforms::testing::test_inline,
        visit::{as_folder, FoldWith, VisitMut},
    },
};

mod power_assert_recorder;

pub struct TransformVisitor;

impl VisitMut for TransformVisitor {
    fn visit_mut_stmt(&mut self, stmt: &mut Stmt) {
        stmt.visit_mut_children_with(self);
        *stmt = TransformVisitor::iife(vec![stmt.take()], stmt.span())
    }
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        let inner = || {
            let callee = node.callee.as_expr()?;
            println!("{callee:?}");
            match &**callee {
                // Could probably use a COW here
                Expr::Ident(id) => Some(id.sym.to_string()),
                Expr::Member(mem) => TransformVisitor::full_name(mem),
                _ => None,
            }
        };
        let res = inner();
    }
}
impl TransformVisitor {
    fn full_name(mem: &MemberExpr) -> Option<String> {
        let mut parts: Vec<&str> = vec![mem.prop.as_ident()?.sym.as_str()];
        let mut lhs = &*mem.obj;
        loop {
            match lhs {
                Expr::Ident(id) => {
                    parts.push(id.sym.as_str());
                    break;
                }
                Expr::Member(mem) => {
                    parts.push(mem.prop.as_ident()?.sym.as_str());
                    lhs = &*mem.obj;
                }
                _ => return None,
            }
        }
        Some(parts.into_iter().rev().collect::<Vec<_>>().join("."))
    }

    fn iife(stmts: Vec<Stmt>, span: Span) -> Stmt {
        Stmt::Expr(ExprStmt {
            span,
            expr: Box::new(Expr::Call(CallExpr {
                span,
                ctxt: SyntaxContext::empty(),
                args: vec![],
                type_args: None,
                callee: Callee::Expr(Box::new(Expr::Arrow(ArrowExpr {
                    span,
                    params: vec![],
                    ctxt: SyntaxContext::empty(),
                    body: Box::new(BlockStmtOrExpr::BlockStmt(BlockStmt {
                        stmts,
                        span,
                        ctxt: SyntaxContext::empty(),
                    })),
                    is_async: false,
                    is_generator: false,
                    return_type: None,
                    type_params: None,
                }))),
            })),
        })
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor))
}

test_inline!(
    Default::default(),
    |_| as_folder(TransformVisitor),
    boo,
    // Input codes
    r#"console.log("transform"); foo(); foo.bar.baz()"#,
    // Output codes after transformed with plugin
    r#"console.log("transform");"#
);
