// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Contains Rust mappings for Thrift definition.
//! Refer to `parquet.thrift` file to see raw definitions.

use std::convert;
use std::fmt;
use std::result;
use std::str;

use errors::ParquetError;
use parquet_format as parquet;

// ----------------------------------------------------------------------
// Types from the Thrift definition

// ----------------------------------------------------------------------
// Mirrors `parquet::Type`

/// Types supported by Parquet.
/// These physical types are intended to be used in combination with the encodings to
/// control the on disk storage format.
/// For example INT16 is not included as a type since a good encoding of INT32
/// would handle this.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
  BOOLEAN,
  INT32,
  INT64,
  INT96,
  FLOAT,
  DOUBLE,
  BYTE_ARRAY,
  FIXED_LEN_BYTE_ARRAY
}

// ----------------------------------------------------------------------
// Mirrors `parquet::ConvertedType`

/// Common types (logical types) used by frameworks when using Parquet.
/// This helps map between types in those frameworks to the base types in Parquet.
/// This is only metadata and not needed to read or write the data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalType {
  NONE,
  /// A BYTE_ARRAY actually contains UTF8 encoded chars.
  UTF8,

  /// A map is converted as an optional field containing a repeated key/value pair.
  MAP,

  /// A key/value pair is converted into a group of two fields.
  MAP_KEY_VALUE,

  /// A list is converted into an optional field containing a repeated field for its
  /// values.
  LIST,

  /// An enum is converted into a binary field
  ENUM,

  /// A decimal value.
  /// This may be used to annotate binary or fixed primitive types. The
  /// underlying byte array stores the unscaled value encoded as two's
  /// complement using big-endian byte order (the most significant byte is the
  /// zeroth element).
  ///
  /// This must be accompanied by a (maximum) precision and a scale in the
  /// SchemaElement. The precision specifies the number of digits in the decimal
  /// and the scale stores the location of the decimal point. For example 1.23
  /// would have precision 3 (3 total digits) and scale 2 (the decimal point is
  /// 2 digits over).
  DECIMAL,

  /// A date stored as days since Unix epoch, encoded as the INT32 physical type.
  DATE,

  /// The total number of milliseconds since midnight. The value is stored as an INT32
  /// physical type.
  TIME_MILLIS,

  /// The total number of microseconds since midnight. The value is stored as an INT64
  /// physical type.
  TIME_MICROS,

  /// Date and time recorded as milliseconds since the Unix epoch.
  /// Recorded as a physical type of INT64.
  TIMESTAMP_MILLIS,

  /// Date and time recorded as microseconds since the Unix epoch.
  /// The value is stored as an INT64 physical type.
  TIMESTAMP_MICROS,

  /// An unsigned 8 bit integer value stored as INT32 physical type.
  UINT_8,

  /// An unsigned 16 bit integer value stored as INT32 physical type.
  UINT_16,

  /// An unsigned 32 bit integer value stored as INT32 physical type.
  UINT_32,

  /// An unsigned 64 bit integer value stored as INT64 physical type.
  UINT_64,

  /// A signed 8 bit integer value stored as INT32 physical type.
  INT_8,

  /// A signed 16 bit integer value stored as INT32 physical type.
  INT_16,

  /// A signed 32 bit integer value stored as INT32 physical type.
  INT_32,

  /// A signed 64 bit integer value stored as INT64 physical type.
  INT_64,

  /// A JSON document embedded within a single UTF8 column.
  JSON,

  /// A BSON document embedded within a single BINARY column.
  BSON,

  /// An interval of time.
  ///
  /// This type annotates data stored as a FIXED_LEN_BYTE_ARRAY of length 12.
  /// This data is composed of three separate little endian unsigned integers.
  /// Each stores a component of a duration of time. The first integer identifies
  /// the number of months associated with the duration, the second identifies
  /// the number of days associated with the duration and the third identifies
  /// the number of milliseconds associated with the provided duration.
  /// This duration of time is independent of any particular timezone or date.
  INTERVAL
}

// ----------------------------------------------------------------------
// Mirrors `parquet::FieldRepetitionType`

/// Representation of field types in schema.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Repetition {
  /// Field is required (can not be null) and each record has exactly 1 value.
  REQUIRED,
  /// Field is optional (can be null) and each record has 0 or 1 values.
  OPTIONAL,
  /// Field is repeated and can contain 0 or more values.
  REPEATED
}

