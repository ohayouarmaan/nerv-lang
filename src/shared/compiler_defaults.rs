pub struct Sizes {
    pub d_int: usize,
    pub d_float: usize,
    pub d_bool: usize,
    pub d_ptr: usize,
}

pub const SIZES: Sizes = Sizes {
    d_int: 4,
    d_float: 8,
    d_bool: 1,
    d_ptr: 8,
};

