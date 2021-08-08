
# vtk-io-rs

VTK file input/output in rust

This is a toy for me to learn rust.  It is not even close to feature complete.  If you're looking for a VTK IO library in rust, try [vtkio](https://github.com/elrnv/vtkio).

## Supported features

- Binary XML load
- ASCII XML export
- Binary XML export
- Big endian IO
- Little endian IO

## TODO

- Error handling:
  - Probably should return Result instead of panic! or expect()
  - Remove all unwrap()'s (done already?)
- ascii xml
- XML parsing
  - Point data
    - See pdata.. file
    - Results may vary in type (e.g. f32, f64, or other!)
    - Test scalar, vector, and tensor (sym & unsym?)
  - Cell data
  - Other types?  e.g. structured grid
    - Only unstructured grid implemented for now
- appended (raw binary) xml
- legacy ascii
- legacy binary
- XML compression.  At least parse and throw error
- VTK file export (legacy and XML, all options above)
- Find repo of test VTK files.  Can open legacy in PV and save as XML or vice versa