// ----------------------------------------------------------------------
// Mirrors `parquet::Encoding`

/// Encodings supported by Parquet.
/// Not all encodings are valid for all types. These enums are also used to specify the
/// encoding of definition and repetition levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
  /// Default byte encoding.
  /// - BOOLEAN - 1 bit per value, 0 is false; 1 is true.
  /// - INT32 - 4 bytes per value, stored as little-endian.
  /// - INT64 - 8 bytes per value, stored as little-endian.
  /// - FLOAT - 4 bytes per value, stored as little-endian.
  /// - DOUBLE - 8 bytes per value, stored as little-endian.
  /// - BYTE_ARRAY - 4 byte length stored as little endian, followed by bytes.
  /// - FIXED_LEN_BYTE_ARRAY - just the bytes are stored.
  PLAIN,

  /// **Deprecated** dictionary encoding.
  ///
  /// The values in the dictionary are encoded using PLAIN encoding.
  /// Since it is deprecated, RLE_DICTIONARY encoding is used for a data page, and PLAIN
  /// encoding is used for dictionary page.
  PLAIN_DICTIONARY,

  /// Group packed run length encoding.
  ///
  /// Usable for definition/repetition levels encoding and boolean values.
  RLE,

  /// Bit packed encoding.
  ///
  /// This can only be used if the data has a known max width.
  /// Usable for definition/repetition levels encoding.
  BIT_PACKED,

  /// Delta encoding for integers, either INT32 or INT64.
  ///
  /// Works best on sorted data.
  DELTA_BINARY_PACKED,

  /// Encoding for byte arrays to separate the length values and the data.
  ///
  /// The lengths are encoded using DELTA_BINARY_PACKED encoding.
  DELTA_LENGTH_BYTE_ARRAY,

  /// Incremental encoding for byte arrays.
  ///
  /// Prefix lengths are encoded using DELTA_BINARY_PACKED encoding.
  /// Suffixes are stored using DELTA_LENGTH_BYTE_ARRAY encoding.
  DELTA_BYTE_ARRAY,

  /// Dictionary encoding.
  ///
  /// The ids are encoded using the RLE encoding.
  RLE_DICTIONARY
}

// ----------------------------------------------------------------------
// Mirrors `parquet::CompressionCodec`

/// Supported compression algorithms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Compression {
  UNCOMPRESSED,
  SNAPPY,
  GZIP,
  LZO,
  BROTLI,
  LZ4,
  ZSTD
}

// ----------------------------------------------------------------------
// Mirrors `parquet::PageType`

/// Available data pages for Parquet file format.
/// Note that some of the page types may not be supported.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageType {
  DATA_PAGE,
  INDEX_PAGE,
  DICTIONARY_PAGE,
  DATA_PAGE_V2
}

