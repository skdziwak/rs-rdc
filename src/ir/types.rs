use crate::ir::{CustomType, Type};
use crate::targets::java::type_resolver::{JavaCustomType, JavaType};

/// This is a type that is used to mark a target type for IR data.
/// IR is mostly generic, but type resolution is target specific.
pub enum TypeTarget {
    Java,
}

impl TypeTarget {
    pub fn resolve_type<T>(&self) -> Type
    where
        T: JavaType,
    {
        match self {
            TypeTarget::Java => T::java_type(),
        }
    }

    pub fn resolve_custom_type<T>(&self) -> CustomType
    where
        T: JavaCustomType,
    {
        match self {
            TypeTarget::Java => T::java_custom_type(),
        }
    }
}
