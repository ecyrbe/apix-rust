use indexmap::IndexMap;
use serde_json::Value;
use tera::{Context, Error, Tera};

pub trait ValueTemplate {
  fn render_value(&mut self, name: &str, value: &Value, context: &Context) -> Result<Value, Error>;
}

impl ValueTemplate for Tera {
  fn render_value(&mut self, name: &str, value: &Value, context: &Context) -> Result<Value, Error> {
    match value {
      Value::Object(obj) => {
        let mut new_obj = serde_json::Map::new();
        for (key, val) in obj {
          new_obj.insert(
            key.clone(),
            self.render_value(&format!("{}.{}", name, key), val, context)?,
          );
        }
        Ok(Value::Object(new_obj))
      }
      Value::Array(arr) => {
        let mut new_arr = Vec::new();
        for (index, val) in arr.iter().enumerate() {
          new_arr.push(self.render_value(&format!("{}.{}", name, index), val, context)?);
        }
        Ok(Value::Array(new_arr))
      }
      Value::String(content) => {
        self.add_raw_template(name, content)?;
        let new_content = self.render(&name, context)?;
        Ok(Value::String(new_content))
      }
      _ => Ok(value.clone()),
    }
  }
}

pub trait MapTemplate {
  fn render_map(
    &mut self,
    name: &str,
    map: &IndexMap<String, String>,
    context: &Context,
  ) -> Result<IndexMap<String, String>, Error>;
}

impl MapTemplate for Tera {
  fn render_map(
    &mut self,
    name: &str,
    map: &IndexMap<String, String>,
    context: &Context,
  ) -> Result<IndexMap<String, String>, Error> {
    let mut new_map = IndexMap::new();
    for (key, val) in map {
      let template_name = format!("{}.{}", name, key);
      self.add_raw_template(&template_name, val)?;
      let new_content = self.render(&template_name, context)?;
      new_map.insert(key.clone(), new_content);
    }
    Ok(new_map)
  }
}

pub trait StringTemplate {
  fn render_string(&mut self, name: &str, content: &str, context: &Context) -> Result<String, Error>;
}

impl StringTemplate for Tera {
  fn render_string(&mut self, name: &str, content: &str, context: &Context) -> Result<String, Error> {
    self.add_raw_template(name, content)?;
    self.render(name, context)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;
  use tera::{Context, Tera};

  #[test]
  fn test_render_value_object() {
    let mut tera = Tera::default();
    let mut context = Context::new();
    context.insert("context", &json!({ "test": "test" }));
    let rendered = tera
      .render_value(
        "test",
        &json!({"test": "{{context.test}}","another": "{{context.test}}"}),
        &context,
      )
      .expect("render value object");
    assert_eq!(rendered, json!({"test":"test","another":"test"}));
  }

  #[test]
  fn test_render_value_array() {
    let mut tera = Tera::default();
    let mut context = Context::new();
    context.insert("context", &json!({ "test": "test" }));
    let rendered = tera
      .render_value("test", &json!(["{{context.test}}", "{{context.test}}"]), &context)
      .expect("render value array");
    assert_eq!(rendered, json!(["test", "test"]));
  }

  #[test]
  fn test_render_map() {
    let mut tera = Tera::default();
    let mut context = Context::new();
    context.insert("context", &json!({ "test": "test" }));
    let rendered = tera
      .render_map(
        "test",
        &IndexMap::from_iter(vec![("test".to_string(), "{{context.test}}".to_string())]),
        &context,
      )
      .expect("render map");
    assert_eq!(
      rendered,
      IndexMap::<String, String>::from_iter(vec![("test".to_string(), "test".to_string())])
    );
  }

  #[test]
  fn test_render_string() {
    let mut tera = Tera::default();
    let mut context = Context::new();
    context.insert("context", &json!({ "test": "test" }));
    let rendered = tera
      .render_string("test", "{{context.test}} {{context.test}}", &context)
      .expect("render string");
    assert_eq!(rendered, "test test");
  }
}
