use crate::ir::CustomType;
use crate::ir::Type;

/// This is a trait that is implemented by all types that can be converted to a Java type.
/// It is used to resolve the Java type of a given type.
pub trait JavaType {
    fn java_type() -> Type;
}

/// This is a trait that is implemented by types that would be implemented as a Java class.
/// It means that it will only be used for code generation and not for type resolution.
pub trait JavaCustomType {
    fn java_custom_type() -> CustomType;
}

macro_rules! bind_java_type {
    ($type:ty, $java_type:expr) => {
        impl JavaType for $type {
            fn java_type() -> Type {
                Type::new($java_type)
            }
        }
    };
}

bind_java_type!(bool, "Boolean");
bind_java_type!(i8, "Byte");
bind_java_type!(i16, "Short");
bind_java_type!(i32, "Integer");
bind_java_type!(i64, "Long");
bind_java_type!(f32, "Float");
bind_java_type!(f64, "Double");
bind_java_type!(String, "String");

impl<T> JavaType for Vec<T>
where
    T: JavaType,
{
    fn java_type() -> Type {
        Type::new(format!("java.util.List<{}>", T::java_type().type_name()))
    }
}

impl<T> JavaType for Option<T>
where
    T: JavaType,
{
    fn java_type() -> Type {
        T::java_type()
    }
}

impl<K, V> JavaType for std::collections::HashMap<K, V>
where
    K: JavaType,
    V: JavaType,
{
    fn java_type() -> Type {
        Type::new(format!(
            "java.util.Map<{}, {}>",
            K::java_type().type_name(),
            V::java_type().type_name()
        ))
    }
}

impl<T> JavaType for Box<T>
where
    T: JavaType,
{
    fn java_type() -> Type {
        T::java_type()
    }
}
