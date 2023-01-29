use crate::errors::Error;
use crate::ir::{DataEnum, DataEnumObjectField, DataEnumStyle, DataEnumVariant, Type};
use crate::quote_iter;
use crate::targets::java::cg_data_enum::external::{
    generate_external_deserializer, generate_external_serializer,
};
use crate::targets::java::cg_utils::Compact;
use crate::targets::java::JavaClass;
use genco::prelude::*;
use genco::quote;

mod external;

pub fn generate_enum_data_class(de: &DataEnum) -> Result<JavaClass, Error> {
    let class_name = de.name().as_pascal_case();
    let class_name_str = class_name.as_str();

    let serializer_code = match de.style() {
        DataEnumStyle::External => generate_external_serializer(de),
    };
    let deserializer_code = match de.style() {
        DataEnumStyle::External => generate_external_deserializer(de),
    };
    let fields_code = generate_fields_code(de);
    let variants_enum = generate_variants_enum(de);
    let tokens: java::Tokens = quote!(
        import com.fasterxml.jackson.annotation.JsonIgnore;
        import com.fasterxml.jackson.annotation.JsonInclude;
        import com.fasterxml.jackson.annotation.JsonProperty;
        import com.fasterxml.jackson.core.*;
        import com.fasterxml.jackson.databind.DeserializationContext;
        import com.fasterxml.jackson.databind.JsonNode;
        import com.fasterxml.jackson.databind.SerializerProvider;
        import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
        import com.fasterxml.jackson.databind.annotation.JsonSerialize;
        import com.fasterxml.jackson.databind.deser.std.StdDeserializer;
        import com.fasterxml.jackson.databind.node.ObjectNode;
        import com.fasterxml.jackson.databind.ser.std.StdSerializer;
        import com.fasterxml.jackson.core.type.TypeReference;

        import java.io.IOException;

        @JsonInclude(JsonInclude.Include.NON_NULL)
        @JsonSerialize(using = $class_name_str.Serializer.class)
        @JsonDeserialize(using = $class_name_str.Deserializer.class)
        public class $class_name_str {
            @JsonIgnore
            private final Variant variant;

            @JsonIgnore
            private final Object value;

            private $class_name_str(Variant variant, Object value) {
                this.variant = variant;
                this.value = value;
            }

            $fields_code

            $serializer_code

            $deserializer_code

            $variants_enum

            Variant getVariant() {
                return variant;
            }
        }
    );

    JavaClass::from_tokens(class_name, tokens)
}

fn generate_variants_enum(de: &DataEnum) -> java::Tokens {
    let variants = de
        .variants()
        .iter()
        .map(|v: &DataEnumVariant| {
            let name = v.name().as_upper_snake_case();
            quote!($name)
        })
        .collect::<Vec<java::Tokens>>();
    let variants_enum = quote!(
        public enum Variant {
            $(for v in variants join (,) => $v)
        }
    );
    variants_enum
}

fn generate_fields_code(de: &DataEnum) -> java::Tokens {
    let contents: Vec<java::Tokens> = de
        .variants()
        .iter()
        .map(|v| generate_field_code(de, v))
        .collect();

    quote!(
        $(for c in contents => $c)
    )
}

