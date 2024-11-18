use std::sync::Arc;

use assert_import_replacer::ReplaceImportsVisitor;
use power_assert_transformer::PowerAssertTransformerVisitor;
use swc_core::common::SourceMapper;
use swc_core::ecma::visit::{visit_mut_pass, VisitMutWith};
use swc_core::ecma::{ast::Program, visit::VisitMut};
use swc_core::plugin::metadata::TransformPluginMetadataContextKind;
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

mod assert_import_replacer;
mod power_assert_transformer;

pub struct TransformVisitor {
    file_name: String,
    source_map: Arc<dyn SourceMapper>,
}

impl VisitMut for TransformVisitor {
    fn visit_mut_program(&mut self, node: &mut Program) {
        node.visit_mut_children_with(&mut ReplaceImportsVisitor {});
        node.visit_mut_children_with(&mut PowerAssertTransformerVisitor::new(
            self.file_name.clone(),
            self.source_map.clone(),
        ));
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, metadata: TransformPluginProgramMetadata) -> Program {
    let file_name = metadata
        .get_context(&TransformPluginMetadataContextKind::Filename)
        .unwrap_or("<source>".into());
    program.apply(visit_mut_pass(TransformVisitor {
        file_name,
        source_map: Arc::new(metadata.source_map),
    }))
}
