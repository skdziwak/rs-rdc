use crate::ir::IntermediateRepresentation;
use crate::RDCType;

/// This is a trait that adds all required structs and enums to the IR.
/// It is also implemented for all the primitive types that are supported natively by the target language.
/// Then `GenerateIR::add_to_ir` method is empty, because the type is already supported by the target language.
pub trait GenerateIR {
    fn add_to_ir(_ir: &mut IntermediateRepresentation) {}
}

macro_rules! rdc_type {
    ($type:ty) => {
        impl GenerateIR for $type {}
        impl RDCType for $type {}
    };
}

rdc_type!(bool);
rdc_type!(i8);
rdc_type!(i16);
rdc_type!(i32);
rdc_type!(i64);
rdc_type!(f32);
rdc_type!(f64);
rdc_type!(String);

impl<T> GenerateIR for Vec<T>
where
    T: RDCType,
{
    fn add_to_ir(ir: &mut IntermediateRepresentation) {
        T::add_to_ir(ir);
    }
}
impl<T> RDCType for Vec<T> where T: RDCType {}

impl<T> GenerateIR for Option<T>
where
    T: RDCType,
{
    fn add_to_ir(ir: &mut IntermediateRepresentation) {
        T::add_to_ir(ir);
    }
}
impl<T> RDCType for Option<T> where T: RDCType {}

impl<K, V> GenerateIR for std::collections::HashMap<K, V>
where
    K: RDCType,
    V: RDCType,
{
    fn add_to_ir(ir: &mut IntermediateRepresentation) {
        K::add_to_ir(ir);
        V::add_to_ir(ir);
    }
}
impl<K, V> RDCType for std::collections::HashMap<K, V>
where
    K: RDCType,
    V: RDCType,
{
}

impl<T> GenerateIR for Box<T>
where
    T: RDCType,
{
    fn add_to_ir(ir: &mut IntermediateRepresentation) {
        T::add_to_ir(ir);
    }
}
impl<T> RDCType for Box<T> where T: RDCType {}
