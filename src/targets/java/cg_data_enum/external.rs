use crate::ir::{DataEnum, DataEnumVariant, Type};
use crate::quote_iter;
use crate::targets::java::cg_utils::Compact;
use genco::prelude::*;
use genco::quote;

pub fn generate_external_serializer(de: &DataEnum) -> java::Tokens {
    let class_name_str = &de.name().as_pascal_case();
    let cases = quote_iter!(de.variants().iter() => |v: &DataEnumVariant| {
        let case = v.name().as_upper_snake_case();
        let json_name = v.json_name();
        let serializer: java::Tokens = match v {
            DataEnumVariant::Unit {..} => quote!(
                gen.writeString($[str]($[const](json_name)));
            ),
            DataEnumVariant::Object {..} => quote!(
                gen.writeStartObject();
                gen.writeObjectField($[str]($[const](json_name)), value.value);
                gen.writeEndObject();
            ),
            DataEnumVariant::Tuple { fields, ..} => {
                if fields.len() == 1 {
                    quote!(
                        gen.writeStartObject();
                        gen.writeObjectField($[str]($[const](json_name)), ((Object[]) value.value)[0]);
                        gen.writeEndObject();
                    )
                } else {
                    quote!(
                        gen.writeStartObject();
                        gen.writeObjectField($[str]($[const](json_name)), value.value);
                        gen.writeEndObject();
                    )
                }
            }
        };
        quote!(
            case $case: {
                $serializer
            }
            break;
        )
    });

    quote!(
        public static class Serializer extends StdSerializer<$class_name_str> {
            public Serializer() {
                super($class_name_str.class);
            }

            @Override
            public void serialize($class_name_str value, JsonGenerator gen, SerializerProvider provider) throws IOException {
                switch (value.getVariant()) {
                    $cases
                }
            }
        }
    )
}

pub fn generate_external_deserializer(de: &DataEnum) -> java::Tokens {
    let class_name_str = &de.name().as_pascal_case();
    let mut unit_cases: Vec<java::Tokens> = vec![];
    let mut object_cases: Vec<java::Tokens> = vec![];

    for variant in de.variants() {
        match variant {
            DataEnumVariant::Unit { .. } => unit_cases.push(deserialize_unit_variant(de, variant)),
            DataEnumVariant::Object { .. } => {
                object_cases.push(deserialize_object_variant(de, variant))
            }
            DataEnumVariant::Tuple { fields, .. } => {
                object_cases.push(deserialize_tuple_variant(de, variant, fields))
            }
        }
    }

    let unit_cases_code = unit_cases.compact();
    let object_cases_code = object_cases.compact();
    quote!(
        public static class Deserializer extends StdDeserializer<$class_name_str> {
            public Deserializer() {
                super($class_name_str.class);
            }

            private Object[] parseField(DeserializationContext cxtx, ObjectNode node, String key, TypeReference<?>...types) throws IOException {
                JsonNode field = node.get(key);
                if (field == null) {
                    return new Object[types.length];
                }
                if (field.isArray()) {
                    if (field.size() != types.length) {
                        throw new JsonParseException(cxtx.getParser(), "Expected array of size " + types.length + " for field " + key);
                    }
                    Object[] result = new Object[types.length];
                    for (int i = 0; i < types.length; i++) {
                        try (JsonParser parser = field.get(i).traverse(cxtx.getParser().getCodec())) {
                            result[i] = parser.readValueAs(types[i]);
                        }
                    }
                    return result;
                } else {
                    if (types.length != 1) {
                        throw new JsonParseException(cxtx.getParser(), "Expected array for field " + key);
                    }
                    try (JsonParser parser = field.traverse(cxtx.getParser().getCodec())) {
                        return new Object[]{parser.readValueAs(types[0])};
                    }
                }
            }

            @Override
            public $class_name_str deserialize(JsonParser p, DeserializationContext ctxt) throws IOException, JacksonException {

                if (p.currentToken() == JsonToken.VALUE_STRING) {
                    $unit_cases_code
                } else if (p.currentToken() == JsonToken.START_OBJECT) {
                    var node = (ObjectNode) p.getCodec().readTree(p);
                    $object_cases_code
                }
                throw ctxt.instantiationException($class_name_str.class, "Cannot deserialize " + $[str]($[const](class_name_str)));
            }
        }
    )
}

fn deserialize_unit_variant(de: &DataEnum, variant: &DataEnumVariant) -> java::Tokens {
    let base_name = &de.name().as_pascal_case();
    let case = variant.name().as_upper_snake_case();
    let json_name = variant.json_name();
    quote!(
        if (p.getText().equals($[str]($[const](json_name)))) {
            return new $base_name(Variant.$case, null);
        }
    )
}

fn deserialize_object_variant(de: &DataEnum, variant: &DataEnumVariant) -> java::Tokens {
    let base_name = &de.name().as_pascal_case();
    let case = variant.name().as_upper_snake_case();
    let class_name = variant.name().as_pascal_case();
    let json_name = variant.json_name();
    quote!(
        if (node.has($[str]($[const](json_name)))) {
            return new $base_name(Variant.$case, parseField(ctxt, node, $[str]($[const](json_name)), new TypeReference<$class_name>(){})[0]);
        }
    )
}

fn deserialize_tuple_variant(
    de: &DataEnum,
    variant: &DataEnumVariant,
    fields: &Vec<Type>,
) -> java::Tokens {
    let base_name = &de.name().as_pascal_case();
    let case = variant.name().as_upper_snake_case();
    let json_name = variant.json_name();
    let class_names = fields
        .iter()
        .map(|f: &Type| f.type_name())
        .collect::<Vec<&String>>();
    quote!(
        if (node.has($[str]($[const](json_name)))) {
            return new $base_name(Variant.$case, parseField(ctxt, node, $[str]($[const](json_name)), $(for c in class_names join (, ) => new TypeReference<$c>(){})));
        }
    )
}
