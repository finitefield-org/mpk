use payment_policy::approved_reserve_cents;

#[test]
fn approval_is_passed_through() {
    assert!(approved_reserve_cents(true, true));
    assert!(!approved_reserve_cents(false, true));
}
