pub fn early_return(value: i8, return_early: bool) -> i8 {
    if return_early {
        return value;
    }
    -value
}
