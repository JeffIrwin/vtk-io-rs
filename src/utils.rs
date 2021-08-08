
use std::env;
use std::path::Path;
use std::ffi::OsStr;

use clap::{Arg, App};

use crate::vtk;

// Get the filename of this executable
pub fn this() -> Option<String>
{
	env::current_exe().ok()
		.as_ref()
		.map(Path::new)
		.and_then(Path::file_name)
		.and_then(OsStr::to_str)
		.map(String::from)
}

// Command line arg IDs
pub const INPUT : &str = "INPUT";
pub const OUTPUT: &str = "OUTPUT";
pub const LEND  : &str = "little-endian";
pub const BEND  : &str = "big-endian";
pub const ASCII : &str = "ascii";
pub const BINARY: &str = "binary";

pub fn get_settings(app_name: &str) -> vtk::Settings
{

	//return App::new(app_name)
	let args = App::new(app_name)

		.version("0.1.0")
		.author("https://github.com/JeffIrwin")
		.about("VTK file input/output toy")

		.arg(Arg::with_name(INPUT)
			.help("Sets the input VTK file to load")
			.required(true)
			.index(1))

		.arg(Arg::with_name(OUTPUT)
			.help("Sets the output VTK file to export")
			.required(true)
			.index(2))

		.arg(Arg::with_name(LEND)
			.long("le")
			.help("Sets little endian output format"))

		.arg(Arg::with_name(BEND)
			.long("be")
			.help("Sets big endian output format"))

		.arg(Arg::with_name(ASCII)
			.short("a")
			.long(ASCII)
			.help("Sets ASCII output format"))

		.arg(Arg::with_name(BINARY)
			.short("b")
			.long(BINARY)
			.help("Sets binary (base64 encoded) output format"))

		.get_matches();

	// Instead of returning the args struct, abstract it to a settings struct.
	// This leaves room for extension, in case other settings are loaded later
	// e.g. from a JSON file
	let settings = vtk::Settings
	{
		input : args.value_of(INPUT ).unwrap().to_string(),
		output: args.value_of(OUTPUT).unwrap().to_string(),
		le    : args.is_present(LEND),
		be    : args.is_present(BEND),
		ascii : args.is_present(ASCII),
		binary: args.is_present(BINARY),
	};

	return settings;
}

