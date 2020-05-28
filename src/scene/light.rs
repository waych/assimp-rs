use ffi::aiLight;

define_type_and_iterator_indirect! {
    /// Light type (not yet implemented)
    struct Light(&aiLight)
    /// Light iterator type.
    struct LightIter
}
