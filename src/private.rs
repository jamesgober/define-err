//! Items that generated code calls. Not part of the public API for users:
//! see `docs/GENERATED-CODE.md` for the versioning rule that applies to them.
//!
//! The dispatch macros pick an implementation by autoref method lookup: the
//! macro calls a method on `&&&Wrap(value)`, and method resolution tries the
//! receiver with the most references first, so the impl for the most specific
//! type wins (`Box<dyn Error>` before `Option<E>` before `E: Error`). This is
//! how a declarative macro, which cannot see types, still generates the right
//! code for every field type.

use core::error::Error;

use crate::ErrorInfo;

/// A borrowed value for autoref dispatch.
#[doc(hidden)]
pub struct Wrap<'a, T: ?Sized>(pub &'a T);

/// `source()` for a field of type `E: Error + 'static`.
#[doc(hidden)]
pub trait SourceViaError<'a> {
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)>;
}

impl<'a, E: Error + 'static> SourceViaError<'a> for Wrap<'a, E> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(self.0)
    }
}

/// `source()` for a field of type `Option<E>`.
#[doc(hidden)]
pub trait SourceViaOption<'a> {
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)>;
}

impl<'a, E: Error + 'static> SourceViaOption<'a> for &Wrap<'a, Option<E>> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        match self.0 {
            Some(error) => Some(error),
            None => None,
        }
    }
}

/// `source()` for boxed and `'static` trait-object fields.
#[doc(hidden)]
pub trait SourceViaObject<'a> {
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)>;
}

impl<'a> SourceViaObject<'a> for &&Wrap<'a, &'static (dyn Error + 'static)> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(*self.0)
    }
}

impl<'a> SourceViaObject<'a> for &&Wrap<'a, &'static (dyn Error + Send + Sync + 'static)> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(*self.0)
    }
}

#[cfg(feature = "alloc")]
impl<'a> SourceViaObject<'a> for &&Wrap<'a, alloc::boxed::Box<dyn Error + 'static>> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(&**self.0)
    }
}

#[cfg(feature = "alloc")]
impl<'a> SourceViaObject<'a> for &&Wrap<'a, alloc::boxed::Box<dyn Error + Send + 'static>> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(&**self.0)
    }
}

#[cfg(feature = "alloc")]
impl<'a> SourceViaObject<'a> for &&Wrap<'a, alloc::boxed::Box<dyn Error + Send + Sync + 'static>> {
    #[inline]
    fn __define_err_source(&self) -> Option<&'a (dyn Error + 'static)> {
        Some(&**self.0)
    }
}

/// The inner value's own `source()`, for `transparent` variants.
#[doc(hidden)]
pub trait ForwardViaError<'a> {
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)>;
}

impl<'a, E: Error + 'static> ForwardViaError<'a> for Wrap<'a, E> {
    #[inline]
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)> {
        self.0.source()
    }
}

/// The inner value's own `source()` when it is a boxed trait object.
#[doc(hidden)]
pub trait ForwardViaObject<'a> {
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)>;
}

#[cfg(feature = "alloc")]
impl<'a> ForwardViaObject<'a> for &Wrap<'a, alloc::boxed::Box<dyn Error + 'static>> {
    #[inline]
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)> {
        (**self.0).source()
    }
}

#[cfg(feature = "alloc")]
impl<'a> ForwardViaObject<'a> for &Wrap<'a, alloc::boxed::Box<dyn Error + Send + 'static>> {
    #[inline]
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)> {
        (**self.0).source()
    }
}

#[cfg(feature = "alloc")]
impl<'a> ForwardViaObject<'a> for &Wrap<'a, alloc::boxed::Box<dyn Error + Send + Sync + 'static>> {
    #[inline]
    fn __define_err_forward(&self) -> Option<&'a (dyn Error + 'static)> {
        (**self.0).source()
    }
}

/// `ErrorInfo` of the inner value of a `transparent` variant, when it has one.
#[doc(hidden)]
pub trait InfoViaErrorInfo<'a> {
    fn __define_err_info(&self) -> Option<&'a dyn ErrorInfo>;
}

impl<'a, E: ErrorInfo> InfoViaErrorInfo<'a> for &Wrap<'a, E> {
    #[inline]
    fn __define_err_info(&self) -> Option<&'a dyn ErrorInfo> {
        Some(self.0)
    }
}

