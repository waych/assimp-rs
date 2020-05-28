use ffi::aiCamera;

define_type_and_iterator_indirect! {
    /// Camera type (not yet implemented)
    struct Camera(&aiCamera)
    /// Camera iterator type.
    struct CameraIter
}
