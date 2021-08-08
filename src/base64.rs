
//==============================================================================

use std::sync::Once;

//********

pub const LITTLE_ENDIAN: u8 = 0;
pub const BIG_ENDIAN: u8 = 1;

const PAD: char = '=';

//********

static INIT: Once = Once::new();

static mut LUT_ENC: Vec<char> = Vec::new();
static mut LUT_DEC: Vec<u8> = Vec::new();

//********

//==============================================================================

fn init()
{
	// Initiaze lookup tables for base64 encoding/decoding

	//println!("starting base64::init()");

	unsafe
	{
		INIT.call_once(||
		{
			//println!("starting base64::INIT.call_once()");

			LUT_ENC = vec![' '; 64];
			LUT_DEC = vec![0; 256];

			// Everything but padding.  Order of string characters implies
			// decoded value
			const BASE64_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklm\
					nopqrstuvwxyz0123456789+/";

			// Splitting the String into a Vec of chars is the only way to use
			// them as an index of another Vec.  UTF-8 strings (default in rust)
			// may not be 1-byte per character, so they cannot be indexed by
			// bytes and need to be split ahead of time
			let chars: Vec<char> = BASE64_STR.chars().collect();

			let mut i: u8 = 0;
			for c in chars
			{
				LUT_ENC[i as usize] = c;
				LUT_DEC[c as usize] = i;
				i += 1;
			}

			// Padding.  Same value as "A" so it can't be included in loop above
			LUT_DEC[PAD as usize] =  0;

			//println!("LUT_ENC = {:?}", LUT_ENC);
			//println!("LUT_DEC = {:?}", LUT_DEC);

		});
	}
}

//==============================================================================

pub fn encode_u8_raw(v: &Vec<u8>) -> String
{
	//println!("starting base64::encode_u8_raw()");
	//println!("v = {:?}", v);

	// Divide and round up
	let len = (v.len() + 2) / 3;

	let mut s = String::with_capacity(4 * len);

	//println!("4 * len = {}", 4 * len);

	init();

	for i in 0 .. len
	{ unsafe {
		//println!("i = {}", i);

		let i0 = 3*i + 0;
		let i1 = 3*i + 1;
		let i2 = 3*i + 2;

		// Pad with zeros for partial chars
		let v0 = if i0 < v.len() { v[i0] } else { 0 };  // never 0
		let v1 = if i1 < v.len() { v[i1] } else { 0 };
		let v2 = if i2 < v.len() { v[i2] } else { 0 };

		// First two chars are trivial
		s.push(LUT_ENC[  (v0               >> 2)  as usize]);
		s.push(LUT_ENC[(((v0 & 0b00000011) << 4)
		               | (v1               >> 4)) as usize]);

		// Add anywhere from 0 to 2 padding chars

		if i1 < v.len() {

		s.push(LUT_ENC[(((v1 & 0b00001111) << 2)
		               | (v2               >> 6)) as usize]);

		} else { s.push(PAD) };

		if i2 < v.len() {

		s.push(LUT_ENC[  (v2 & 0b00111111)        as usize]);

		} else { s.push(PAD) };
	}}

	//println!("s.len() = {}", s.len());

	return s;
}

//********

