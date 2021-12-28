use super::manifests::ApixParameter;
use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Input, Password};
use jsonschema::{Draft, JSONSchema};
use serde_json::Value;

fn input_to_value(input: &String) -> Value {
    return serde_json::from_str(&input)
        .or::<serde_json::Error>(Ok(Value::String(input.clone())))
        .unwrap();
}

pub trait Dialog {
    fn ask(&self) -> Result<Value>;
}

impl Dialog for ApixParameter {
    fn ask(&self) -> Result<Value> {
        let value_schema = self.schema.as_ref().unwrap();
        let schema = JSONSchema::options()
            .with_draft(Draft::Draft7)
            .compile(value_schema)
            .map_err(|err| anyhow::anyhow!("{}", err))?;
        if self.required {
            if self.password {
                let input = Password::with_theme(&ColorfulTheme::default())
                    .with_prompt(&self.name)
                    .interact()?;

                Ok(Value::String(input))
            } else {
                let input = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt(&self.name)
                    .validate_with(|input: &String| {
                        let value = input_to_value(input);
                        let result = schema.validate(&value);
                        if let Err(errors) = result {
                            let mut msg: Vec<String> = vec!["Invalid input:".to_string()];
                            for (index, cause) in errors.enumerate() {
                                msg.push(format!("cause {}: {}", index, cause));
                            }
                            return Err(msg.join("\n"));
                        }
                        Ok(())
                    })
                    .interact_text()?;

                Ok(input_to_value(&input))
            }
        } else {
            Ok(Value::Null)
        }
    }
}
