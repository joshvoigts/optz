# optz

A no-magic option parser for Rust.


## Usage

```rust
use optz::{Opt, Optz};

let opts = Optz::new("myapp")
  .option(
    Opt::new("blah")
      .description("An operation that performs blah")
      .short("-b"))
  .parse();
```


## TODO

- [ ] Check types during parsing instead of at `get()`
- [ ] Remove anyhow dependency
