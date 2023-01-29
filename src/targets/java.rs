use crate::errors::Error;
use crate::ir::IntermediateRepresentation;
use std::io::Write;

mod cg_data_enum;
mod cg_enum;
mod cg_struct;
mod cg_utils;
pub mod type_resolver;

#[cfg(test)]
mod tests;

/// This is a struct that represents a generated Java class.
pub struct JavaClass {
    name: String,
    code: String,
}

impl JavaClass {
    pub fn new(name: String, code: String) -> Self {
        JavaClass { name, code }
    }

    pub fn from_tokens(name: String, tokens: genco::prelude::java::Tokens) -> Result<Self, Error> {
        let code = tokens
            .to_string()
            .map_err(|_| Error::new("Failed to generate Java code"))?;
        Ok(JavaClass::new(name, code))
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn code(&self) -> &str {
        &self.code
    }
}

/// This function generates Java code from an IntermediateRepresentation.
/// Result can be saved to files and compiled.
pub fn generate_java_code(ir: &IntermediateRepresentation) -> Result<Vec<JavaClass>, Error> {
    let mut classes = Vec::new();
    for struct_ir in ir.structs() {
        classes.push(cg_struct::generate_data_class(struct_ir)?);
    }
    for enum_ir in ir.enums() {
        classes.push(cg_enum::generate_enum_class(enum_ir)?);
    }
    for data_enum_ir in ir.data_enums() {
        classes.push(cg_data_enum::generate_enum_data_class(data_enum_ir)?);
    }
    Ok(classes)
}

/// This macro generates code for all the provided types and their dependencies.
///
/// Example:
/// ```rust
/// use rdc::{rdc_java, RDC};
///
/// #[derive(RDC)]
/// struct MyStruct {
///     field1: String,
///     dependency: Dependency
/// }
///
/// #[derive(RDC)]
/// struct Dependency {
///     field1: String,
/// }
///
/// #[derive(RDC)]
/// enum MyEnum {
///    Variant1,
///    Variant2
/// }
///
/// let classes = rdc_java!(MyStruct, MyEnum);
/// assert!(classes.is_ok());
/// assert_eq!(classes.unwrap().len(), 3);
/// ```
#[macro_export]
macro_rules! rdc_java {
    ($($type:ty),*) => {
        {
            let mut ir = $crate::ir::IntermediateRepresentation::new($crate::ir::TypeTarget::Java);
            $(
                ir.add::<$type>();
            )*
            $crate::targets::java::generate_java_code(&ir)
        }
    };
}

/// This function writes generated Java code to files.
/// ```rust
/// use rdc::targets::java::{JavaClass, write_java};
/// use rdc::errors::Error;
/// fn write_example(classes: Vec<JavaClass>) -> Result<(), Error> {
///     write_java(classes.as_slice(), "com.example", "src/main/java")
/// }
/// ```
/// This will create a directory structure like this:
/// ```text
/// src/main/java
/// ├── com
/// │   └── example
/// │       ├── Dependency.java
/// │       ├── MyEnum.java
/// │       └── MyStruct.java
/// ```
pub fn write_java(classes: &[JavaClass], package: &str, directory: &str) -> Result<(), Error> {
    let mut path = std::path::PathBuf::from(directory);
    path.push(package.replace('.', "/"));
    std::fs::create_dir_all(&path).map_err(|_| Error::new("Failed to create directory"))?;
    for class in classes {
        let mut file_path = path.clone();
        file_path.push(format!("{}.java", class.name()));
        let mut file =
            std::fs::File::create(file_path).map_err(|_| Error::new("Failed to create file"))?;
        let code = format!("package {};\n\n{}", package, class.code());
        file.write_all(code.as_bytes())
            .map_err(|_| Error::new("Failed to write to file"))?;
    }
    Ok(())
}

#[cfg(test)]
mod writer_tests {
    #[allow(unused_imports)]
    use super::*;
    use crate as rdc;
    use crate::targets::java::write_java;
    use crate::{rdc_java, RDC};

    #[derive(RDC)]
    #[allow(unused)]
    struct MyStruct {
        field1: String,
        dependency: Dependency,
    }

    #[derive(RDC)]
    #[allow(unused)]
    struct Dependency {
        field1: String,
    }

    #[derive(RDC)]
    #[allow(unused)]
    enum MyEnum {
        Variant1,
        Variant2,
    }

    #[test]
    fn test_generate_java_code() {
        let results = rdc_java!(MyStruct, MyEnum);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 3);
    }

    #[test]
    fn test_write_java() {
        let results = rdc_java!(MyStruct, MyEnum);
        assert!(results.is_ok());
        assert!(write_java(
            &results.unwrap(),
            "com.example",
            "target/test-tmp/src/main/java"
        )
        .is_ok());
        assert!(
            std::path::Path::new("target/test-tmp/src/main/java/com/example/MyStruct.java")
                .exists()
        );
        assert!(
            std::path::Path::new("target/test-tmp/src/main/java/com/example/MyEnum.java").exists()
        );
        assert!(
            std::path::Path::new("target/test-tmp/src/main/java/com/example/Dependency.java")
                .exists()
        );
    }
}
