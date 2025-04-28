use anyhow::Result;
use std::any::Any;
use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::process;

use crate::fail;

#[derive(Default)]
pub struct Opts {
  pub args: BTreeSet<String>,
  pub handler: Option<fn(&Opts) -> Result<()>>,
  pub name: String,
  pub usage: Option<String>,
  pub description: Option<String>,
  pub authors: Vec<String>,
  pub options: Vec<Opt>,
  pub config: Option<Box<dyn Any>>, // Any user-defined type
  pub rest: Vec<String>,
}

impl Opts {
  pub fn new(name: &str) -> Self {
    Self {
      args: env::args().skip(1).collect(),
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

  pub fn get(&self, name: &str) -> Option<String> {
    for opt in &self.options {
      if opt.name != name {
        continue;
      }
      return opt.value.clone();
    }
    None
  }

  pub fn handler(mut self, handler: fn(&Opts) -> Result<()>) -> Self {
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
    process::exit(0);
  }

  pub fn option(mut self, opt: Opt) -> Self {
    self.options.push(opt);
    self
  }

  pub fn parse(mut self) -> Self {
    if self.usage.is_none() {
      self.usage = Some(format!("Usage: {} [options]", self.name));
    }

    self.options.push(
      Opt::new("help")
        .description("Show help")
        .short("-h")
        .handler(Self::help),
    );

    let mut args_iter = self.args.iter();
    while let Some(arg) = args_iter.next() {
      if arg == "-" {
        continue;
      }
      let mut has_opt = false;
      for opt in self.options.iter_mut() {
        if &opt.long == arg || opt.short == Some(arg.clone()) {
          if opt.arg.is_some() {
            opt.value = Some(
              args_iter
                .next()
                .expect("Expected argument for option")
                .clone(),
            );
          } else {
            opt.value = Some("true".into());
          }
          has_opt = true;
          break;
        }
      }
      if !has_opt {
        self.rest.push(arg.clone());
      }
    }

    for opt in self.options.iter() {
      if opt.value.is_some() {
        if let Some(handler) = opt.handler {
          let res = handler(&self);
          if !res.is_ok() {
            fail!("{}: {}", self.name, res.unwrap_err());
          }
        }
      }
    }

    if let Some(handler) = self.handler {
      let res = handler(&self);
      if !res.is_ok() {
        fail!("{}: {}", self.name, res.unwrap_err());
      }
    }

    if self.args.len() == 0 {
      let _ = self.help();
    }

    self
  }

  pub fn usage<S: Into<String>>(mut self, text: S) -> Self {
    self.usage = Some(text.into());
    self
  }
}

impl IntoIterator for Opts {
  type Item = Opt;
  type IntoIter = std::vec::IntoIter<Self::Item>;

  fn into_iter(self) -> Self::IntoIter {
    self.options.into_iter()
  }
}

#[derive(Clone, Default)]
pub struct Opt {
  pub description: Option<String>,
  pub handler: Option<fn(&Opts) -> Result<()>>,
  pub long: String,
  pub name: String,
  pub short: Option<String>,
  pub value: Option<String>,
  pub arg: Option<String>,
}

impl Opt {
  pub fn new(name: &str) -> Self {
    let long = format!("--{}", name);
    Self {
      name: name.to_owned(),
      long: long,
      ..Default::default()
    }
  }

  pub fn arg(mut self, arg: &str) -> Self {
    self.arg = Some(arg.into());
    self
  }

  pub fn description(mut self, desc: &str) -> Self {
    self.description = Some(desc.into());
    self
  }

  pub fn handler(mut self, handler: fn(&Opts) -> Result<()>) -> Self {
    self.handler = Some(handler);
    self
  }

  pub fn short(mut self, short: &str) -> Self {
    self.short = Some(short.into());
    self
  }

  pub fn value(mut self, value: &str) -> Self {
    self.value = Some(value.into());
    self
  }
}

impl fmt::Debug for Opt {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Opt")
      .field("description", &self.description)
      .field("handler", &"handler")
      .field("long", &self.long)
      .field("short", &self.short)
      .finish()
  }
}
