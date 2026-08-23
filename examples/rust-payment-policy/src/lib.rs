fn retain_approval(policy_approved: bool, _reserve_requested: bool) -> bool {
    policy_approved
}

/// Returns the policy decision unchanged.
///
/// A reserve request is the explicit precondition for calling the helper. The
/// sibling fixture intentionally omits that precondition from this function's
/// contract while keeping this accepted source unchanged.
pub fn approved_reserve_cents(policy_approved: bool, reserve_requested: bool) -> bool {
    retain_approval(policy_approved, reserve_requested)
}
