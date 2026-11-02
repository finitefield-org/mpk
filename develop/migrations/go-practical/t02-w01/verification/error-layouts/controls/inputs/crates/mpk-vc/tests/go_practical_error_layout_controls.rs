// Reuse the original immutable helpers while its separate matrix run lives.
// The filter selects only the added control root below; included roots do not run.
include!("go_practical_error_layouts.rs");

#[test]
fn controls_without_unreferenced_type_probe() {
    corruption(3, "wrong-tag", "Tag", |c| constant(c, "Mpk.Go.Practical.Word.W64.Zero"));
    corruption(3, "wrong-zero-tag", "ZeroTag", |c| constant(c, "Mpk.Go.Practical.Word.W64.One"));
    for (n, index) in [(3,0),(3,2),(70,0),(70,64),(70,69)] {
        corruption(n, &format!("dropped-N{n}-payload{index}"), &format!("Payload{index}"),
            |c| constant(c,"Std.Bool.false"));
    }
    for n in [0,3,70] {
        corruption(n, &format!("dropped-N{n}-from-view-tag"), "FromView",
            |c| term(c,TermNode::Var(0)));
    }
}
