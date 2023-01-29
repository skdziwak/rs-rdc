use crate::errors::Error;
use crate::ir::{Field, Struct};
use crate::targets::java::JavaClass;
use genco::prelude::*;
use genco::quote;

fn generate_field_declaration(field: &Field) -> java::Tokens {
    let name = field.name().as_camel_case();
    let json_name = field.json_name();
    let type_string = field.field_type().type_name();
    quote! {
        @JsonProperty($[str]($[const](json_name)))
        private $type_string $name;
    }
}

fn generate_access_methods(field: &Field) -> java::Tokens {
    let pascal_name = &field.name().as_pascal_case();
    let camel_name = &field.name().as_camel_case();
    let type_string = field.field_type().type_name();
    quote! {
        public $type_string get$pascal_name() {
            return $camel_name;
        }

        public void set$pascal_name($type_string $camel_name) {
            this.$camel_name = $camel_name;
        }
    }
}

pub fn generate_data_class(dc: &Struct) -> Result<JavaClass, Error> {
    let class_name = dc.name().as_pascal_case();
    let class_name_str = class_name.as_str();
    let field_declarations = dc.fields().iter().map(generate_field_declaration);
    let access_methods = dc.fields().iter().map(generate_access_methods);
    let class_custom_type = dc.self_type().type_name();
    let tokens: java::Tokens = quote!(
        import com.fasterxml.jackson.annotation.JsonProperty;

        public class $class_custom_type {
            $(for fd in field_declarations => $fd)

            public $class_name_str() {}

            $(for am in access_methods => $am)
        }
    );

    JavaClass::from_tokens(class_name, tokens)
}

#[cfg(test)]
mod tests {
    use crate as rdc;
    use crate::ir::TypeTarget::Java;
    use crate::ir::{CustomType, Field, IntermediateRepresentation, Name, Struct, Type};
    use crate::targets::java::tests::run_java;
    use crate::targets::java::{generate_java_code, JavaClass};
    use crate::RDCType;
    use genco::quote;
    use rdc_macros::RDC;
    use serde::{Deserialize, Serialize};

    #[test]
    fn generate_basic_class() {
        let mut ir = IntermediateRepresentation::new(Java);
        let mut struct_ir = Struct::new(
            Name::from_pascal_case("TestStruct"),
            CustomType::new("TestStruct"),
        );

        let field1 = Field::new(
            Name::from_snake_case("complex_name"),
            "otherName",
            Type::new("String"),
        );
        struct_ir.add_field(field1);

        ir.add_struct(struct_ir);

        let classes = generate_java_code(&ir).unwrap();
        assert_eq!(classes.len(), 1);
        let class = &classes[0];
        assert_eq!(class.name(), "TestStruct");
        println!("{}", classes[0].code());
    }

    #[derive(RDC, Serialize, Deserialize)]
    struct A {
        a: i32,
        b: i32,
    }

    #[derive(RDC, Serialize, Deserialize)]
    struct B<K, V>
    where
        K: RDCType,
        V: RDCType,
    {
        k: K,
        #[serde(rename = "hello")]
        v: V,
        a: A,
    }

    #[derive(RDC, Serialize, Deserialize)]
    struct C<X>
    where
        X: RDCType,
    {
        x: X,
        b: B<X, f64>,
    }

    #[test]
    fn struct_derive_test() {
        let mut ir = IntermediateRepresentation::new(Java);
        ir.add::<B<i32, f64>>();
        let mut classes = generate_java_code(&ir).unwrap();
        classes.push(JavaClass::from_tokens("Main".to_string(), quote!(
            import com.fasterxml.jackson.databind.ObjectMapper;
            import com.fasterxml.jackson.core.type.TypeReference;
            public class Main {
                public static void main(String[] args) throws Exception {
                    var objectMapper = new ObjectMapper();
                    String input = Utils.input();
                    BIntegerDouble b = objectMapper.readValue(input, new TypeReference<BIntegerDouble>() {});
                    String serialized = objectMapper.writeValueAsString(b);
                    BIntegerDouble b2 = objectMapper.readValue(serialized, new TypeReference<BIntegerDouble>() {});
                    String serialized2 = objectMapper.writeValueAsString(b2);
                    System.out.print(serialized2);
                }
            }
        )).unwrap());
        assert_eq!(classes.len(), 3);
        for class in &classes {
            println!("{}", class.code());
        }
        let b: B<i32, f64> = B {
            k: 1,
            v: 1.0,
            a: A { a: 1, b: 2 },
        };
        let b_json = serde_json::to_string(&b).unwrap();
        let result = run_java(&classes, b_json.as_str());
        match result {
            Ok(value) => {
                let read_b = serde_json::from_str::<B<i32, f64>>(&value).unwrap();
                assert_eq!(b.a.a, read_b.a.a);
                assert_eq!(b.a.b, read_b.a.b);
                assert_eq!(b.k, read_b.k);
                assert_eq!(b.v, read_b.v);
            }
            Err(e) => panic!("{}", e.message()),
        }
    }
}