fn generate_field_code(de: &DataEnum, v: &DataEnumVariant) -> java::Tokens {
    let class_name = &de.name().as_pascal_case();
    let name = v.name();
    let variant_enum_name = &name.as_upper_snake_case();
    let of_method_name = &format!("of{}", name.as_pascal_case());
    let is_method_name = &format!("is{}", name.as_pascal_case());
    let variant_specific_code: java::Tokens = match v {
        DataEnumVariant::Unit { .. } => {
            quote!(
                public static $class_name $of_method_name() {
                    return new $class_name(Variant.$variant_enum_name, null);
                }
            )
        }
        DataEnumVariant::Object { fields, .. } => {
            let sub_class_name = &name.as_pascal_case();
            let field_names = &fields
                .iter()
                .map(|f: &DataEnumObjectField| f.name().as_camel_case())
                .collect::<Vec<String>>();
            let field_declarations = &fields
                .iter()
                .map(|f: &DataEnumObjectField| {
                    let field_name = f.name().as_camel_case();
                    let field_type = f.field_type().type_name();
                    format!("{field_type} {field_name}")
                })
                .collect::<Vec<String>>();
            let field_args = &fields
                .iter()
                .map(|f: &DataEnumObjectField| {
                    let field_name = f.name().as_camel_case();
                    let field_type = f.field_type().type_name();
                    let json_name = f.json_name();
                    quote!(
                        @JsonProperty($[str]($[const](json_name))) $field_type $field_name
                    )
                })
                .collect::<Vec<java::Tokens>>();
            let getters = &fields
                .iter()
                .map(|f: &DataEnumObjectField| {
                    let field_name = f.name().as_camel_case();
                    let getter_name = &format!("get{}", f.name().as_pascal_case());
                    let field_type = f.field_type().type_name();
                    let json_name = f.json_name();
                    quote!(
                        @JsonProperty($[str]($[const](json_name)))
                        public $field_type $getter_name() {
                            return $field_name;
                        }
                    )
                })
                .collect::<Vec<java::Tokens>>();
            let main_getter_name = &format!("get{}", name.as_pascal_case());
            let enum_field_name = &name.as_upper_snake_case();
            quote!(
                public static $class_name $of_method_name($sub_class_name value) {
                    return new $class_name(Variant.$variant_enum_name, value);
                }

                public $sub_class_name $main_getter_name() {
                    if (variant != Variant.$enum_field_name) {
                        throw new IllegalStateException("Invalid variant: " + variant);
                    }
                    return ($sub_class_name) value;
                }

                public static class $sub_class_name {
                    $(for fd in field_declarations => private final $fd;)

                    public $sub_class_name($(for fd in field_args join (,) => $fd)) {
                        $(for f in field_names => this.$f = $f;)
                    }

                    $(for g in getters => $g)
                }
            )
        }
        DataEnumVariant::Tuple { fields, .. } => {
            let mut counter = 0;
            let args = fields
                .iter()
                .map(|t: &Type| {
                    let arg_name = format!("arg{counter}");
                    let type_name = t.type_name();
                    counter += 1;
                    quote!($type_name $arg_name)
                })
                .collect::<Vec<java::Tokens>>();

            let mut counter = 0;
            let objects = fields
                .iter()
                .map(|_t: &Type| {
                    let arg_name = format!("arg{counter}");
                    counter += 1;
                    quote!($arg_name)
                })
                .collect::<Vec<java::Tokens>>();

            let getter_numbering = fields.len() != 1;
            let mut counter = 0;
            let variant_enum_name = &name.as_upper_snake_case();
            let getters = quote_iter!(fields.iter() => |f: &Type| {
                let type_name = f.type_name();
                let getter_name = if getter_numbering {
                    format!("get{}{}", name.as_pascal_case(), counter)
                } else {
                    format!("get{}", name.as_pascal_case())
                };
                let getter = quote!(
                    @SuppressWarnings("unchecked")
                    public $type_name $getter_name() {
                        if (variant != Variant.$variant_enum_name) {
                            throw new IllegalStateException("Invalid variant: " + variant);
                        }
                        return ($type_name) ((Object[]) value)[$counter];
                    }
                );
                counter += 1;
                getter
            });

            quote!(
                public static $class_name $of_method_name($(for a in args join (,) => $a)) {
                    return new $class_name(Variant.$variant_enum_name, new Object[] {
                        $(for o in objects join (,) => $o)
                    });
                }

                $getters
            )
        }
    };
    quote!(
        $variant_specific_code

        public boolean $is_method_name() {
            return variant == Variant.$variant_enum_name;
        }
    )
}

#[cfg(test)]
mod tests {
    use crate as rdc;
    use crate::targets::java::tests::run_java;
    use crate::targets::java::JavaClass;
    use crate::{rdc_java, RDCType, RDC};
    use genco::quote;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    #[derive(RDC, Serialize, Deserialize, PartialEq, Debug)]
    enum TestEnum<T>
    where
        T: RDCType + PartialEq + Debug,
    {
        Csv(String),
        #[serde(rename = "JSON")]
        Json(i32),
        #[serde(rename = "XML")]
        Xml(f64, i32),
        #[serde(rename = "YAML")]
        Yaml(T),
        Other {
            #[serde(rename = "other")]
            name: String,
        },
        Unit,
        List(Vec<i32>),
        Nested(Box<TestEnum<T>>),
    }

    #[test]
    fn enum_derive_test() {
        let mut classes = rdc_java!(TestEnum<i32>).unwrap();
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
                    TestEnumInteger b = objectMapper.readValue(input, new TypeReference<TestEnumInteger>() {});
                    if (b.isOther()) {
                        var other = b.getOther();
                        assert other.getName().equals("test");
                    }
                    if (b.isYaml()) {
                        var yaml = b.getYaml();
                        assert yaml == 42;
                    }
                    if (b.isXml()) {
                        assert b.getXml0() == 3.13467;
                        assert b.getXml1() == 57;
                    }
                    if (b.isNested()) {
                        var nested = b.getNested();
                        assert nested.isUnit();
                    }
                    String serialized = objectMapper.writeValueAsString(b);
                    TestEnumInteger b2 = objectMapper.readValue(serialized, new TypeReference<TestEnumInteger>() {});
                    String serialized2 = objectMapper.writeValueAsString(b2);
                    System.out.print(serialized2);
                }
            }
        )).unwrap());
        let enums = vec![
            TestEnum::Csv("test".to_string()),
            TestEnum::Json(42),
            TestEnum::Xml(3.13467, 57),
            TestEnum::Yaml(42),
            TestEnum::Other {
                name: "test".to_string(),
            },
            TestEnum::Unit,
            TestEnum::Nested(Box::new(TestEnum::Unit)),
        ];
        for e in enums {
            let serialized = serde_json::to_string(&e).unwrap();
            let processed = run_java(&classes, serialized.as_str()).unwrap();
            let deserialized: TestEnum<i32> = serde_json::from_str(&processed).unwrap();
            assert_eq!(e, deserialized);
        }
    }
}
