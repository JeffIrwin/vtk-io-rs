
mod base64;
mod utils;
mod vtk;

//==============================================================================

fn main()
{
	// TODO:  don't unwrap
	let this = utils::this().unwrap();
	println!("\n{}:  starting\n", this);

	// Get command line args or other configuration settings
	let settings = utils::get_settings(&this);

	let mut v = vtk::load(&settings.input);

	v.convert(&settings);
	v.export(&settings.output);

	println!("{}:  done", this);
}

//==============================================================================

