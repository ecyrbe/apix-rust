use anyhow::Result;
use clap::ArgMatches;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indexmap::IndexMap;
pub trait MatchPrompts {
  fn match_or_input(&self, name: &str, msg: &str) -> Result<String>;
  fn match_or_validate_input<V: FnMut(&String) -> Result<()>>(
    &self,
    name: &str,
    msg: &str,
    validator: V,
  ) -> Result<String>;
  fn match_or_input_multiples(&self, name: &str, msg: &str) -> Result<IndexMap<String, String>>;
  fn match_or_optional_input(&self, name: &str, msg: &str) -> Result<Option<String>>;
  fn match_or_select<T: ToString>(&self, name: &str, msg: &str, options: &[T]) -> Result<String>;
}

impl MatchPrompts for ArgMatches {
  fn match_or_input(&self, name: &str, msg: &str) -> Result<String> {
    match self.value_of(name) {
      Some(value) => Ok(value.to_string()),
      None => Ok(
        Input::with_theme(&ColorfulTheme::default())
          .with_prompt(msg)
          .interact_text()?,
      ),
    }
  }
  fn match_or_validate_input<V>(&self, name: &str, msg: &str, validator: V) -> Result<String>
  where
    V: FnMut(&String) -> Result<()>,
  {
    match self.value_of(name) {
      Some(value) => Ok(value.to_string()),
      None => Ok(
        Input::with_theme(&ColorfulTheme::default())
          .with_prompt(msg)
          .validate_with(validator)
          .interact_text()?,
      ),
    }
  }

  fn match_or_input_multiples(&self, name: &str, msg: &str) -> Result<IndexMap<String, String>> {
    match self.values_of(name) {
      Some(values) => {
        let mut map = IndexMap::new();
        for value in values {
          let mut parts = value.splitn(2, ':');
          let key = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("No key found in '{}'", value))?;
          let value = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("No value found in '{}'", value))?;
          map.insert(key.to_string(), value.to_string());
        }
        Ok(map)
      }
      None => {
        let mut map = IndexMap::new();
        loop {
          let add = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(msg)
            .interact()?;
          if add {
            let key = Input::with_theme(&ColorfulTheme::default())
              .with_prompt(format!("{} name", name))
              .interact_text()?;
            let value = Input::with_theme(&ColorfulTheme::default())
              .with_prompt(format!("{} value", name))
              .interact_text()?;
            map.insert(key, value);
          } else {
            break;
          }
        }
        Ok(map)
      }
    }
  }

  fn match_or_optional_input(&self, name: &str, msg: &str) -> Result<Option<String>> {
    match self.value_of(name) {
      Some(value) => Ok(Some(value.to_string())),
      None => {
        if Confirm::with_theme(&ColorfulTheme::default())
          .with_prompt(msg)
          .interact()?
        {
          Ok(Some(
            Input::with_theme(&ColorfulTheme::default())
              .with_prompt(name)
              .interact_text()?,
          ))
        } else {
          Ok(None)
        }
      }
    }
  }

  fn match_or_select<T: ToString>(&self, name: &str, msg: &str, options: &[T]) -> Result<String> {
    match self.value_of(name) {
      Some(value) => Ok(value.to_string()),
      None => {
        let select = Select::with_theme(&ColorfulTheme::default())
          .with_prompt(msg)
          .items(options)
          .interact()?;
        Ok(options[select].to_string())
      }
    }
  }
}
