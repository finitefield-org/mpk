mod helpers {
    fn private_leaf(value: u8) -> u8 {
        value
    }

    pub fn public_helper(value: u8) -> u8 {
        private_leaf(value)
    }
}

pub fn cross_module(value: u8) -> u8 {
    crate::helpers::public_helper(value)
}
