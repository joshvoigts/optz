# optz

A no-magic option parser for Rust.

`optz` provides a simple, flexible, and minimalistic way to parse
command-line options in Rust applications. It focuses on clarity
and control, avoiding "magic" behavior in favor of explicit
configuration.


## Usage

### Basic Example

```rust
use optz::{Opt, Optz};

let optz = Optz::new("myapp")
  .option(
    Opt::flag("verbose")
      .description("Enable verbose mode")
      .short("-v")
  )
  .option(
    Opt::arg("num-items")
      .description("Number of items to process")
      .default_value("5")
  )
  .parse()
  .unwrap();

if optz.get("verbose").unwrap().unwrap() {
  println!("Verbose mode enabled");
}

let count: u32 = optz.get("num-items").unwrap().unwrap();
println!("Processing {} items", count);
```

### Example with Configuration

```rust
#[derive(Debug, PartialEq)]
struct MyConfig {
  value: i32,
}

let config = MyConfig { value: 42 };
let optz = Optz::new("myapp")
  .config(config)
  .option(
    Opt::flag("verbose")
      .description("Enable verbose output")
      .short("-v")
  )
  .parse()
  .unwrap();

let retrieved: &MyConfig = optz.get_config().unwrap();
assert_eq!(*retrieved, MyConfig { value: 42 });
```

### Example with Handlers

```rust
fn my_handler(_optz: &Optz) -> Result<(), OptzError> {
  println!("Custom handler called!");
  Ok(())
}

let optz = Optz::from_args("test", vec!["test", "--custom"])
  .option(
    Opt::flag("custom")
      .description("Trigger custom logic")
      .handler(my_handler)
  )
  .parse()
  .unwrap();
```


## TODO

- [ ] Check types during parsing instead of at `get()`
