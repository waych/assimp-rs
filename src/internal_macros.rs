// TODO: Quite messy macro stuff, needs documenting/tidying up

// Define the iterator struct and constructor from raw data.
// Same for all data types.
macro_rules! define_iter {
    ($(#[$iter_attr:meta])* struct $name:ident -> $raw:ty) => (
        $(#[$iter_attr])*
        pub struct $name<'a> {
            ptr: Option<$raw>,
            len: usize,
            _mk: ::std::marker::PhantomData<&'a ()>
        }

        #[doc(hidden)]
        impl<'a> $name<'a> {
            pub fn new(ptr: Option<$raw>, len: usize) -> $name<'a> {
                $name { ptr, len: len, _mk: ::std::marker::PhantomData }
            }
        }
    )
}

macro_rules! impl_iterator {
    ($name:ident, $item:ident) => {
        impl<'a> Iterator for $name<'a> {
            type Item = &'a $item;

            fn next(&mut self) -> Option<&'a $item> {
                if self.len > 0 {
                    unsafe {
                        let ptr = self.ptr?;

                        let item = $item::from_raw(ptr);

                        self.ptr = ::std::ptr::NonNull::new(ptr.as_ptr().offset(1) as *mut _);
                        self.len -= 1;

                        Some(item)
                    }
                } else {
                    None
                }
            }
        }

        impl<'a> ExactSizeIterator for $name<'a> {
            fn len(&self) -> usize {
                self.len
            }
        }
    };
}

macro_rules! impl_iterator_indirect {
    ($name:ident, $item:ident) => {
        impl<'a> Iterator for $name<'a> {
            type Item = &'a $item;

            fn next(&mut self) -> Option<Self::Item> {
                if self.len > 0 {
                    unsafe {
                        let ptr = self.ptr?;

                        let item =
                            $item::from_raw(::std::ptr::NonNull::new(*ptr.as_ptr() as *mut _)?);

                        self.ptr = ::std::ptr::NonNull::new(ptr.as_ptr().offset(1) as *mut _);
                        self.len -= 1;

                        Some(item)
                    }
                } else {
                    None
                }
            }
        }

        impl<'a> ExactSizeIterator for $name<'a> {
            fn len(&self) -> usize {
                self.len
            }
        }
    };
}

macro_rules! impl_iterator_pod {
    ($name:ident, $item:ident) => {
        impl<'a> Iterator for $name<'a> {
            type Item = $item;

            fn next(&mut self) -> Option<$item> {
                if self.len > 0 {
                    let ptr = self.ptr?;

                    let item = $item::from_raw(unsafe { *ptr.as_ptr() });

                    self.ptr =
                        unsafe { ::std::ptr::NonNull::new(ptr.as_ptr().offset(1) as *mut _) };
                    self.len -= 1;

                    Some(item)
                } else {
                    None
                }
            }
        }

        impl<'a> ExactSizeIterator for $name<'a> {
            fn len(&self) -> usize {
                self.len
            }
        }
    };
}

macro_rules! define_type {
    // Reference type
    ($(#[$type_attr:meta])* struct $name:ident(&$raw:ty)) => (
        $(#[$type_attr])*
        #[repr(transparent)]
        pub struct $name($raw);

        impl $name {
            /// Create a borrow of this struct from a raw pointer
            pub unsafe fn from_raw<'a>(raw: ::std::ptr::NonNull<$raw>) -> &'a $name {
                ::std::mem::transmute(raw)
            }

            /// Convert a borrow of this struct to a raw pointer
            pub fn to_raw(&self) -> ::std::ptr::NonNull<$raw> {
                self.as_ref().into()
            }

            /// Get a pointer to the inner value
            pub fn as_ref(&self) -> &$raw {
                &self.0
            }
        }

        impl ::std::ops::Deref for $name {
            type Target = $raw;

            fn deref(&self) -> &$raw {
                self.as_ref()
            }
        }
    );
    // Non-reference type = POD
    ($(#[$type_attr:meta])* struct $name:ident($raw:ty)) => (
        $(#[$type_attr])*
        pub struct $name(pub $raw);

        impl $name {
            /// Create this struct from the equivalent struct in assimp
            pub fn from_raw(raw: $raw) -> $name {
                $name(raw)
            }

            /// Convert this struct to the equivalent struct in assimp
            pub fn to_raw(self) -> $raw {
                self.0
            }
        }

        impl ::std::ops::Deref for $name {
            type Target = $raw;

            fn deref(&self) -> &$raw {
                &self.0
            }
        }
    );
}

macro_rules! define_type_and_iterator {
    (
        $(#[$type_attr:meta])* struct $type_name:ident(&$raw:ty)
        $(#[$iter_attr:meta])* struct $iter_name:ident
    ) => (
        define_type!($(#[$type_attr])* struct $type_name(&$raw));
        define_iter!($(#[$iter_attr])* struct $iter_name -> ::std::ptr::NonNull<$raw>);
        impl_iterator!($iter_name, $type_name);
    );
    (
        $(#[$type_attr:meta])* struct $type_name:ident($raw:ty)
        $(#[$iter_attr:meta])* struct $iter_name:ident
    ) => (
        define_type!($(#[$type_attr])* struct $type_name($raw));
        define_iter!($(#[$iter_attr])* struct $iter_name -> ::std::ptr::NonNull<$raw>);
        impl_iterator_pod!($iter_name, $type_name);
    );
}

macro_rules! define_type_and_iterator_indirect {
    (
        $(#[$type_attr:meta])* struct $type_name:ident(&$raw:ty)
        $(#[$iter_attr:meta])* struct $iter_name:ident
    ) => (
        define_type!($(#[$type_attr])* struct $type_name(&$raw));
        define_iter!($(#[$iter_attr])* struct $iter_name -> ::std::ptr::NonNull<*const $raw>);
        impl_iterator_indirect!($iter_name, $type_name);
    );
}
