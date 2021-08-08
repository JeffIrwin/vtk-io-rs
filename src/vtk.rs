
//==============================================================================

// Standard
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Write;
use std::str;

//********

// Third party

use ansi_term::Colour;

use quick_xml::Reader;
use quick_xml::events::Event;

//********

// This lib
use crate::base64;

//********

// VTK identifiers
const ASCII   : &str = "ascii";
const BINARY  : &str = "binary";
const VTK_F32 : &str = "Float32";
const VTK_I64 : &str = "Int64";
const VTK_U8  : &str = "UInt8";
//const VTK_U64 : &str = "UInt64";  // not used (yet)
const VTK_FILE: &str = "VTKFile";
const UGRID   : &str = "UnstructuredGrid";
const PIECE   : &str = "Piece";
const DATA    : &str = "DataArray";
const POINTS  : &str = "Points";
const CELLS   : &str = "Cells";
const PDATA   : &str = "PointData";
const CDATA   : &str = "CellData";
const CONN    : &str = "connectivity";
const OFFSETS : &str = "offsets";
const TYPES   : &str = "types";
const TYPE    : &str = "type";
const VERSION : &str = "version";
const BYTEORD : &str = "byte_order";
const VTK_LIT : &str = "LittleEndian";
const VTK_BIG : &str = "BigEndian";
const NPOINTS : &str = "NumberOfPoints";
const NCELLS  : &str = "NumberOfCells";
const NCOMP   : &str = "NumberOfComponents";
const NAME    : &str = "Name";
const FORMAT  : &str = "format";

//==============================================================================

pub struct Settings
{
	pub input: String,
	pub output: String,
	pub le: bool,
	pub be: bool,
	pub ascii: bool,
	pub binary: bool,
}

//impl Settings
//{
//	pub fn new() -> Settings
//	{
//		Settings
//		{
//			input: "".to_string(),
//			output: "".to_string(),
//			le: false,
//			be: false,
//		}
//	}
//}

#[derive(Debug)]
pub struct VtkFile
{
	// File eader info
	pub vtype     : String,
	pub version   : String,
	pub endianness: u8,

	pub format    : String,

	// The real data

	pub npoints    : u64,
	pub ncells     : u64,
	pub ncomponents: u64,

	pub points      : Vec<f32>,
	pub connectivity: Vec<i64>,
	pub offsets     : Vec<i64>,
	pub types       : Vec<u8>,

}

impl VtkFile
{
	// Constructor
	pub fn new() -> VtkFile
	{
		VtkFile
		{
			vtype: "".to_string(),
			version: "".to_string(),
			endianness: base64::LITTLE_ENDIAN,

			format: BINARY.to_string(),

			npoints: 0,
			ncells: 0,
			ncomponents: 3,

			points      : Vec::new(),
			connectivity: Vec::new(),
			offsets     : Vec::new(),
			types       : Vec::new(),

		}
	}
}

//==============================================================================

struct DataHeader
{
	// Attributes of DataArray element.  Could add RangeMin, RangeMax, etc.
	pub dtype : String,
	pub name  : String,
	pub format: String,
}

impl DataHeader
{
	fn new() -> DataHeader
	{
		DataHeader
		{
			dtype  : "".to_string(),
			name   : "".to_string(),
			format : BINARY.to_string(),
		}
	}
}

//==============================================================================
//==============================================================================

struct SliceDisplay<'a, T: 'a>(&'a [T]);

// Format a slice for printing without wrapping brackets
impl<'a, T: fmt::Display + 'a> fmt::Display for SliceDisplay<'a, T>
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		// TODO: line break every n elements

		// End-delimited spaces, no commas
		for item in self.0
		{
			write!(f, "{} ", item)?;
		}

		//// Comma-separated, no trailing comma
		//let mut first = true;
		//for item in self.0
		//{
		//	if !first
		//	{
		//		write!(f, ", {}", item)?;
		//	}
		//	else
		//	{
		//		write!(f, "{}", item)?;
		//	}
		//	first = false;
		//}

		Ok(())
	}
}

//==============================================================================

