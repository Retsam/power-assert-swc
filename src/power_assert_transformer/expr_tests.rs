use swc_core::ecma::transforms::testing::test;

#[allow(unused)]
use super::tr;

macro_rules! expr_test {
    ($name: ident, $code: literal) => {
        test!(
            Default::default(),
            tr,
            $name,
            &format!("import assert from 'assert';\n\n{}", $code)
        );
    };
}
expr_test!(
    expr_basic,
    r#"
    assert(x.toUpperCase() == "BAR" ? x.y : x.z);
    "#
);

expr_test!(
    expr_classes,
    r#"
    assert(new x.y() instanceof SomeThing)
    "#
);
