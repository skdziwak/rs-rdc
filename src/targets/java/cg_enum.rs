use crate::errors::Error;
use crate::ir::{Enum, EnumVariant};
use crate::targets::java::JavaClass;
use genco::quote;

pub fn generate_enum_class(enum_ir: &Enum) -> Result<JavaClass, Error> {
    let class_name = enum_ir.name().as_pascal_case();
    let class_name_str = class_name.as_str();
    let variants = enum_ir.variants().iter().map(|variant: &EnumVariant| {
        let name = variant.name().as_upper_snake_case();
        let json_name = variant.json_name();
        quote!(
            @JsonProperty($[str]($[const](json_name)))
            $name
        )
    });

    let tokens = quote!(
        import com.fasterxml.jackson.annotation.JsonProperty;

        public enum $class_name_str {
            $(for v in variants join (,) => $v)
        }
    );

    JavaClass::from_tokens(class_name, tokens)
}

#[cfg(test)]
mod tests {
    use crate as rdc;
    use crate::ir::TypeTarget::Java;
    use crate::ir::{CustomType, Enum, EnumVariant, IntermediateRepresentation, Name};
    use crate::targets::java::tests::run_java;
    use crate::targets::java::{generate_java_code, JavaClass};
    use crate::RDC;
    use genco::quote;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    #[test]
    fn generate_basic_class() {
        let mut ir = IntermediateRepresentation::new(Java);
        let mut enum_ir = Enum::new(
            Name::from_pascal_case("TestEnum"),
            CustomType::new("TestEnum"),
        );
        enum_ir.add_variant(EnumVariant::new(
            Name::from_pascal_case("Test"),
            "TEST_OPTION",
        ));
        enum_ir.add_variant(EnumVariant::new(
            Name::from_pascal_case("Test2"),
            "TEST2_OPTION",
        ));
        ir.add_enum(enum_ir);

        let classes = generate_java_code(&ir).unwrap();
        assert_eq!(classes.len(), 1);
        let class = &classes[0];
        assert_eq!(class.name(), "TestEnum");
        println!("{}", classes[0].code());
    }

    #[derive(RDC, Serialize, Deserialize, PartialEq, Debug)]
    enum ExportType {
        #[serde(rename = "CSV")]
        Csv,
        Json,
        #[serde(rename = "XML")]
        Xml,
    }

    #[derive(RDC, Serialize, Deserialize, PartialEq, Debug)]
    struct Value {
        value: ExportType,
    }

    #[test]
    fn enum_derive_test() {
        let mut ir = IntermediateRepresentation::new(Java);
        ir.add::<Value>();
        let mut classes = generate_java_code(&ir).unwrap();
        for class in &classes {
            println!("{}", class.code());
        }
        classes.push(JavaClass::from_tokens("Main".to_string(), quote!(
            import com.fasterxml.jackson.databind.ObjectMapper;
            import com.fasterxml.jackson.core.type.TypeReference;
            public class Main {
                public static void main(String[] args) throws Exception {
                    var objectMapper = new ObjectMapper();
                    String input = Utils.input();
                    Value b = objectMapper.readValue(input, new TypeReference<Value>() {});
                    String serialized = objectMapper.writeValueAsString(b);
                    Value b2 = objectMapper.readValue(serialized, new TypeReference<Value>() {});
                    String serialized2 = objectMapper.writeValueAsString(b2);
                    System.out.print(serialized2);
                }
            }
        )).unwrap());
        let values = vec![
            Value {
                value: ExportType::Csv,
            },
            Value {
                value: ExportType::Json,
            },
            Value {
                value: ExportType::Xml,
            },
        ];
        for e in values {
            let serialized = serde_json::to_string(&e).unwrap();
            let processed = run_java(&classes, serialized.as_str()).unwrap();
            let deserialized: Value = serde_json::from_str(&processed).unwrap();
            assert_eq!(e, deserialized);
        }
    }
}