impl fmt::Display for Type {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl fmt::Display for LogicalType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl fmt::Display for Repetition {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl fmt::Display for Encoding {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl fmt::Display for Compression {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl fmt::Display for PageType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl convert::From<parquet::Type> for Type {
  fn from(tp: parquet::Type) -> Self {
    match tp {
      parquet::Type::BOOLEAN => Type::BOOLEAN,
      parquet::Type::INT32 => Type::INT32,
      parquet::Type::INT64 => Type::INT64,
      parquet::Type::INT96 => Type::INT96,
      parquet::Type::FLOAT => Type::FLOAT,
      parquet::Type::DOUBLE => Type::DOUBLE,
      parquet::Type::BYTE_ARRAY => Type::BYTE_ARRAY,
      parquet::Type::FIXED_LEN_BYTE_ARRAY => Type::FIXED_LEN_BYTE_ARRAY
    }
  }
}

impl convert::From<Option<parquet::ConvertedType>> for LogicalType {
  fn from(op: Option<parquet::ConvertedType>) -> Self {
    match op {
      None => LogicalType::NONE,
      Some(tp) => {
        match tp {
          parquet::ConvertedType::UTF8 => LogicalType::UTF8,
          parquet::ConvertedType::MAP => LogicalType::MAP,
          parquet::ConvertedType::MAP_KEY_VALUE => LogicalType::MAP_KEY_VALUE,
          parquet::ConvertedType::LIST => LogicalType::LIST,
          parquet::ConvertedType::ENUM => LogicalType::ENUM,
          parquet::ConvertedType::DECIMAL => LogicalType::DECIMAL,
          parquet::ConvertedType::DATE => LogicalType::DATE,
          parquet::ConvertedType::TIME_MILLIS => LogicalType::TIME_MILLIS,
          parquet::ConvertedType::TIME_MICROS => LogicalType::TIME_MICROS,
          parquet::ConvertedType::TIMESTAMP_MILLIS => LogicalType::TIMESTAMP_MILLIS,
          parquet::ConvertedType::TIMESTAMP_MICROS => LogicalType::TIMESTAMP_MICROS,
          parquet::ConvertedType::UINT_8 => LogicalType::UINT_8,
          parquet::ConvertedType::UINT_16 => LogicalType::UINT_16,
          parquet::ConvertedType::UINT_32 => LogicalType::UINT_32,
          parquet::ConvertedType::UINT_64 => LogicalType::UINT_64,
          parquet::ConvertedType::INT_8 => LogicalType::INT_8,
          parquet::ConvertedType::INT_16 => LogicalType::INT_16,
          parquet::ConvertedType::INT_32 => LogicalType::INT_32,
          parquet::ConvertedType::INT_64 => LogicalType::INT_64,
          parquet::ConvertedType::JSON => LogicalType::JSON,
          parquet::ConvertedType::BSON => LogicalType::BSON,
          parquet::ConvertedType::INTERVAL => LogicalType::INTERVAL
        }
      }
    }
  }
}

impl convert::From<parquet::FieldRepetitionType> for Repetition {
  fn from(tp: parquet::FieldRepetitionType) -> Self {
    match tp {
      parquet::FieldRepetitionType::REQUIRED => Repetition::REQUIRED,
      parquet::FieldRepetitionType::OPTIONAL => Repetition::OPTIONAL,
      parquet::FieldRepetitionType::REPEATED => Repetition::REPEATED
    }
  }
}

impl convert::From<parquet::Encoding> for Encoding {
  fn from(tp: parquet::Encoding) -> Self {
    match tp {
      parquet::Encoding::PLAIN => Encoding::PLAIN,
      parquet::Encoding::PLAIN_DICTIONARY => Encoding::PLAIN_DICTIONARY,
      parquet::Encoding::RLE => Encoding::RLE,
      parquet::Encoding::BIT_PACKED => Encoding::BIT_PACKED,
      parquet::Encoding::DELTA_BINARY_PACKED => Encoding::DELTA_BINARY_PACKED,
      parquet::Encoding::DELTA_LENGTH_BYTE_ARRAY => Encoding::DELTA_LENGTH_BYTE_ARRAY,
      parquet::Encoding::DELTA_BYTE_ARRAY => Encoding::DELTA_BYTE_ARRAY,
      parquet::Encoding::RLE_DICTIONARY => Encoding::RLE_DICTIONARY
    }
  }
}

impl convert::From<parquet::CompressionCodec> for Compression {
  fn from(tp: parquet::CompressionCodec) -> Self {
    match tp {
      parquet::CompressionCodec::UNCOMPRESSED => Compression::UNCOMPRESSED,
      parquet::CompressionCodec::SNAPPY => Compression::SNAPPY,
      parquet::CompressionCodec::GZIP => Compression::GZIP,
      parquet::CompressionCodec::LZO => Compression::LZO,
      parquet::CompressionCodec::BROTLI => Compression::BROTLI,
      parquet::CompressionCodec::LZ4 => Compression::LZ4,
      parquet::CompressionCodec::ZSTD => Compression::ZSTD
    }
  }
}

impl convert::From<parquet::PageType> for PageType {
  fn from(tp: parquet::PageType) -> Self {
    match tp {
      parquet::PageType::DATA_PAGE => PageType::DATA_PAGE,
      parquet::PageType::INDEX_PAGE => PageType::INDEX_PAGE,
      parquet::PageType::DICTIONARY_PAGE => PageType::DICTIONARY_PAGE,
      parquet::PageType::DATA_PAGE_V2 => PageType::DATA_PAGE_V2
    }
  }
}

impl str::FromStr for Repetition {
  type Err = ParquetError;
  fn from_str(s: &str) -> result::Result<Self, Self::Err> {
    match s {
      "REQUIRED" => Ok(Repetition::REQUIRED),
      "OPTIONAL" => Ok(Repetition::OPTIONAL),
      "REPEATED" => Ok(Repetition::REPEATED),
      other => Err(general_err!("Invalid repetition {}", other)),
    }
  }
}

impl str::FromStr for Type {
  type Err = ParquetError;
  fn from_str(s: &str) -> result::Result<Self, Self::Err> {
    match s {
      "BOOLEAN" => Ok(Type::BOOLEAN),
      "INT32" => Ok(Type::INT32),
      "INT64" => Ok(Type::INT64),
      "INT96" => Ok(Type::INT96),
      "FLOAT" => Ok(Type::FLOAT),
      "DOUBLE" => Ok(Type::DOUBLE),
      "BYTE_ARRAY" | "BINARY" => Ok(Type::BYTE_ARRAY),
      "FIXED_LEN_BYTE_ARRAY" => Ok(Type::FIXED_LEN_BYTE_ARRAY),
      other => Err(general_err!("Invalid type {}", other)),
    }
  }
}

impl str::FromStr for LogicalType {
  type Err = ParquetError;
  fn from_str(s: &str) -> result::Result<Self, Self::Err> {
    match s {
      "NONE" => Ok(LogicalType::NONE),
      "UTF8" => Ok(LogicalType::UTF8),
      "MAP" => Ok(LogicalType::MAP),
      "MAP_KEY_VALUE" => Ok(LogicalType::MAP_KEY_VALUE),
      "LIST" => Ok(LogicalType::LIST),
      "ENUM" => Ok(LogicalType::ENUM),
      "DECIMAL" => Ok(LogicalType::DECIMAL),
      "DATE" => Ok(LogicalType::DATE),
      "TIME_MILLIS" => Ok(LogicalType::TIME_MILLIS),
      "TIME_MICROS" => Ok(LogicalType::TIME_MICROS),
      "TIMESTAMP_MILLIS" => Ok(LogicalType::TIMESTAMP_MILLIS),
      "TIMESTAMP_MICROS" => Ok(LogicalType::TIMESTAMP_MICROS),
      "UINT_8" => Ok(LogicalType::UINT_8),
      "UINT_16" => Ok(LogicalType::UINT_16),
      "UINT_32" => Ok(LogicalType::UINT_32),
      "UINT_64" => Ok(LogicalType::UINT_64),
      "INT_8" => Ok(LogicalType::INT_8),
      "INT_16" => Ok(LogicalType::INT_16),
      "INT_32" => Ok(LogicalType::INT_32),
      "INT_64" => Ok(LogicalType::INT_64),
      "JSON" => Ok(LogicalType::JSON),
      "BSON" => Ok(LogicalType::BSON),
      "INTERVAL" => Ok(LogicalType::INTERVAL),
      other => Err(general_err!("Invalid logical type {}", other)),
    }
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_display_type() {
    assert_eq!(Type::BOOLEAN.to_string(), "BOOLEAN");
    assert_eq!(Type::INT32.to_string(), "INT32");
    assert_eq!(Type::INT64.to_string(), "INT64");
    assert_eq!(Type::INT96.to_string(), "INT96");
    assert_eq!(Type::FLOAT.to_string(), "FLOAT");
    assert_eq!(Type::DOUBLE.to_string(), "DOUBLE");
    assert_eq!(Type::BYTE_ARRAY.to_string(), "BYTE_ARRAY");
    assert_eq!(Type::FIXED_LEN_BYTE_ARRAY.to_string(), "FIXED_LEN_BYTE_ARRAY");
  }

  #[test]
  fn test_from_type() {
    assert_eq!(Type::from(parquet::Type::BOOLEAN), Type::BOOLEAN);
    assert_eq!(Type::from(parquet::Type::INT32), Type::INT32);
    assert_eq!(Type::from(parquet::Type::INT64), Type::INT64);
    assert_eq!(Type::from(parquet::Type::INT96), Type::INT96);
    assert_eq!(Type::from(parquet::Type::FLOAT), Type::FLOAT);
    assert_eq!(Type::from(parquet::Type::DOUBLE), Type::DOUBLE);
    assert_eq!(Type::from(parquet::Type::BYTE_ARRAY), Type::BYTE_ARRAY);
    assert_eq!(
      Type::from(parquet::Type::FIXED_LEN_BYTE_ARRAY),
      Type::FIXED_LEN_BYTE_ARRAY
    );
  }

  #[test]
  fn test_from_string_into_type() {
    assert_eq!(Type::BOOLEAN.to_string().parse::<Type>().unwrap(), Type::BOOLEAN);
    assert_eq!(Type::INT32.to_string().parse::<Type>().unwrap(), Type::INT32);
    assert_eq!(Type::INT64.to_string().parse::<Type>().unwrap(), Type::INT64);
    assert_eq!(Type::INT96.to_string().parse::<Type>().unwrap(), Type::INT96);
    assert_eq!(Type::FLOAT.to_string().parse::<Type>().unwrap(), Type::FLOAT);
    assert_eq!(Type::DOUBLE.to_string().parse::<Type>().unwrap(), Type::DOUBLE);
    assert_eq!(Type::BYTE_ARRAY.to_string().parse::<Type>().unwrap(), Type::BYTE_ARRAY);
    assert_eq!("BINARY".parse::<Type>().unwrap(), Type::BYTE_ARRAY);
    assert_eq!(
      Type::FIXED_LEN_BYTE_ARRAY.to_string().parse::<Type>().unwrap(),
      Type::FIXED_LEN_BYTE_ARRAY
    );
  }

  #[test]
  fn test_display_logical_type() {
    assert_eq!(LogicalType::NONE.to_string(), "NONE");
    assert_eq!(LogicalType::UTF8.to_string(), "UTF8");
    assert_eq!(LogicalType::MAP.to_string(), "MAP");
    assert_eq!(LogicalType::MAP_KEY_VALUE.to_string(), "MAP_KEY_VALUE");
    assert_eq!(LogicalType::LIST.to_string(), "LIST");
    assert_eq!(LogicalType::ENUM.to_string(), "ENUM");
    assert_eq!(LogicalType::DECIMAL.to_string(), "DECIMAL");
    assert_eq!(LogicalType::DATE.to_string(), "DATE");
    assert_eq!(LogicalType::TIME_MILLIS.to_string(), "TIME_MILLIS");
    assert_eq!(LogicalType::DATE.to_string(), "DATE");
    assert_eq!(LogicalType::TIME_MICROS.to_string(), "TIME_MICROS");
    assert_eq!(LogicalType::TIMESTAMP_MILLIS.to_string(), "TIMESTAMP_MILLIS");
    assert_eq!(LogicalType::TIMESTAMP_MICROS.to_string(), "TIMESTAMP_MICROS");
    assert_eq!(LogicalType::UINT_8.to_string(), "UINT_8");
    assert_eq!(LogicalType::UINT_16.to_string(), "UINT_16");
    assert_eq!(LogicalType::UINT_32.to_string(), "UINT_32");
    assert_eq!(LogicalType::UINT_64.to_string(), "UINT_64");
    assert_eq!(LogicalType::INT_8.to_string(), "INT_8");
    assert_eq!(LogicalType::INT_16.to_string(), "INT_16");
    assert_eq!(LogicalType::INT_32.to_string(), "INT_32");
    assert_eq!(LogicalType::INT_64.to_string(), "INT_64");
    assert_eq!(LogicalType::JSON.to_string(), "JSON");
    assert_eq!(LogicalType::BSON.to_string(), "BSON");
    assert_eq!(LogicalType::INTERVAL.to_string(), "INTERVAL");
  }

    #[test]
  fn test_from_logical_type() {
    assert_eq!(
      LogicalType::from(None),
      LogicalType::NONE
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::UTF8)),
      LogicalType::UTF8
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::MAP)),
      LogicalType::MAP
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::MAP_KEY_VALUE)),
      LogicalType::MAP_KEY_VALUE
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::LIST)),
      LogicalType::LIST
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::ENUM)),
      LogicalType::ENUM
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::DECIMAL)),
      LogicalType::DECIMAL
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::DATE)),
      LogicalType::DATE
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::TIME_MILLIS)),
      LogicalType::TIME_MILLIS
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::TIME_MICROS)),
      LogicalType::TIME_MICROS
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::TIMESTAMP_MILLIS)),
      LogicalType::TIMESTAMP_MILLIS
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::TIMESTAMP_MICROS)),
      LogicalType::TIMESTAMP_MICROS
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::UINT_8)),
      LogicalType::UINT_8
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::UINT_16)),
      LogicalType::UINT_16
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::UINT_32)),
      LogicalType::UINT_32
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::UINT_64)),
      LogicalType::UINT_64
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::INT_8)),
      LogicalType::INT_8
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::INT_16)),
      LogicalType::INT_16
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::INT_32)),
      LogicalType::INT_32
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::INT_64)),
      LogicalType::INT_64
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::JSON)),
      LogicalType::JSON
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::BSON)),
      LogicalType::BSON
    );
    assert_eq!(
      LogicalType::from(Some(parquet::ConvertedType::INTERVAL)),
      LogicalType::INTERVAL
    );
  }

  #[test]
  fn test_from_string_into_logical_type() {
    assert_eq!(
      LogicalType::NONE.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::NONE
    );
    assert_eq!(
      LogicalType::UTF8.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::UTF8
    );
    assert_eq!(
      LogicalType::MAP.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::MAP
    );
    assert_eq!(
      LogicalType::MAP_KEY_VALUE.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::MAP_KEY_VALUE
    );
    assert_eq!(
      LogicalType::LIST.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::LIST
    );
    assert_eq!(
      LogicalType::ENUM.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::ENUM
    );
    assert_eq!(
      LogicalType::DECIMAL.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::DECIMAL
    );
    assert_eq!(
      LogicalType::DATE.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::DATE
    );
    assert_eq!(
      LogicalType::TIME_MILLIS.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::TIME_MILLIS
    );
    assert_eq!(
      LogicalType::TIME_MICROS.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::TIME_MICROS
    );
    assert_eq!(
      LogicalType::TIMESTAMP_MILLIS.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::TIMESTAMP_MILLIS
    );
    assert_eq!(
      LogicalType::TIMESTAMP_MICROS.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::TIMESTAMP_MICROS
    );
    assert_eq!(
      LogicalType::UINT_8.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::UINT_8
    );
    assert_eq!(
      LogicalType::UINT_16.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::UINT_16
    );
    assert_eq!(
      LogicalType::UINT_32.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::UINT_32
    );
    assert_eq!(
      LogicalType::UINT_64.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::UINT_64
    );
    assert_eq!(
      LogicalType::INT_8.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::INT_8
    );
    assert_eq!(
      LogicalType::INT_16.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::INT_16
    );
    assert_eq!(
      LogicalType::INT_32.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::INT_32
    );
    assert_eq!(
      LogicalType::INT_64.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::INT_64
    );
    assert_eq!(
      LogicalType::JSON.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::JSON
    );
    assert_eq!(
      LogicalType::BSON.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::BSON
    );
    assert_eq!(
      LogicalType::INTERVAL.to_string().parse::<LogicalType>().unwrap(),
      LogicalType::INTERVAL
    );
  }

  #[test]
  fn test_display_repetition() {
    assert_eq!(Repetition::REQUIRED.to_string(), "REQUIRED");
    assert_eq!(Repetition::OPTIONAL.to_string(), "OPTIONAL");
    assert_eq!(Repetition::REPEATED.to_string(), "REPEATED");
  }

  #[test]
  fn test_from_repetition() {
    assert_eq!(
      Repetition::from(parquet::FieldRepetitionType::REQUIRED),
      Repetition::REQUIRED
    );
    assert_eq!(
      Repetition::from(parquet::FieldRepetitionType::OPTIONAL),
      Repetition::OPTIONAL
    );
    assert_eq!(
      Repetition::from(parquet::FieldRepetitionType::REPEATED),
      Repetition::REPEATED
    );
  }

  #[test]
  fn test_from_string_into_repetition() {
    assert_eq!(
      Repetition::REQUIRED.to_string().parse::<Repetition>().unwrap(),
      Repetition::REQUIRED
    );
    assert_eq!(
      Repetition::OPTIONAL.to_string().parse::<Repetition>().unwrap(),
      Repetition::OPTIONAL
    );
    assert_eq!(
      Repetition::REPEATED.to_string().parse::<Repetition>().unwrap(),
      Repetition::REPEATED
    );
  }

  #[test]
  fn test_display_encoding() {
    assert_eq!(Encoding::PLAIN.to_string(), "PLAIN");
    assert_eq!(Encoding::PLAIN_DICTIONARY.to_string(), "PLAIN_DICTIONARY");
    assert_eq!(Encoding::RLE.to_string(), "RLE");
    assert_eq!(Encoding::BIT_PACKED.to_string(), "BIT_PACKED");
    assert_eq!(Encoding::DELTA_BINARY_PACKED.to_string(), "DELTA_BINARY_PACKED");
    assert_eq!(Encoding::DELTA_LENGTH_BYTE_ARRAY.to_string(), "DELTA_LENGTH_BYTE_ARRAY");
    assert_eq!(Encoding::DELTA_BYTE_ARRAY.to_string(), "DELTA_BYTE_ARRAY");
    assert_eq!(Encoding::RLE_DICTIONARY.to_string(), "RLE_DICTIONARY");
  }

  #[test]
  fn test_from_encoding() {
    assert_eq!(
      Encoding::from(parquet::Encoding::PLAIN),
      Encoding::PLAIN
    );
    assert_eq!(
      Encoding::from(parquet::Encoding::PLAIN_DICTIONARY),
      Encoding::PLAIN_DICTIONARY
    );
    assert_eq!(Encoding::from(parquet::Encoding::RLE), Encoding::RLE);
    assert_eq!(Encoding::from(parquet::Encoding::BIT_PACKED), Encoding::BIT_PACKED);
    assert_eq!(
      Encoding::from(parquet::Encoding::DELTA_BINARY_PACKED),
      Encoding::DELTA_BINARY_PACKED
    );
    assert_eq!(
      Encoding::from(parquet::Encoding::DELTA_LENGTH_BYTE_ARRAY),
      Encoding::DELTA_LENGTH_BYTE_ARRAY
    );
    assert_eq!(
      Encoding::from(parquet::Encoding::DELTA_BYTE_ARRAY),
      Encoding::DELTA_BYTE_ARRAY
    );
  }

  #[test]
  fn test_display_compression() {
    assert_eq!(Compression::UNCOMPRESSED.to_string(), "UNCOMPRESSED");
    assert_eq!(Compression::SNAPPY.to_string(), "SNAPPY");
    assert_eq!(Compression::GZIP.to_string(), "GZIP");
    assert_eq!(Compression::LZO.to_string(), "LZO");
    assert_eq!(Compression::BROTLI.to_string(), "BROTLI");
    assert_eq!(Compression::LZ4.to_string(), "LZ4");
    assert_eq!(Compression::ZSTD.to_string(), "ZSTD");
  }

  #[test]
  fn test_from_compression() {
    assert_eq!(
      Compression::from(parquet::CompressionCodec::UNCOMPRESSED),
      Compression::UNCOMPRESSED
    );
    assert_eq!(
      Compression::from(parquet::CompressionCodec::SNAPPY),
      Compression::SNAPPY
    );
    assert_eq!(
      Compression::from(parquet::CompressionCodec::GZIP),
      Compression::GZIP
    );
    assert_eq!(
      Compression::from(parquet::CompressionCodec::LZO),
      Compression::LZO
    );
    assert_eq!(
      Compression::from(parquet::CompressionCodec::BROTLI),
      Compression::BROTLI
    );
    assert_eq!(
      Compression::from(parquet::CompressionCodec::LZ4),
      Compression::LZ4
    );
    assert_eq!(
      Compression::from(parquet::CompressionCodec::ZSTD),
      Compression::ZSTD
    );
  }

  #[test]
  fn test_display_page_type() {
    assert_eq!(PageType::DATA_PAGE.to_string(), "DATA_PAGE");
    assert_eq!(PageType::INDEX_PAGE.to_string(), "INDEX_PAGE");
    assert_eq!(PageType::DICTIONARY_PAGE.to_string(), "DICTIONARY_PAGE");
    assert_eq!(PageType::DATA_PAGE_V2.to_string(), "DATA_PAGE_V2");
  }

  #[test]
  fn test_from_page_type() {
    assert_eq!(PageType::from(parquet::PageType::DATA_PAGE), PageType::DATA_PAGE);
    assert_eq!(PageType::from(parquet::PageType::INDEX_PAGE), PageType::INDEX_PAGE);
    assert_eq!(
      PageType::from(parquet::PageType::DICTIONARY_PAGE),
      PageType::DICTIONARY_PAGE
    );
    assert_eq!(PageType::from(parquet::PageType::DATA_PAGE_V2), PageType::DATA_PAGE_V2);
  }
}
