use assert_import_replacer::ReplaceImportsVisitor;
use power_assert_transformer::PowerAssertTransformerVisitor;
use swc_core::ecma::visit::VisitMutWith;
use swc_core::ecma::{
    ast::Program,
    transforms::testing::test_inline,
    visit::{as_folder, FoldWith, VisitMut},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

mod assert_import_replacer;
mod power_assert_transformer;

pub struct TransformVisitor;

impl VisitMut for TransformVisitor {
    fn visit_mut_program(&mut self, node: &mut Program) {
        node.visit_mut_children_with(&mut ReplaceImportsVisitor {});
        node.visit_mut_children_with(&mut PowerAssertTransformerVisitor);
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