pub fn decode_u8_raw(s: &str) -> Vec<u8>
{
	// Decode a raw u8 Vec from a base64 string.  This is general with no VTK
	// dependence, other than the base64 character set and padding character
	// (which are fairly standard between VTK and other base64 encodings)

	//println!("starting base64::decode_u8_raw()");
	//println!("s = {}", s);
	//println!("s.len = {}", s.len());

	// 6 bits per base64 character, 8 bits per u8
	let vlen = s.len() * 6 / 8;

	//println!("vlen (bytes) = {}", vlen);
	//println!();

	// Size vlen and initialized to 0
	let mut v: Vec<u8> = vec![0; vlen];

	let chars: Vec<char> = s.chars().collect();

	init();

	// Decode 3 bytes at a time
	for i in 0 .. vlen/3
	{ unsafe {
		//println!("{}", &s[4*i .. 4*i+4]);

		// Step bytes by 3 and base64 chars by 4
		v[3*i+0] = ((LUT_DEC[chars[4*i+0] as usize] & 0b111111) << 2)
		         | ((LUT_DEC[chars[4*i+1] as usize] & 0b110000) >> 4);
		v[3*i+1] = ((LUT_DEC[chars[4*i+1] as usize] & 0b001111) << 4)
		         | ((LUT_DEC[chars[4*i+2] as usize] & 0b111100) >> 2);
		v[3*i+2] = ((LUT_DEC[chars[4*i+2] as usize] & 0b000011) << 6)
		         |  (LUT_DEC[chars[4*i+3] as usize] & 0b111111);
	}}
	//println!("v = {:?}", v);

	return v;
}

//==============================================================================

fn encode_u64_len(endianness: u8, blen: u64) -> Vec<u8>
{
	// Allocate bytes Vec and encode the rest of its length into its beginning

	let mut bytes: Vec<u8> = vec![0; (blen+8) as usize];

	let b = if endianness == BIG_ENDIAN
	{
		blen.to_be_bytes()
	}
	else
	{
		blen.to_le_bytes()
	};

	for i in 0 .. b.len()
	{
		bytes[i] = b[i];
	}

	return bytes;
}

//********

fn decode_u64_len(bytes: &Vec<u8>, endianness: u8) -> u64
{
	// By VTK convention, get the length from the beginning of a byte vec

	let mut b: [u8; 8] = [0; 8];
	for i in 0 .. 8
	{
		b[i] = bytes[i];
	}

	return if endianness == BIG_ENDIAN
	{
		u64::from_be_bytes(b)
	}
	else
	{
		u64::from_le_bytes(b)
	};
}

//==============================================================================

pub fn encode_f32(v: &Vec<f32>, endianness: u8) -> String
{
	// Encode a VTK-encoded base64 string from an f32 Vec

	//println!("starting base64::encode_f32()");

	let blen = v.len() * 4;
	//println!("vlen (f32) = {}", vlen);

	let mut bytes = encode_u64_len(endianness, blen as u64);

	for i in 2 .. v.len()+2
	{
		let b = if endianness == BIG_ENDIAN
		{
			v[i-2].to_be_bytes()
		}
		else
		{
			v[i-2].to_le_bytes()
		};

		for j in 0 .. b.len()
		{
			bytes[4*i + j] = b[j];
		}
	}

	return encode_u8_raw(&bytes);
}

//********

pub fn decode_f32(string: &str, endianness: u8) -> Vec<f32>
{
	// Decode a f32 vec from a VTK-encoded base64 string

	//println!("starting base64::decode_f32()");

	let bytes = decode_u8_raw(string);

	let vlen = (decode_u64_len(&bytes, endianness) as usize) / 4;
	//println!("vlen (f32) = {}", vlen);

	let mut v: Vec<f32> = vec![0.0; vlen];
	let mut b: [u8; 4] = [0; 4];

	for i in 2 .. vlen+2
	{
		for j in 0 .. 4
		{
			b[j] = bytes[4*i + j];
		}

		v[i-2] = if endianness == BIG_ENDIAN
		{
			f32::from_be_bytes(b)
		}
		else
		{
			f32::from_le_bytes(b)
		}
	}

	return v;
}

//==============================================================================

// Cannot have the same name as above fn, even though they take/return different
// types

pub fn encode_i64(v: &Vec<i64>, endianness: u8) -> String
{
	// Encode a VTK-encoded base64 string from an i64 Vec

	//println!("starting base64::encode_i64()");

	let blen = v.len() * 8;
	//println!("vlen (i64) = {}", vlen);

	let mut bytes = encode_u64_len(endianness, blen as u64);

	for i in 1 .. v.len()+1
	{
		let b = if endianness == BIG_ENDIAN
		{
			v[i-1].to_be_bytes()
		}
		else
		{
			v[i-1].to_le_bytes()
		};

		for j in 0 .. b.len()
		{
			bytes[8*i + j] = b[j];
		}
	}

	return encode_u8_raw(&bytes);
}