/// The fallback: the inner value does not implement `ErrorInfo`.
#[doc(hidden)]
pub trait InfoViaNothing<'a> {
    fn __define_err_info(&self) -> Option<&'a dyn ErrorInfo>;
}

impl<'a, T: ?Sized> InfoViaNothing<'a> for Wrap<'a, T> {
    #[inline]
    fn __define_err_info(&self) -> Option<&'a dyn ErrorInfo> {
        None
    }
}

/// `source()` of a field: `$crate::__private::source_of!(field)`.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_source_of {
    ($field:expr) => {{
        #[allow(unused_imports)]
        use $crate::__private::{SourceViaError as _, SourceViaObject as _, SourceViaOption as _};
        (&&&$crate::__private::Wrap($field)).__define_err_source()
    }};
}

/// The inner value's `source()`, for `transparent` variants.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_forward_source {
    ($field:expr) => {{
        #[allow(unused_imports)]
        use $crate::__private::{ForwardViaError as _, ForwardViaObject as _};
        (&&$crate::__private::Wrap($field)).__define_err_forward()
    }};
}

/// `Option<&dyn ErrorInfo>` of a value, for `transparent` variants.
#[doc(hidden)]
#[macro_export]
macro_rules! __define_err_info_of {
    ($field:expr) => {{
        #[allow(unused_imports)]
        use $crate::__private::{InfoViaErrorInfo as _, InfoViaNothing as _};
        (&&$crate::__private::Wrap($field)).__define_err_info()
    }};
}

#[doc(hidden)]
pub use crate::{
    __define_err_forward_source as forward_source, __define_err_info_of as info_of,
    __define_err_source_of as source_of,
};

#[cfg(test)]
mod tests {
    extern crate std;

    use core::{error::Error, fmt};
    #[cfg(feature = "alloc")]
    use std::boxed::Box;

    use super::{forward_source, info_of, source_of};

    #[derive(Debug)]
    struct Leaf;

    impl fmt::Display for Leaf {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("leaf")
        }
    }

    impl Error for Leaf {}

    #[derive(Debug)]
    struct Node(Leaf);

    impl fmt::Display for Node {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("node")
        }
    }

    impl Error for Node {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.0)
        }
    }

    impl crate::ErrorInfo for Node {}

    #[test]
    fn test_source_of_plain_error_returns_it() {
        let leaf = Leaf;
        let source = source_of!(&leaf);
        assert_eq!(source.map(|e| std::format!("{e}")).as_deref(), Some("leaf"));
    }

    #[test]
    fn test_source_of_option_none_and_some() {
        let none: Option<Leaf> = None;
        assert!(source_of!(&none).is_none());
        let some = Some(Leaf);
        assert!(source_of!(&some).is_some());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_source_of_boxed_objects_return_inner() {
        let a: Box<dyn Error> = Box::new(Leaf);
        let b: Box<dyn Error + Send> = Box::new(Leaf);
        let c: Box<dyn Error + Send + Sync> = Box::new(Leaf);
        assert!(source_of!(&a).is_some_and(|e| e.is::<Leaf>()));
        assert!(source_of!(&b).is_some_and(|e| e.is::<Leaf>()));
        assert!(source_of!(&c).is_some_and(|e| e.is::<Leaf>()));
    }

    #[test]
    fn test_source_of_static_object_returns_it() {
        static LEAF: Leaf = Leaf;
        let object: &'static (dyn Error + 'static) = &LEAF;
        assert!(source_of!(&object).is_some());
    }

    #[test]
    fn test_forward_source_skips_the_wrapped_value() {
        let node = Node(Leaf);
        assert!(forward_source!(&node).is_some_and(|e| e.is::<Leaf>()));
        assert!(forward_source!(&Leaf).is_none());
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_forward_source_through_box() {
        let boxed: Box<dyn Error + Send + Sync> = Box::new(Node(Leaf));
        assert!(forward_source!(&boxed).is_some_and(|e| e.is::<Leaf>()));
    }

    #[test]
    fn test_info_of_finds_error_info_or_none() {
        assert!(info_of!(&Node(Leaf)).is_some());
        assert!(info_of!(&Leaf).is_none());
    }
}
