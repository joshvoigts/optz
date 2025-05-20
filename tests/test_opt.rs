use optz::{Opt, Optz, OptzError};
use std::sync::{LazyLock, Mutex};

#[test]
fn test_flag() {
  let optz = Optz::from_args(
    "test",
    vec!["test", "--verbose"],
  )
  .option(Opt::flag("verbose"))
  .parse()
  .unwrap();
  let result: bool = optz.get("verbose").unwrap().unwrap();
  assert_eq!(result, true);
}

#[test]
fn test_args() {
  let optz = Optz::from_args(
    "test",
    vec![
      "test",
      "--num-items",
      "12",
    ],
  )
  .option(Opt::arg("num-items"))
  .parse()
  .unwrap();
  let result: u64 = optz.get("num-items").unwrap().unwrap();
  assert_eq!(result, 12u64);
}

#[test]
fn test_short_option() {
  let optz = Optz::from_args(
    "test",
    vec!["test", "-v"],
  )
  .option(Opt::flag("verbose").short("-v"))
  .parse()
  .unwrap();
  let result: bool = optz.get("verbose").unwrap().unwrap();
  assert_eq!(result, true);
}

#[test]
fn test_rest_arguments() {
  let optz = Optz::from_args(
    "test",
    vec![
      "test",
      "--verbose",
      "file1",
      "file2",
    ],
  )
  .option(Opt::flag("verbose"))
  .parse()
  .unwrap();
  assert_eq!(optz.rest, vec!["file1", "file2"]);
}

#[test]
fn test_config() {
  #[derive(Debug, PartialEq)]
  struct MyConfig {
    value: i32,
  }
  let config = MyConfig { value: 42 };
  let optz = Optz::new("test").config(config).parse().unwrap();
  let retrieved: &MyConfig = optz.get_config().unwrap();
  assert_eq!(*retrieved, MyConfig { value: 42 });
}

#[test]
fn test_default_value() {
  let optz = Optz::from_args("test", vec!["test"])
    .option(Opt::arg("count").default_value("5"))
    .parse()
    .unwrap();
  let result: u32 = optz.get("count").unwrap().unwrap();
  assert_eq!(result, 5);
}

#[test]
fn test_missing_argument() {
  let optz = Optz::from_args(
    "test",
    vec!["test", "--num-items"],
  )
  .option(Opt::arg("num-items"))
  .parse();
  assert!(optz.is_err());
}

#[test]
fn test_unknown_option_ignored() {
  let optz = Optz::from_args(
    "test",
    vec!["test", "--unknown"],
  )
  .option(Opt::flag("verbose"))
  .parse()
  .unwrap();
  assert!(optz.rest.is_empty());
}

#[test]
fn test_usage_default() {
  let optz = Optz::new("myprog").parse().unwrap();
  assert_eq!(optz.usage, Some("Usage: myprog [options]".to_string()));
}

#[test]
fn test_multiple_options() {
  let optz = Optz::from_args(
    "test",
    vec![
      "test",
      "--verbose",
      "-n",
      "10",
    ],
  )
  .option(Opt::flag("verbose"))
  .option(Opt::arg("num").short("-n"))
  .parse()
  .unwrap();

  let verbose: bool = optz.get("verbose").unwrap().unwrap();
  let num: u32 = optz.get("num").unwrap().unwrap();

  assert!(verbose);
  assert_eq!(num, 10);
}

#[test]
fn test_help_option_auto_added() {
  let optz = Optz::from_args("test", vec!["test"])
    .option(Opt::flag("verbose"))
    .parse()
    .unwrap();

  let has_help = optz.options.iter().any(|opt| opt.name == "help");
  assert!(has_help);
}

#[test]
fn test_multiple_values() {
  let optz = Optz::from_args(
    "test",
    vec!["test", "--num-items", "10", "--num-items", "20"],
  )
  .option(Opt::arg("num-items").multiple(true))
  .parse()
  .unwrap();
  let result: Vec<u32> = optz.get_values("num-items").unwrap();
  assert_eq!(result, vec![10, 20]);
}

// Use `LazyLock` to initialize the static variable lazily
static CALLED: LazyLock<Mutex<bool>> =
  LazyLock::new(|| Mutex::new(false));

fn handler(_optz: &Optz) -> Result<(), OptzError> {
  let mut called = CALLED.lock().unwrap();
  *called = true;
  Ok(())
}

#[test]
fn test_custom_handler() {
  let _ = Optz::from_args(
    "test",
    vec!["test", "--test"],
  )
  .option(Opt::flag("test").handler(handler))
  .parse()
  .unwrap();

  assert!(*CALLED.lock().unwrap());
}

#[test]
fn test_handler_error() {
  let result = Optz::from_args(
    "test",
    vec!["test", "--error"],
  )
  .option(
    Opt::flag("error")
      .handler(|_| Err(OptzError::Parse("Custom error".to_string()))),
  )
  .parse();
  assert!(result.is_err());
  if let Err(OptzError::Parse(msg)) = result {
    assert_eq!(msg, "Custom error");
  } else {
    panic!("Unexpected error type");
  }
}

#[test]
fn test_short_and_long_options() {
  let optz = Optz::from_args(
    "test",
    vec![
      "test",
      "-v",
      "--verbose",
    ],
  )
  .option(Opt::flag("verbose").short("-v"))
  .parse()
  .unwrap();
  let result: bool = optz.get("verbose").unwrap().unwrap();
  assert_eq!(result, true);
}
