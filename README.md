
aeroscore
==============================================================================

[![Build Status](https://travis-ci.org/Turbo87/aeroscore-rs.svg?branch=master)](https://travis-ci.org/Turbo87/aeroscore-rs)

Algorithms for Glider Pilots

The `aeroscore` project contains a variety of algorithms for glider pilots
that solve common task scoring problems like FAI triangle and OnlineContest
optimization, and competition scoring.

Please note that this is work in progress and the above statement is mostly
wishful thinking for now!


Usage
------------------------------------------------------------------------------

```rust
extern crate aeroscore;

use aeroscore::olc;

fn main() {
    // ...
    let result = olc::solve_classic(&gps_fixes);
}
```


License
-------------------------------------------------------------------------------

This project is released under the [MIT license](LICENSE).