//********

pub fn decode_i64(string: &str, endianness: u8) -> Vec<i64>
{
	// Decode an i64 vec from a VTK-encoded base64 string

	//println!("starting base64::decode_i64()");

	let bytes = decode_u8_raw(string);

	// Get length from first 8 bytes, because the remaining may be a different
	// type (e.g.  for Uint8 types in ParaView).  Return only the rest of the
	// array.

	let vlen = (decode_u64_len(&bytes, endianness) as usize) / 8;
	//println!("vlen (i64) = {}", vlen);

	let mut v: Vec<i64> = vec![0; vlen];
	let mut b: [u8; 8] = [0; 8];

	for i in 1 .. vlen+1
	{
		//println!("i = {}", i);
		for j in 0 .. 8
		{
			b[j] = bytes[8*i + j];
		}

		v[i-1] = if endianness == BIG_ENDIAN
		{
			i64::from_be_bytes(b)
		}
		else
		{
			i64::from_le_bytes(b)
		};
	}

	return v;
}

//==============================================================================

pub fn encode_u8(v: &Vec<u8>, endianness: u8) -> String
{
	// Encode a VTK-encoded base64 string from a u8 Vec

	//println!("starting base64::encode_u8()");

	let blen = v.len();

	let mut bytes = encode_u64_len(endianness, blen as u64);

	for i in 8 .. v.len()+8
	{
		let b = if endianness == BIG_ENDIAN
		{
			v[i-8].to_be_bytes()
		}
		else
		{
			v[i-8].to_le_bytes()
		};

		bytes[i] = b[0];
	}

	return encode_u8_raw(&bytes);
}

//********

pub fn decode_u8(string: &str, endianness: u8) -> Vec<u8>
{
	// Decode a u8 vec from a VTK-encoded base64 string

	// Still need endianness for the u64 len in the first bytes

	//println!("starting base64::decode_u8()");

	let bytes = decode_u8_raw(string);

	let vlen = decode_u64_len(&bytes, endianness) as usize;
	//println!("vlen (u8) = {}", vlen);

	let mut v: Vec<u8> = vec![0; vlen];

	for i in 8 .. vlen+8
	{
		// Endianness doesn't matter for a single byte
		v[i-8] = bytes[i];
	}

	return v;
}

//==============================================================================

#[cfg(test)]
mod tests
{
	// Import names from outer scope
	use super::*;

	// Test strings from "./data/icosahedron-binary.vtu"

	// Points (Float32)
	const STR_F32: &str = "kAAAAAAAAABpg40+QMRZPy755D5MPjm/UJYGPy755D5MPjm/UJ\
			YGvy755D5pg40+QMRZvy755D4u+WQ/f5J8pS755D5pg42+QMRZPy755L4u+WS/d5L\
			8JC755L5pg42+QMRZvy755L5MPjk/UJYGvy755L5MPjk/UJYGPy755L4AAAAAAAAA\
			AAAAgD8zMQ0lAAAAAAAAgL8=";

	const STR_F32_RAW: &str =
	                      "kAAAAAAAAABpg40+QMRZPy755D5MPjm/UJYGPy755D5MPjm/UJ\
			YGvy755D5pg40+QMRZvy755D4u+WQ/f5J8pS755D5pg42+QMRZPy755L4u+WS/d5L\
			8JC755L5pg42+QMRZvy755L5MPjk/UJYGvy755L5MPjk/UJYGPy755L4AAAAAAAAA\
			AAAAgD8zMQ0lAAAAAAAAgL8A";

