use ffi::aiMaterial;

define_type_and_iterator_indirect! {
    /// Material type (not yet implemented)
    struct Material(&aiMaterial)
    /// Material iterator type.
    struct MaterialIter
}
