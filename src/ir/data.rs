use crate::ir::TypeTarget;
use crate::RDCType;
use std::any::TypeId;
use std::collections::HashSet;

/// This is intermediate representation of the data.
/// It is used to generate code for other languages.
pub struct IntermediateRepresentation {
    structs: Vec<Struct>,
    enums: Vec<Enum>,
    data_enums: Vec<DataEnum>,
    target: TypeTarget,
    type_ids: HashSet<TypeId>,
}

/// This is a struct that is used to represent a struct in the IR.
/// It is usually translated to a class in other languages.
pub struct Struct {
    name: Name,
    self_type: CustomType,
    fields: Vec<Field>,
}

/// This is a regular enum representation in the IR.
/// It does not have any data associated with it.
pub struct Enum {
    name: Name,
    self_type: CustomType,
    variants: Vec<EnumVariant>,
}

/// This is a struct that represents a field in `Struct`.
/// It contains the name of the field, the type of the field and the name of the field in JSON.
pub struct Field {
    name: Name,
    json_name: String,
    field_type: Type,
}

/// This is a struct that represents a variant in `Enum`.
/// It contains the name of the variant and the name of the variant in JSON.
pub struct EnumVariant {
    name: Name,
    json_name: String,
}

/// This is a data enum representation in the IR.
pub struct DataEnum {
    name: Name,
    self_type: CustomType,
    style: DataEnumStyle,
    variants: Vec<DataEnumVariant>,
}

/// This enum represents type of data enum variant
pub enum DataEnumVariant {
    Unit {
        name: Name,
        json_name: String,
    },
    Object {
        name: Name,
        json_name: String,
        fields: Vec<DataEnumObjectField>,
    },
    Tuple {
        name: Name,
        json_name: String,
        fields: Vec<Type>,
    },
}

/// This struct represents a single field in a data enum object variant.
pub struct DataEnumObjectField {
    name: Name,
    json_name: String,
    field_type: Type,
}

/// This enum represents the style of a data enum.
/// Currently only external style is supported.
pub enum DataEnumStyle {
    External,
}

/// This is a basic type struct that is used to represent a type in the IR.
///
/// Despite IR is mostly language agnostic, types are language specific.
/// It is used for type resolution.
pub struct Type(String);

/// This is a custom type struct that is used to represent a type in the IR.
///
/// Despite IR is mostly language agnostic, types are language specific.
/// It is used for type code generation.
pub struct CustomType(String);

/// This is a struct that is used to represent a name in the IR.
/// It is used to generate code for other languages.
/// It is capable of converting multiple cases between each other.
pub struct Name {
    snake_case: String,
}

impl IntermediateRepresentation {
    pub fn new(target: TypeTarget) -> Self {
        Self {
            structs: Vec::new(),
            enums: Vec::new(),
            data_enums: Vec::new(),
            target,
            type_ids: HashSet::new(),
        }
    }

    pub fn add_struct(&mut self, s: Struct) {
        self.structs.push(s);
    }

    pub fn add_enum(&mut self, e: Enum) {
        self.enums.push(e);
    }

    pub fn add_data_enum(&mut self, de: DataEnum) {
        self.data_enums.push(de);
    }

    pub fn structs(&self) -> &[Struct] {
        &self.structs
    }

    pub fn enums(&self) -> &[Enum] {
        &self.enums
    }

    pub fn data_enums(&self) -> &[DataEnum] {
        &self.data_enums
    }

    pub fn target(&self) -> &TypeTarget {
        &self.target
    }

    pub fn add_type_id(&mut self, type_id: TypeId) {
        self.type_ids.insert(type_id);
    }

    pub fn has_type_id(&self, type_id: TypeId) -> bool {
        self.type_ids.contains(&type_id)
    }

    pub fn add<T: RDCType>(&mut self) {
        if !self.has_type_id(TypeId::of::<T>()) {
            self.add_type_id(TypeId::of::<T>());
            T::add_to_ir(self);
        }
    }
}

impl Struct {
    pub fn new(name: Name, self_type: CustomType) -> Self {
        Self {
            name,
            self_type,
            fields: Vec::new(),
        }
    }

    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn self_type(&self) -> &CustomType {
        &self.self_type
    }

    pub fn fields(&self) -> &Vec<Field> {
        &self.fields
    }
}

impl Enum {
    pub fn new(name: Name, self_type: CustomType) -> Self {
        Self {
            name,
            self_type,
            variants: Vec::new(),
        }
    }