	// connectivity (Int64)
	const STR_I64: &str = "4AEAAAAAAAAAAAAAAAAAAAEAAAAAAAAACgAAAAAAAAABAAAAAA\
			AAAAIAAAAAAAAACgAAAAAAAAACAAAAAAAAAAMAAAAAAAAACgAAAAAAAAADAAAAAAA\
			AAAQAAAAAAAAACgAAAAAAAAAEAAAAAAAAAAAAAAAAAAAACgAAAAAAAAABAAAAAAAA\
			AAAAAAAAAAAABQAAAAAAAAACAAAAAAAAAAEAAAAAAAAABgAAAAAAAAADAAAAAAAAA\
			AIAAAAAAAAABwAAAAAAAAAEAAAAAAAAAAMAAAAAAAAACAAAAAAAAAAAAAAAAAAAAA\
			QAAAAAAAAACQAAAAAAAAAFAAAAAAAAAAYAAAAAAAAAAQAAAAAAAAAGAAAAAAAAAAc\
			AAAAAAAAAAgAAAAAAAAAHAAAAAAAAAAgAAAAAAAAAAwAAAAAAAAAIAAAAAAAAAAkA\
			AAAAAAAABAAAAAAAAAAJAAAAAAAAAAUAAAAAAAAAAAAAAAAAAAAGAAAAAAAAAAUAA\
			AAAAAAACwAAAAAAAAAHAAAAAAAAAAYAAAAAAAAACwAAAAAAAAAIAAAAAAAAAAcAAA\
			AAAAAACwAAAAAAAAAJAAAAAAAAAAgAAAAAAAAACwAAAAAAAAAFAAAAAAAAAAkAAAA\
			AAAAACwAAAAAAAAA=";

	// types (UInt8)
	const STR_U8: &str = "FAAAAAAAAAAFBQUFBQUFBQUFBQUFBQUFBQUFBQ==";

	// offsets (Int64) not tested here, same type as connectivity

	// Raw bytes for STR_F32
	const EXPECTED_F32_BYTES: [u8; 153] = [144, 0, 0, 0, 0, 0, 0, 0, 105, 131,
			141, 62, 64, 196, 89, 63, 46, 249, 228, 62, 76, 62, 57, 191, 80,
			150, 6, 63, 46, 249, 228, 62, 76, 62, 57, 191, 80, 150, 6, 191, 46,
			249, 228, 62, 105, 131, 141, 62, 64, 196, 89, 191, 46, 249, 228, 62,
			46, 249, 100, 63, 127, 146, 124, 165, 46, 249, 228, 62, 105, 131,
			141, 190, 64, 196, 89, 63, 46, 249, 228, 190, 46, 249, 100, 191,
			119, 146, 252, 36, 46, 249, 228, 190, 105, 131, 141, 190, 64, 196,
			89, 191, 46, 249, 228, 190, 76, 62, 57, 63, 80, 150, 6, 191, 46,
			249, 228, 190, 76, 62, 57, 63, 80, 150, 6, 63, 46, 249, 228, 190, 0,
			0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63, 51, 49, 13, 37, 0, 0, 0, 0, 0,
			0, 128, 191, 0];

	// STR_F32, decoded as actual f32, and without the u64 from the beginning of
	// the string
	const EXPECTED_F32: [f32; 36] = [0.2763932, 0.8506508, 0.4472136,
			-0.7236068, 0.5257311, 0.4472136, -0.7236068, -0.5257311, 0.4472136,
			0.2763932, -0.8506508, 0.4472136, 0.8944272, -2.190715e-16,
			0.4472136, -0.2763932, 0.8506508, -0.4472136, -0.8944272,
			1.095357e-16, -0.4472136, -0.2763932, -0.8506508, -0.4472136,
			0.7236068, -0.5257311, -0.4472136, 0.7236068, 0.5257311, -0.4472136,
			0.0, 0.0, 1.0, 1.224647e-16, 0.0, -1.0];

