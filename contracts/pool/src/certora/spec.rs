use cvlr::asserts::cvlr_satisfy;
use cvlr_soroban_derive::rule;

#[rule]
fn sanity() {
    cvlr_satisfy!(true);
}