pub fn load(file: &str) -> VtkFile
{
	println!("Loading VTK file \"{}\"\n", file);

	let mut v = VtkFile::new();

	let mut data_array = false;
	let mut dh = DataHeader::new();
	let mut ename = "".to_string();

	let errstr = format!("Cannot load VTK file \"{}\"", file);

	let mut reader = Reader::from_file(file).expect(&errstr);
	reader.trim_text(true);

	let mut buf = Vec::new();

	loop { match reader.read_event(&mut buf)
	{
		Ok(Event::Start(ref e)) =>
		{
			data_array = false;
			ename = String::from_utf8(e.name().to_vec()).expect(&errstr);

			// "match" in rust is like switch/case
			match ename.as_str()
			{
			// Header
			VTK_FILE =>
			{
				for a in e.attributes()
				{
					let attr = a.expect(&errstr);
					let key = reader.decode(attr.key).expect(&errstr);
					let val = attr.unescape_and_decode_value(&reader).expect(&errstr);

					match key
					{
					TYPE =>
					{
						v.vtype = val;
						if v.vtype != UGRID
						{
							unimplemented!("{} type {} is not \
								implemented.  Only {} is implemented",
								VTK_FILE, v.vtype, UGRID);
						}
					},

					VERSION =>
					{
						v.version = val;

						// TODO:  unimplemented on unsupported old versions,
						// warn about new versions.  Need to parse version into
						// multiple ints, compare lexicographically

					},

					BYTEORD =>
					{
						// Default to little endian if unrecognized

						v.endianness = if val == VTK_BIG {
							base64::BIG_ENDIAN
						} else {//if val == VTK_LIT {
							base64::LITTLE_ENDIAN
						};//else {
						//	panic!("Unexpected {}: {}.  Expected {} or {}",
						//		   BYTEORD, val, VTK_LIT, VTK_BIG);
						//};
					},

					// header_type not parsed

					_ => (),
					}  // match key

				}  // attributes loop
			},  // VTKFile (header)

			PIECE =>
			{
				for a in e.attributes()
				{
					let attr = a.expect(&errstr);
					let key = reader.decode(attr.key).expect(&errstr);
					let val = attr.unescape_and_decode_value(&reader).expect(&errstr);

					match key
					{
						NPOINTS => v.npoints = val.parse().expect(&errstr),
						NCELLS  => v.ncells  = val.parse().expect(&errstr),
						_ => (),
					}
				}
			},  // Piece

			// Outer tags are ignored, "name" attribute is used later in
			// Text event
			UGRID  => (),
			POINTS => (),
			CELLS  => (),

			// TODO
			PDATA => (),
			CDATA => (),

			DATA =>
			{
				//println!("DataArray");
				data_array = true;

				for a in e.attributes()
				{
					let attr = a.expect(&errstr);
					let key = reader.decode(attr.key).expect(&errstr);
					let val = attr.unescape_and_decode_value(&reader).expect(&errstr);

					match key
					{
						TYPE   => dh.dtype = val,
						NAME   => dh.name = val,
						NCOMP  => v.ncomponents = val.parse().expect(&errstr),
						FORMAT => dh.format = val,
						_ => (),
					}
				}

			},  // DataArray

			// Default case
			_ =>
			{
				// Bold does not work on Windows
				println!("{}: unknown tag \"{}\" at position {}",
						 Colour::Yellow.bold().paint("warning"),
						 ename, reader.buffer_position());
				//println!("attributes values: {:?}",
				//	e.attributes().collect::<Vec<_>>());
				println!();
			},

			}  // Start tag match
		},  // Start tag

		// Text holds the string contents in between a start <tag> and end </tag>
		Ok(Event::Text(e)) =>
		{
			if data_array
			{
				let string = e.unescape_and_decode(&reader).expect(&errstr);
				//println!("string = {}", string);

				// just use the name attribute (e.g. Name="Points") and ignore
				// the outer tag (e.g.  <Points>)
				match dh.name.as_str()
				{
					POINTS  => v.points       = parse_data_f32(&dh, &string, &v),
					CONN    => v.connectivity = parse_data_i64(&dh, &string, &v),
					OFFSETS => v.offsets      = parse_data_i64(&dh, &string, &v),
					TYPES   => v.types        = parse_data_u8 (&dh, &string, &v),

					_ =>
					{
						println!("{}: unknown {} name \"{}\" at position {}\n",
								 Colour::Yellow.bold().paint("warning"), DATA,
								 dh.name, reader.buffer_position());
					},
				}
			}
			else
			{
				println!("{}: not parsing text in tag \"{}\" at position {}\n",
						 Colour::Yellow.bold().paint("warning"),
						 ename, reader.buffer_position());
			}

		},  // Text event

		Ok(Event::Eof) => break, // exits the loop when reaching end of file
		Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
		_ => (), // There are several other `Event`s we do not consider here

	}}  // reader match and loop
	buf.clear();

	// Verify DataArray sizes are consistent w/ npoints, ncells,
	// ncomponents, etc.

	if v.points.len() < (v.ncomponents * v.npoints) as usize
	{
		panic!("Points are only of len {} < {}", v.points.len(),
			v.ncomponents * v.npoints);
	}

	// No check on connectivity size.  That would require decoding every
	// possible cell type and summing

	if v.offsets.len() < v.ncells as usize
	{
		panic!("Offsets are only of len {} < {}", v.offsets.len(), v.ncells);
	}

	if v.types.len() < v.ncells as usize
	{
		panic!("Offsets are only of len {} < {}", v.types.len(), v.ncells);
	}

	// Could also warn if sizes are greater than expected, but that's less
	// severe

	// TODO:  check PointData and CellData too

	return v;
}

