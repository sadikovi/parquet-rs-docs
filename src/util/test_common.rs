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

use rand::{thread_rng, Rng, Rand};
use rand::distributions::range::SampleRange;
use std::env;
use std::fs;
use std::io::Write;

use data_type::{ByteArray, DataType, FixedLenByteArrayType};

pub trait RandGen<T: DataType> {
  fn gen(len: i32) -> T::T;

  fn gen_vec(len: i32, total: usize) -> Vec<T::T> {
    let mut result = vec![];
    for _ in 0..total {
      result.push(Self::gen(len))
    }
    result
  }
}

impl<T: DataType> RandGen<T> for T {
  default fn gen(_: i32) -> T::T {
    let mut rng = thread_rng();
    rng.gen::<T::T>()
  }
}

impl RandGen<FixedLenByteArrayType> for FixedLenByteArrayType {
  fn gen(len: i32) -> ByteArray {
    let mut rng = thread_rng();
    let value_len =
      if len < 0 {
        rng.gen_range::<usize>(0, 128)
      } else {
        len as usize
      };
    let value = random_bytes(value_len);
    ByteArray::from(value)
  }
}

pub fn random_bytes(n: usize) -> Vec<u8> {
  let mut result = vec![];
  let mut rng = thread_rng();
  for _ in 0..n {
    result.push(rng.gen_range(0, 255) & 0xFF);
  }
  result
}

pub fn random_bools(n: usize) -> Vec<bool> {
  let mut result = vec![];
  let mut rng = thread_rng();
  for _ in 0..n {
    result.push(rng.gen::<bool>());
  }
  result
}

pub fn random_numbers<T: Rand>(n: usize) -> Vec<T> {
  let mut result = vec![];
  let mut rng = thread_rng();
  for _ in 0..n {
    result.push(rng.gen::<T>());
  }
  result
}

pub fn random_numbers_range<T>(
  n: usize,
  low: T,
  high: T,
  result: &mut Vec<T>
) where T: PartialOrd + SampleRange + Copy {
  let mut rng = thread_rng();
  for _ in 0..n {
    result.push(rng.gen_range(low, high));
  }
}

/// Returns file handle for a test parquet file from 'data' directory
pub fn get_test_file(file_name: &str) -> fs::File {
  let mut path_buf = env::current_dir().unwrap();
  path_buf.push("data");
  path_buf.push(file_name);
  let file = fs::File::open(path_buf.as_path());
  assert!(file.is_ok());
  file.unwrap()
}

/// Returns file handle for a temp file in 'target' directory with a provided content
pub fn get_temp_file(file_name: &str, content: &[u8]) -> fs::File {
  // build tmp path to a file in "target/debug/testdata"
  let mut path_buf = env::current_dir().unwrap();
  path_buf.push("target");
  path_buf.push("debug");
  path_buf.push("testdata");
  fs::create_dir_all(&path_buf).unwrap();
  path_buf.push(file_name);

  // write file content
  let mut tmp_file = fs::File::create(path_buf.as_path()).unwrap();
  tmp_file.write_all(content).unwrap();
  tmp_file.sync_all().unwrap();

  // read file and return file handle
  let file = fs::File::open(path_buf.as_path());
  assert!(file.is_ok());
  file.unwrap()
}
