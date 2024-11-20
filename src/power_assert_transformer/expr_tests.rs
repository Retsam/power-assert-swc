use swc_core::ecma::transforms::testing::test;

#[allow(unused)]
use super::tr;

test!(
    Default::default(),
    tr,
    basic_test,
    r#"
    import assert from 'assert';

    assert(x.toUpperCase() == "BAR" ? x.y : x.z);
    "#
);
