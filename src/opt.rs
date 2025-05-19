use std::any::Any;
use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum OptzError {
  MissingArgument,
  Parse(String),
}

impl std::fmt::Display for OptzError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      OptzError::MissingArgument => write!(f, "Missing argument"),
      OptzError::Parse(msg) => write!(f, "{}", msg),
    }
  }
}

impl std::error::Error for OptzError {}

type Result<T> = std::result::Result<T, OptzError>;

#[derive(Debug, Default)]
pub struct Optz {
  pub args: BTreeSet<String>,
  pub handler: Option<fn(&Optz) -> Result<()>>,
  pub name: String,
  pub usage: Option<String>,
  pub description: Option<String>,
  pub authors: Vec<String>,
  pub options: Vec<Opt>,
  pub config: Option<Box<dyn Any>>,
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

  pub fn get_config<T: 'static>(&self) -> Option<&T> {
    self.config.as_ref().and_then(|c| c.downcast_ref::<T>())
  }

  pub fn description<S: Into<String>>(mut self, text: S) -> Self {
    self.description = Some(text.into());
    self
  }

  pub fn get<T: FromStr>(&self, name: &str) -> Result<Option<T>>
  where
    <T as FromStr>::Err: std::fmt::Debug,
  {
    for opt in &self.options {
      if opt.name != name {
        continue;
      }

      if !opt.values.is_empty() {
        let value = opt.values.first().unwrap().clone();
        return Ok(Some(
          value
            .parse::<T>()
            .map_err(|e| OptzError::Parse(format!("{:?}", e)))?,
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
              Arg::Flag => {
                if opt.multiple {
                  opt.values.push("true".to_string());
                } else {
                  opt.values = vec!["true".to_string()];
                }
              }
              Arg::Arg => {
                let next_arg = args_iter.next();
                match next_arg {
                  Some(arg) => {
                    if opt.multiple {
                      opt.values.push(arg.clone());
                    } else {
                      opt.values = vec![arg.clone()];
                    }
                  }
                  None => {
                    return Err(OptzError::MissingArgument);
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
      if !opt.values.is_empty() {
        if let Some(handler) = opt.handler {
          let res = handler(&self);
          if !res.is_ok() {
            return Err(OptzError::Parse(
              res.unwrap_err().to_string(),
            ));
          }
        }
      }
    }

    if let Some(handler) = self.handler {
      let res = handler(&self);
      if !res.is_ok() {
        return Err(OptzError::Parse(res.unwrap_err().to_string()));
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
  pub multiple: bool,
  pub name: String,
  pub short: Option<String>,
  pub values: Vec<String>,
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

  pub fn multiple(mut self, multiple: bool) -> Self {
    self.multiple = multiple;
    self
  }

  pub fn default_value(mut self, value: &str) -> Self {
    self.values = vec![value.to_owned()];
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
      .field("values", &self.values)
      .finish()
  }
}