//==============================================================================

impl VtkFile
{
pub fn export(&self, file: &str)
{
	println!("Exporting VTK file \"{}\"\n", file);

	//// Just print a whole struct?  WTF rust!
	//println!("self = {:?}", self);
	//println!();

	// Basically a try/catch block
	let try_export = || -> Result<(), io::Error>
	{
		let mut f = File::create(file)?;

		let byte_order = if self.endianness == base64::BIG_ENDIAN {
			VTK_BIG
		} else {
			VTK_LIT
		};

		writeln!(f, "<{} {}=\"{}\" {}=\"{}\" {}=\"{}\" \
			header_type=\"UInt64\">", VTK_FILE, TYPE, self.vtype, VERSION,
			self.version, BYTEORD, byte_order)?;

		writeln!(f, "	<{}>", self.vtype)?;
		writeln!(f, "		<{} {}=\"{}\" {}=\"{}\">", PIECE, NPOINTS, self.npoints,
			NCELLS, self.ncells)?;

		//********

		// TODO
		writeln!(f, "			<{}>", PDATA)?;
		writeln!(f, "			</{}>", PDATA)?;
		//********
		writeln!(f, "			<{}>", CDATA)?;
		writeln!(f, "			</{}>", CDATA)?;

		// No RangeMin/RangeMax

		//********

		writeln!(f, "			<{}>", POINTS)?;

		writeln!(f, "				<{} {}=\"{}\" {}=\"{}\" {}=\"{}\" {}=\"{}\">",
			DATA, TYPE, VTK_F32, NAME, POINTS, NCOMP, self.ncomponents, FORMAT,
			self.format)?;

		// Could refactor these as format_data_*() fns for each type
		if self.format == BINARY
		{
			writeln!(f, "					{}",
				base64::encode_f32(&self.points, self.endianness))?;
		}
		else
		{
			writeln!(f, "{}", SliceDisplay(&self.points))?;
		}

		writeln!(f, "				</{}>", DATA)?;

		writeln!(f, "			</{}>", POINTS)?;

		//********

		writeln!(f, "			<{}>", CELLS)?;

		writeln!(f, "				<{} {}=\"{}\" {}=\"{}\" {}=\"{}\">",
			DATA, TYPE, VTK_I64, NAME, CONN, FORMAT, self.format)?;
		if self.format == BINARY
		{
			writeln!(f, "					{}",
				base64::encode_i64(&self.connectivity, self.endianness))?;
		}
		else
		{
			writeln!(f, "{}", SliceDisplay(&self.connectivity))?;
		}
		writeln!(f, "				</{}>", DATA)?;

		writeln!(f, "				<{} {}=\"{}\" {}=\"{}\" {}=\"{}\">",
			DATA, TYPE, VTK_I64, NAME, OFFSETS, FORMAT, self.format)?;
		if self.format == BINARY
		{
			writeln!(f, "					{}",
				base64::encode_i64(&self.offsets, self.endianness))?;
		}
		else
		{
			writeln!(f, "{}", SliceDisplay(&self.offsets))?;
		}
		writeln!(f, "				</{}>", DATA)?;

		writeln!(f, "				<{} {}=\"{}\" {}=\"{}\" {}=\"{}\">",
			DATA, TYPE, VTK_U8, NAME, TYPES, FORMAT, self.format)?;
		if self.format == BINARY
		{
			writeln!(f, "					{}",
				base64::encode_u8(&self.types, self.endianness))?;
		}
		else
		{
			writeln!(f, "{}", SliceDisplay(&self.types))?;
		}
		writeln!(f, "				</{}>", DATA)?;

		writeln!(f, "			</{}>", CELLS)?;

		//********

		writeln!(f, "		</{}>", PIECE)?;
		writeln!(f, "	</{}>", self.vtype)?;
		writeln!(f, "</{}>", VTK_FILE)?;

		Ok(())
	};

	if let Err(_err) = try_export()
	{
		panic!("Cannot export VTK file \"{}\"", file);
	}

}}

//==============================================================================

