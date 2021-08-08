#!/bin/bash

# "--" is optional for some arguments, but not others.  e.g. "cargo run -h"
# will show cargo help instead of my exe's help
cargo run -- ./data/icosahedron-binary.vtu scratch/test.vtu

