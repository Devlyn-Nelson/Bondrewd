pub enum ArrayType {
    /// Each element of the array is considered its own value.
    ///
    /// Good for data, or a collection of datums.
    Element,
    /// All of the useful bits/bytes in the array describe a singular piece of information.
    ///
    /// Good for Strings, or a series of bits/bytes that make 1 datum.
    Block,
}

struct ArrayInfo {
    ty: ArrayType,
    /// Each element represents a dimension to the array with the value being the amount of elements
    /// for that dimension.
    ///
    /// # Examples
    /// a single dimensional array would only have 1 value
    /// |      |Element 1|
    /// |:-----|:-------:|
    /// |[u8;4]|        4|
    ///
    /// X dimensional array will have X values, first being the outer-most array size going
    /// to the inner-most.
    ///
    /// |          |Element 1|Element 2|
    /// |:---------|:-------:|:-------:|
    /// |[[u8;4];5]|        5|        4|
    sizings: Vec<usize>,
}

///
pub struct DataBuilder {
    /// Type of data.
    ty: DataType,
    /// Size of the rust native type in bytes.
    rust_size: u8,
    /// If `None` this data is not in an array and should just be treated as a single value.
    ///
    /// If `Some` than this is an array, NOT a single value. Also Note that the `ty` and `rust_size` only
    /// describe a true data type, which would be the innermost part of an array. The array info
    /// is marly keeping track of the order and magnitude of the array and its dimensions.
    array: Option<ArrayInfo>,
}

enum DataType {
    /// Floating point numbers
    ///
    /// # Valid
    /// - f32
    /// - f64
    Float,
    /// Unsigned numbers
    ///
    /// # Valid
    /// - u8
    /// - u16
    /// - u32
    /// - u64
    /// - u128
    Unsigned,
    /// Unsigned numbers
    ///
    /// # Valid
    /// - i8
    /// - i16
    /// - i32
    /// - i64
    /// - i128
    Signed,
    /// This is a nested structure and does not have a know type. and the name of the struct shall be stored
    /// within.
    Nested(String),
}
