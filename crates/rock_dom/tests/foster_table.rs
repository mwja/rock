mod common;
use common::*;
use rock_dom::prelude::*;
use tracing_test::traced_test;

#[traced_test]
#[test]
fn foster_parenting() {
    let dom = create_dom("<table>oops<tr><td>x</td></tr></table>").expect("failed to create dom");
    insta::assert_snapshot!(dump(&dom));
}
