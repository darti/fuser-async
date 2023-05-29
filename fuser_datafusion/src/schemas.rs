use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use lazy_static::lazy_static;

#[cfg(not(feature = "large-binary"))]
use datafusion::arrow::array::BinaryArray;

#[cfg(not(feature = "large-binary"))]
pub type BinArray = BinaryArray;

#[cfg(not(feature = "large-binary"))]
pub const BINARY_TYPE: DataType = DataType::Binary;

#[cfg(feature = "large-binary")]
use datafusion::arrow::array::LargeBinaryArray;

#[cfg(feature = "large-binary")]
pub type BinArray = LargeBinaryArray;

#[cfg(feature = "large-binary")]
pub const BINARY_TYPE: DataType = DataType::LargeBinary;

pub const TIMESTAMP: DataType = DataType::Timestamp(TimeUnit::Microsecond, None);

lazy_static! {
    pub static ref METADATA_SCHEMA: SchemaRef = SchemaRef::new(Schema::new(vec![
        Field::new("ino", DataType::UInt64, false),
        Field::new("id", DataType::Utf8, false),
        Field::new("type", DataType::Utf8, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("parent_ino", DataType::UInt64, false),
        Field::new("atime", TIMESTAMP, false),
        Field::new("mtime", TIMESTAMP, false),
        Field::new("ctime", TIMESTAMP, false),
    ]));
    pub static ref CONTENT_SCHEMA: SchemaRef = SchemaRef::new(Schema::new(vec![
        Field::new("ino", DataType::UInt64, false),
        Field::new("size", DataType::UInt64, false),
        Field::new("content", BINARY_TYPE, true),
    ]));
}
