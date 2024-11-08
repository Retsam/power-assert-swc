use swc_core::ecma::{
    ast::{BinExpr, BinaryOp, Expr, Lit, Program},
    transforms::testing::test_inline,
    visit::{as_folder, FoldWith, VisitMut, VisitMutWith},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

pub struct TransformVisitor;

impl VisitMut for TransformVisitor {
    fn visit_mut_expr(&mut self, e: &mut Expr) {
        match e {
            Expr::Lit(Lit::Str(str)) => {
                *e = Expr::Bin(BinExpr {
                    span: str.span,
                    op: BinaryOp::Add,
                    left: Box::new(str.clone().into()),
                    right: "!".into(),
                });
            }
            _ => e.visit_mut_children_with(self),
        }
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
    r#"console.log("transform");"#,
    // Output codes after transformed with plugin
    r#"console.log("transform" + "!");"#
);
