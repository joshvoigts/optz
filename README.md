# opt

A no-magic option parser for Rust.


## Usage

```rust
use opt::{Opts, Opt};

let opts = Opts::new("myapp")
  .option(
    Opt::new("blah")
      .description("An operation that performs blah")
      .short("-b"))
  .parse();
```
