use optz::{Opt, Optz};

#[test]
fn test_flag() {
  let optz = Optz::from_args(
    "test",
    vec!["test".to_string(), "--verbose".to_string()],
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
      "test".to_string(),
      "--num-items".to_string(),
      "12".to_string(),
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
    vec!["test".to_string(), "-v".to_string()],
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
      "test".to_string(),
      "--verbose".to_string(),
      "file1".to_string(),
      "file2".to_string(),
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
  let optz = Optz::from_args("test", vec!["test".to_string()])
    .option(Opt::arg("count").default_value("5"))
    .parse()
    .unwrap();
  let result: u32 = optz.get("count").unwrap().unwrap();
  assert_eq!(result, 5);
}

#[test]
#[should_panic(expected = "test: Missing argument")]
fn test_missing_argument() {
  let _ = Optz::from_args(
    "test",
    vec!["test".to_string(), "--num-items".to_string()],
  )
  .option(Opt::arg("num-items"))
  .parse()
  .unwrap();
}

#[test]
fn test_unknown_option_ignored() {
  let optz = Optz::from_args(
    "test",
    vec!["test".to_string(), "--unknown".to_string()],
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
      "test".to_string(),
      "--verbose".to_string(),
      "-n".to_string(),
      "10".to_string(),
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
  let optz = Optz::from_args("test", vec!["test".to_string()])
    .option(Opt::flag("verbose"))
    .parse()
    .unwrap();

  let has_help = optz.options.iter().any(|opt| opt.name == "help");
  assert!(has_help);
}