impl VtkFile
{
pub fn convert(&mut self, settings: &Settings)
{
	self.endianness = if settings.le
	{
		base64::LITTLE_ENDIAN
	}
	else if settings.be
	{
		base64::BIG_ENDIAN
	}
	else
	{
		self.endianness
	};

	// Can't use conditional assignment like above because `self.format` has
	// type `String`, which does not implement the `Copy` trait
	if settings.ascii
	{
		self.format = ASCII.to_string();
	}
	else if settings.binary
	{
		self.format = BINARY.to_string();
	}

}}

//==============================================================================

fn check_type(dh: &DataHeader, expected: &str)
{
	if dh.dtype != expected
	{
		panic!("Expected type {} for {} {}.  Found type {}",
			expected, DATA, dh.name, dh.dtype);
	}
}

//==============================================================================

// TODO:  generic fns that return any type Vec

//fn parse_data<T: i64>(dh: &DataHeader, string: &str, v: &VtkFile) -> Vec<T>
//{
//	check_type(&dh, VTK_I64);
//	return if dh.format == BINARY {
//		base64::decode_i64(string, v.endianness)
//		//base64::decode_i64(string, v.endianness) as Vec<T>
//		//Vec<T>::from(base64::decode_i64(string, v.endianness))
//	} else {
//		unimplemented!("format {} is not implemented", dh.format)
//	};
//	return v;
//}

fn parse_data_f32(dh: &DataHeader, string: &str, v: &VtkFile) -> Vec<f32>
{
	check_type(&dh, VTK_F32);
	return if dh.format == BINARY {
		base64::decode_f32(string, v.endianness)
	} else {
		// TODO ascii, appended (raw binary)
		unimplemented!("format {} is not implemented", dh.format)
	};
}

fn parse_data_i64(dh: &DataHeader, string: &str, v: &VtkFile) -> Vec<i64>
{
	check_type(&dh, VTK_I64);
	return if dh.format == BINARY {
		base64::decode_i64(string, v.endianness)
	} else {
		unimplemented!("format {} is not implemented", dh.format)
	};
}

fn parse_data_u8(dh: &DataHeader, string: &str, v: &VtkFile) -> Vec<u8>
{
	check_type(&dh, VTK_U8);
	return if dh.format == BINARY {
		base64::decode_u8(string, v.endianness)
	} else {
		unimplemented!("format {} is not implemented", dh.format)
	};
}

//==============================================================================

#[cfg(test)]
mod tests
{
	use super::*;

	fn icosahedron() -> VtkFile
	{
		let ico = VtkFile
		{

			vtype: "UnstructuredGrid".to_string(),
			version: "1.0".to_string(),
			endianness: base64::LITTLE_ENDIAN,
			format: BINARY.to_string(),

			npoints: 12,
			ncells: 20,
			ncomponents: 3,

			points: [0.2763932, 0.8506508, 0.4472136,
				-0.7236068, 0.5257311, 0.4472136, -0.7236068, -0.5257311,
				0.4472136, 0.2763932, -0.8506508, 0.4472136, 0.8944272,
				-2.190715e-16, 0.4472136, -0.2763932, 0.8506508, -0.4472136,
				-0.8944272, 1.095357e-16, -0.4472136, -0.2763932, -0.8506508,
				-0.4472136, 0.7236068, -0.5257311, -0.4472136, 0.7236068,
				0.5257311, -0.4472136, 0.0, 0.0, 1.0, 1.224647e-16, 0.0,
				-1.0].to_vec(),

			connectivity: [0, 1, 10, 1, 2, 10, 2, 3, 10, 3, 4, 10, 4, 0, 10, 1,
				0, 5, 2, 1, 6, 3, 2, 7, 4, 3, 8, 0, 4, 9, 5, 6, 1, 6, 7, 2, 7,
				8, 3, 8, 9, 4, 9, 5, 0, 6, 5, 11, 7, 6, 11, 8, 7, 11, 9, 8, 11,
				5, 9, 11].to_vec(),

			offsets: [3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45,
				48, 51, 54, 57, 60].to_vec(),

			types: [6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5,
				5, 5].to_vec()

		};
		return ico;
	}

	#[test]
	fn test_load()
	{
		let v = load("./data/icosahedron-binary.vtu");
		let ico = icosahedron();

		// Dirty struct comparison:  format as String
		let vs = format!("{:?}", v);
		let icos = format!("{:?}", ico);

		assert_eq!(vs, icos);
	}

	// TODO: ascii tests (need ascii load fn first)

	#[test]
	fn test_ico()
	{
		let ico = icosahedron();
		let temp = "./scratch/tmp.spGwQCRZ3V.vtu";
		ico.export(temp);

		let v = load(temp);
		assert_eq!(format!("{:?}", v), format!("{:?}", ico));
	}
}

//==============================================================================

