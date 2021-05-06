/* Copyright (C) 2018 Olivier Goffart <ogoffart@woboq.com>

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial
portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES
OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

// That's the same as parent::write_u32, but it should always be little endian
fn write_u32(val: u32) -> [u8; 4] {
    [
        (val & 0xff) as u8,
        ((val >> 8) & 0xff) as u8,
        ((val >> 16) & 0xff) as u8,
        ((val >> 24) & 0xff) as u8,
    ]
}

// That's the same as parent::write_u32, but it should always be little endian
fn write_u16(val: u16) -> [u8; 2] {
    [(val & 0xff) as u8, ((val >> 8) & 0xff) as u8]
}

pub enum Value {
    String(String),
    Double(f64),
    Bool(bool),
    // TODO: other things
}

fn write_string(result: &mut Vec<u8>, str: &str) {
    result.extend_from_slice(&write_u16(str.len() as u16));
    result.extend_from_slice(str.as_bytes());
    let pad = (str.len() + 2) % 4;
    if pad != 0 {
        for _ in 0..(4 - pad) {
            result.push(0); //Padding
        }
    }
}

fn string_size(str: &str) -> u32 {
    ((2 + str.len() + 3) & !3) as u32
}

fn write_value(result: &mut Vec<u8>, v: &Value) {
    match v {
        Value::String(ref s) => {
            write_string(result, &s);
        }
        Value::Double(d) => {
            let bits = unsafe { std::mem::transmute::<f64, u64>(*d) };
            result.extend_from_slice(&write_u32((bits & 0xffff_ffff) as u32));
            result.extend_from_slice(&write_u32((bits >> 32) as u32));
        }
        Value::Bool(_) => {} // value encoded in header
    }
}

fn compute_header(v: &Value, off: u32) -> u32 {
    match v {
        Value::String(_) => (3 | (1 << 3)) | (off << 5), // FIXME: Assume ascii (as does all the code)
        Value::Double(_) => 2 | (off << 5),
        Value::Bool(v) => 1 | ((*v as u32) << 5),
    }
}

fn compute_size(v: &Value) -> u32 {
    match v {
        Value::String(ref s) => string_size(&s),
        Value::Double(_) => 8,
        Value::Bool(_) => 0,
    }
}

pub fn serialize(obj: &[(&'static str, Value)]) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let mut size = 12;
    for e in obj {
        size += string_size(e.0) + compute_size(&e.1) + 8;
    }

    result.extend_from_slice(&write_u32(size as u32));
    result.extend_from_slice(&write_u32(1 | (obj.len() as u32) << 1));
    result.extend_from_slice(&write_u32(size - (obj.len() as u32) * 4));

    let mut table: Vec<u32> = Vec::new();
    let mut off = 12;
    for e in obj {
        table.push(off);
        off += 4 + string_size(e.0);
        let mut h = compute_header(&e.1, off);
        h |= 1 << 4;
        result.extend_from_slice(&write_u32(h));
        write_string(&mut result, e.0);
        write_value(&mut result, &e.1);
        off += compute_size(&e.1);
    }
    for x in table {
        result.extend_from_slice(&write_u32(x));
    }
    result
}
