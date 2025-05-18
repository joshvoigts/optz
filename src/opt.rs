use anyhow::{anyhow, Result};
use std::any::Any;
use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct Optz {
  pub args: BTreeSet<String>,
  pub handler: Option<fn(&Optz) -> Result<()>>,
  pub name: String,
  pub usage: Option<String>,
  pub description: Option<String>,
  pub authors: Vec<String>,
  pub options: Vec<Opt>,
  pub config: Option<Box<dyn Any>>, // Any user-defined type
  pub rest: Vec<String>,
}

impl Optz {
  pub fn new(name: &str) -> Self {
    Optz::from_args(name, env::args().collect())
  }

  pub fn from_args(name: &str, args: Vec<String>) -> Self {
    Self {
      args: args.into_iter().skip(1).collect(),
      name: name.into(),
      ..Default::default()
    }
  }

  pub fn config<T: 'static>(mut self, config: T) -> Self {
    self.config = Some(Box::new(config));
    self
  }

  // Retrieve the config as a specific type
  pub fn get_config<T: 'static>(&self) -> Option<&T> {
    self.config.as_ref().and_then(|c| c.downcast_ref::<T>())
  }

  pub fn description<S: Into<String>>(mut self, text: S) -> Self {
    self.description = Some(text.into());
    self
  }

  pub fn get<T: FromStr>(&self, name: &str) -> Result<Option<T>>
  where
    <T as FromStr>::Err: std::fmt::Display,
  {
    for opt in &self.options {
      if opt.name != name {
        continue;
      }

      if let Some(value) = &opt.value {
        return Ok(Some(
          value
            .to_string()
            .parse::<T>()
            .map_err(|e| anyhow!("{}", e))?,
        ));
      }
    }
    Ok(None)
  }

  pub fn handler(mut self, handler: fn(&Optz) -> Result<()>) -> Self {
    self.handler = Some(handler);
    self
  }

  fn help(self: &Self) -> Result<()> {
    if let Some(usage) = &self.usage {
      println!("{}", usage);
    }
    for opt in self.options.iter() {
      let mut res = "  ".to_owned();
      if let Some(short) = &opt.short {
        res.push_str(short);
        res.push_str(", ");
      } else {
        res.push_str("    ");
      }
      res.push_str(format!("{:<12} ", opt.long).as_str());
      if let Some(desc) = &opt.description {
        res.push_str(desc);
      }
      println!("{}", res);
    }
    Ok(())
  }

  pub fn option(mut self, opt: Opt) -> Self {
    self.options.push(opt);
    self
  }

  pub fn parse(mut self) -> Result<Self> {
    if self.usage.is_none() {
      self.usage = Some(format!("Usage: {} [options]", self.name));
    }

    self.options.push(
      Opt::flag("help")
        .description("Show help")
        .short("-h")
        .handler(Self::help),
    );

    let mut args_iter = self.args.iter().peekable();
    while let Some(arg) = args_iter.next() {
      if arg == "-" {
        continue;
      }
      if arg.starts_with("-") {
        for opt in self.options.iter_mut() {
          if &opt.long == arg || opt.short == Some(arg.clone()) {
            match opt.arg {
              Arg::Flag => opt.value = Some("true".to_string()),
              Arg::Arg => {
                let next_arg = args_iter.next();
                match next_arg {
                  Some(arg) => opt.value = Some(arg.clone()),
                  None => {
                    return Err(anyhow!("{}: {}", self.name, "Missing argument"));
                  }
                }
              }
            }
            break;
          }
        }
      } else {
        self.rest.push(arg.clone());
      }
    }

    for opt in self.options.iter() {
      if opt.value.is_some() {
        if let Some(handler) = opt.handler {
          let res = handler(&self);
          if !res.is_ok() {
            return Err(anyhow!("{}: {}", self.name, res.unwrap_err()))
          }
        }
      }
    }

    if let Some(handler) = self.handler {
      let res = handler(&self);
      if !res.is_ok() {
        return Err(anyhow!("{}: {}", self.name, res.unwrap_err()));
      }
    }

    if self.args.len() == 0 {
      let _ = self.help();
    }

    Ok(self)
  }

  pub fn usage<S: Into<String>>(mut self, text: S) -> Self {
    self.usage = Some(text.into());
    self
  }
}

impl IntoIterator for Optz {
  type Item = Opt;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.options.into_iter()
  }
}

#[derive(Clone, Debug, Default)]
pub enum Arg {
  Arg,
  #[default]
  Flag,
}

#[derive(Clone, Default)]
pub struct Opt {
  pub arg: Arg,
  pub description: Option<String>,
  pub handler: Option<fn(&Optz) -> Result<()>>,
  pub long: String,
  pub name: String,
  pub short: Option<String>,
  pub value: Option<String>,
}

impl Opt {
  pub fn flag(name: &str) -> Self {
    let long = format!("--{}", name);
    Self {
      arg: Arg::Flag,
      name: name.to_owned(),
      long: long,
      ..Default::default()
    }
  }

  pub fn arg(name: &str) -> Self {
    let long = format!("--{}", name);
    Self {
      arg: Arg::Arg,
      name: name.to_owned(),
      long: long,
      ..Default::default()
    }
  }

  pub fn default_value(mut self, value: &str) -> Self {
    self.value = Some(value.try_into().expect("Invalid value"));
    self
  }

  pub fn description(mut self, desc: &str) -> Self {
    self.description = Some(desc.into());
    self
  }

  pub fn handler(mut self, handler: fn(&Optz) -> Result<()>) -> Self {
    self.handler = Some(handler);
    self
  }

  pub fn short(mut self, short: &str) -> Self {
    self.short = Some(short.into());
    self
  }
}

impl fmt::Debug for Opt {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Opt")
      .field("arg", &self.arg)
      .field("description", &self.description)
      .field("handler", &"handler")
      .field("long", &self.long)
      .field("name", &self.name)
      .field("short", &self.short)
      .field("value", &self.value)
      .finish()
  }
}

#[test]
fn test_flag() {
  let optz = Optz::from_args(
    "test",
    vec!["test".to_string(), "--verbose".to_string()],
  )
  .option(Opt::flag("verbose"))
  .parse().unwrap();
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
  .parse().unwrap();
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
  .parse().unwrap();
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
  .parse().unwrap();
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
    .parse().unwrap();
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
  .parse().unwrap();
}

#[test]
fn test_unknown_option_ignored() {
  let optz = Optz::from_args(
    "test",
    vec!["test".to_string(), "--unknown".to_string()],
  )
  .option(Opt::flag("verbose"))
  .parse().unwrap();
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
  .parse().unwrap();

  let verbose: bool = optz.get("verbose").unwrap().unwrap();
  let num: u32 = optz.get("num").unwrap().unwrap();

  assert!(verbose);
  assert_eq!(num, 10);
}

#[test]
fn test_help_option_auto_added() {
  let optz = Optz::from_args("test", vec!["test".to_string()])
    .option(Opt::flag("verbose"))
    .parse().unwrap();

  let has_help = optz.options.iter().any(|opt| opt.name == "help");
  assert!(has_help);
}