    pub fn add_variant(&mut self, variant: EnumVariant) {
        self.variants.push(variant);
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn self_type(&self) -> &CustomType {
        &self.self_type
    }

    pub fn variants(&self) -> &Vec<EnumVariant> {
        &self.variants
    }
}

impl DataEnum {
    pub fn new(name: Name, self_type: CustomType, style: DataEnumStyle) -> Self {
        Self {
            name,
            self_type,
            variants: Vec::new(),
            style,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn self_type(&self) -> &CustomType {
        &self.self_type
    }

    pub fn add_variant(&mut self, variant: DataEnumVariant) {
        self.variants.push(variant);
    }

    pub fn variants(&self) -> &Vec<DataEnumVariant> {
        &self.variants
    }

    pub fn style(&self) -> &DataEnumStyle {
        &self.style
    }
}

impl Field {
    pub fn new<S: Into<String>>(name: Name, json_name: S, field_type: Type) -> Self {
        Self {
            name,
            json_name: json_name.into(),
            field_type,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn json_name(&self) -> &String {
        &self.json_name
    }

    pub fn field_type(&self) -> &Type {
        &self.field_type
    }
}

impl EnumVariant {
    pub fn new<S: Into<String>>(name: Name, json_name: S) -> Self {
        Self {
            name,
            json_name: json_name.into(),
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn json_name(&self) -> &String {
        &self.json_name
    }
}

impl DataEnumVariant {
    pub fn object(name: Name, json_name: String, fields: Vec<DataEnumObjectField>) -> Self {
        Self::Object {
            name,
            json_name,
            fields,
        }
    }

    pub fn tuple(name: Name, json_name: String, fields: Vec<Type>) -> Self {
        Self::Tuple {
            name,
            json_name,
            fields,
        }
    }

    pub fn unit(name: Name, json_name: String) -> Self {
        Self::Unit { name, json_name }
    }

    pub fn name(&self) -> &Name {
        match self {
            Self::Object { name, .. } => name,
            Self::Tuple { name, .. } => name,
            Self::Unit { name, .. } => name,
        }
    }

    pub fn json_name(&self) -> &String {
        match self {
            Self::Object { json_name, .. } => json_name,
            Self::Tuple { json_name, .. } => json_name,
            Self::Unit { json_name, .. } => json_name,
        }
    }
}

impl DataEnumObjectField {
    pub fn new(name: Name, json_name: String, field_type: Type) -> Self {
        Self {
            name,
            json_name,
            field_type,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }
    pub fn json_name(&self) -> &str {
        &self.json_name
    }
    pub fn field_type(&self) -> &Type {
        &self.field_type
    }
}

impl Type {
    pub fn new<S: Into<String>>(type_name: S) -> Self {
        Self(type_name.into())
    }

    pub fn type_name(&self) -> &String {
        &self.0
    }
}

impl CustomType {
    pub fn new<S: Into<String>>(type_name: S) -> Self {
        Self(type_name.into())
    }

    pub fn type_name(&self) -> &String {
        &self.0
    }
}

impl Name {
    pub fn from_snake_case<S: Into<String>>(snake_case: S) -> Self {
        Self {
            snake_case: snake_case.into(),
        }
    }

    pub fn from_camel_case<S: Into<String>>(camel_case: S) -> Self {
        let camel_case = camel_case.into();
        let mut snake_case = String::new();
        let mut chars = camel_case.chars();
        if let Some(first_char) = chars.next() {
            snake_case.push(first_char.to_ascii_lowercase());
        }
        for char in chars {
            if char.is_ascii_uppercase() {
                snake_case.push('_');
                snake_case.push(char.to_ascii_lowercase());
            } else {
                snake_case.push(char);
            }
        }
        Self { snake_case }
    }

    pub fn from_pascal_case<S: Into<String>>(pascal_case: S) -> Self {
        Self::from_camel_case(pascal_case)
    }

    pub fn as_snake_case(&self) -> String {
        self.snake_case.clone()
    }

    pub fn as_upper_snake_case(&self) -> String {
        self.snake_case.to_ascii_uppercase()
    }

    pub fn as_camel_case(&self) -> String {
        let mut camel_case = String::new();
        let mut chars = self.snake_case.chars().peekable();
        while let Some(char) = chars.next() {
            if char == '_' {
                if let Some(next_char) = chars.peek() {
                    camel_case.push(next_char.to_ascii_uppercase());
                    chars.next();
                }
            } else {
                camel_case.push(char);
            }
        }
        camel_case
    }

    pub fn as_pascal_case(&self) -> String {
        let mut camel_case = self.as_camel_case();
        if let Some(first_char) = camel_case.chars().next() {
            camel_case.replace_range(..1, &first_char.to_ascii_uppercase().to_string());
        }
        camel_case
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_name_from_snake_case() {
        let name = Name::from_snake_case("snake_case".to_string());
        assert_eq!(name.as_snake_case(), "snake_case");
        assert_eq!(name.as_camel_case(), "snakeCase");
        assert_eq!(name.as_pascal_case(), "SnakeCase");
        assert_eq!(name.as_upper_snake_case(), "SNAKE_CASE");
    }

    #[test]
    fn test_name_from_camel_case() {
        let name = Name::from_camel_case("camelCase".to_string());
        assert_eq!(name.as_snake_case(), "camel_case");
        assert_eq!(name.as_camel_case(), "camelCase");
        assert_eq!(name.as_pascal_case(), "CamelCase");
        assert_eq!(name.as_upper_snake_case(), "CAMEL_CASE");
    }

    #[test]
    fn test_name_from_pascal_case() {
        let name = Name::from_pascal_case("PascalCase".to_string());
        assert_eq!(name.as_snake_case(), "pascal_case");
        assert_eq!(name.as_camel_case(), "pascalCase");
        assert_eq!(name.as_pascal_case(), "PascalCase");
        assert_eq!(name.as_upper_snake_case(), "PASCAL_CASE");
    }
}