	const EXPECTED_I64: [i64; 60] = [0, 1, 10, 1, 2, 10, 2, 3, 10, 3, 4, 10, 4,
			0, 10, 1, 0, 5, 2, 1, 6, 3, 2, 7, 4, 3, 8, 0, 4, 9, 5, 6, 1, 6, 7,
			2, 7, 8, 3, 8, 9, 4, 9, 5, 0, 6, 5, 11, 7, 6, 11, 8, 7, 11, 9, 8,
			11, 5, 9, 11];

	const EXPECTED_U8: [u8; 20] = [5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
			5, 5, 5, 5, 5];

	//********

	#[test]
	fn test_encode_u8_raw()
	{
		let s = encode_u8_raw(&EXPECTED_F32_BYTES.to_vec());
		//println!("s = {}", s);
		assert_eq!(s, STR_F32_RAW);
	}

	#[test]
	fn test_decode_u8_raw()
	{
		let data = decode_u8_raw(STR_F32);

		//// Run with "cargo test -- --nocapture" to print during tests
		//println!("data = {:?}", data);

		assert_eq!(data, EXPECTED_F32_BYTES);
	}

	//********

	#[test]
	fn test_encode_f32()
	{
		let s = encode_f32(&EXPECTED_F32.to_vec(), LITTLE_ENDIAN);
		//println!("s = {}", s);
		assert_eq!(s, STR_F32);
	}

	#[test]
	fn test_decode_f32()
	{
		let data = decode_f32(STR_F32, LITTLE_ENDIAN);
		assert_eq!(data, EXPECTED_F32);
	}

	//********

	#[test]
	fn test_decode_i64()
	{
		let data = decode_i64(STR_I64, LITTLE_ENDIAN);
		assert_eq!(data, EXPECTED_I64);
	}

	//********

	#[test]
	fn test_decode_u8()
	{
		let data = decode_u8(STR_U8, LITTLE_ENDIAN);
		assert_eq!(data, EXPECTED_U8);
	}

	//********

	#[test]
	fn test_le_f32()
	{
		let endianness = LITTLE_ENDIAN;
		let s = encode_f32(&EXPECTED_F32.to_vec(), endianness);
		//println!("s = {}", s);
		let data = decode_f32(&s, endianness);
		assert_eq!(data, EXPECTED_F32);
	}

	#[test]
	fn test_be_f32()
	{
		let endianness = BIG_ENDIAN;
		let s = encode_f32(&EXPECTED_F32.to_vec(), endianness);
		//println!("s = {}", s);
		let data = decode_f32(&s, endianness);
		assert_eq!(data, EXPECTED_F32);
	}

	#[test]
	fn test_le_i64()
	{
		let endianness = LITTLE_ENDIAN;
		let s = encode_i64(&EXPECTED_I64.to_vec(), endianness);
		//println!("s = {}", s);
		let data = decode_i64(&s, endianness);
		assert_eq!(data, EXPECTED_I64);
	}

	#[test]
	fn test_be_i64()
	{
		let endianness = BIG_ENDIAN;
		let s = encode_i64(&EXPECTED_I64.to_vec(), endianness);
		//println!("s = {}", s);
		let data = decode_i64(&s, endianness);
		assert_eq!(data, EXPECTED_I64);
	}

	#[test]
	fn test_le_u8()
	{
		let endianness = LITTLE_ENDIAN;
		let s = encode_u8(&EXPECTED_U8.to_vec(), endianness);
		//println!("s = {}", s);
		let data = decode_u8(&s, endianness);
		assert_eq!(data, EXPECTED_U8);
	}

	#[test]
	fn test_be_u8()
	{
		let endianness = BIG_ENDIAN;
		let s = encode_u8(&EXPECTED_U8.to_vec(), endianness);
		//println!("s = {}", s);
		let data = decode_u8(&s, endianness);
		assert_eq!(data, EXPECTED_U8);
	}
}

//==============================================================================

