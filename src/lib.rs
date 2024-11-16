use assert_import_replacer::ReplaceImportsVisitor;
use power_assert_transformer::PowerAssertTransformerVisitor;
use swc_core::ecma::visit::{visit_mut_pass, VisitMutWith};
use swc_core::ecma::{ast::Program, visit::VisitMut};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

mod assert_import_replacer;
mod power_assert_transformer;

pub struct TransformVisitor;

impl VisitMut for TransformVisitor {
    fn visit_mut_program(&mut self, node: &mut Program) {
        node.visit_mut_children_with(&mut ReplaceImportsVisitor {});
        node.visit_mut_children_with(&mut PowerAssertTransformerVisitor::new());
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.apply(visit_mut_pass(TransformVisitor))
}
