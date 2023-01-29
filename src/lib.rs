//! # RDC
//! RDC - Rust Data Codegen
//!
//! This crate is used to generate code for other languages from Rust's data structures.
//! It can be used to generate DTO classes to make it easier to interact with other languages.
//!
//! It currently supports only Java, but it can be easily extended to support other languages in the future.
//!
//! It relies on the `serde` crate to serialize and deserialize data.
//!
//! For testing purposes it uses gradle to compile and run the generated Java code.
//!
//! ## Java Examples
//!
//! ### Simple struct
//!
//! ```rust
//! use rdc::{rdc_java, RDC};
//! use rdc::targets::java::JavaClass;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(RDC, Serialize, Deserialize)]
//! struct MyStruct {
//!     field1: String,
//!     field2: i32,
//! }
//!
//! #[derive(RDC)]
//! struct MyStruct2 {
//!     field1: Vec<String>,
//! }
//!
//! let classes: Vec<JavaClass> = rdc_java!(MyStruct, MyStruct2).unwrap();
//! ```
//!
//! ### Struct with dependencies
//! You do not have to specify all the types that are used in your data structures.
//! RDC will automatically add all the dependencies.
//!
//! ```rust
//! use rdc::{rdc_java, RDC};
//! use rdc::targets::java::JavaClass;
//!
//! #[derive(RDC)]
//! struct Dependency {
//!    field1: String,
//! }
//!
//! #[derive(RDC)]
//! struct MyStruct {
//!     field1: String,
//!     dependency: Dependency,
//! }
//!
//! let classes: Vec<JavaClass> = rdc_java!(MyStruct).unwrap();
//! assert_eq!(classes.len(), 2);
//! ```
//!
//! ### Enum
//!
//! ```rust
//! use rdc::{rdc_java, RDC};
//!
//! #[derive(RDC)]
//! enum MyEnum {
//!     Variant1,
//!     Variant2
//! }
//! rdc_java!(MyEnum).unwrap();
//! ```
//!
//! ### Data enum
//! Data enum is a special type of enum that can contain data.
//! ```rust
//! use rdc::{rdc_java, RDC};
//!
//! #[derive(RDC)]
//! enum MyEnum {
//!     Variant1(i32),
//!     Variant2,
//!     Variant3 { field1: String }
//! }
//!
//! rdc_java!(MyEnum).unwrap();
//! ```
//! Example JSON representations:
//! ```json
//! [
//!   {
//!     "Variant1": 1
//!   },
//!   "Variant2",
//!   {
//!     "Variant3": {
//!        "field1": "value"
//!     }
//!   }
//! ]
//! ```
//!
//! ### Generics
//! There is support for generics in RDC.
//! It works by generating a Java class for each combination of generic types.
//! Every used generic type should implement `rdc::RDCType` trait.
//! Please note that this trait is implemented automatically by `#[derive(RDC)]`.
//! ```rust
//! use rdc::{rdc_java, RDC, RDCType};
//!
//! #[derive(RDC)]
//! struct MyStruct<T> where T: RDCType {
//!     value: T,
//! }
//!
//! let classes = rdc_java!(MyStruct<i32>, MyStruct<String>).unwrap();
//! assert_eq!(classes.len(), 2);
//! ```
//!
//! ### Writing
//! RDC can write the generated code to files.
//! ```rust
//! use rdc::{rdc_java, RDC};
//! use rdc::targets::java::{JavaClass, write_java};
//!
//! #[derive(RDC)]
//! enum MyEnum {
//!     Variant1,
//!     Variant2
//! }
//! let classes: Vec<JavaClass> = rdc_java!(MyEnum).unwrap();
//!
//! write_java(&classes, "com.example", "target/test-tmp/src/main/java").unwrap();
//! ```
//!
//! ### Serde compatibility
//! You can use `#[serde(rename = "new_name")]` to rename fields and it will be reflected in the generated code.

use crate::codegen::GenerateIR;
use crate::targets::java::type_resolver::JavaType;
pub use rdc_macros::RDC;

/// This module defines GenerateIR trait and implementations for all primitive types used in IR.
pub mod codegen;

/// This module contains error types.
pub mod errors;

/// This module contains intermediate representation of the data.
pub mod ir;

/// This module contains all the programming language targets.
pub mod targets;

/// This is a type that is used to mark data structures that can be converted to IR.
/// It should be also implemented for all types that are used in IR.
pub trait RDCType: GenerateIR + JavaType + 'static {}
