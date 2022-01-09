use anyhow::Result;
use std::ffi::OsString;

// get the user default editor
fn get_default_editor() -> OsString {
  if let Some(prog) = std::env::var_os("VISUAL") {
    return prog;
  }
  if let Some(prog) = std::env::var_os("EDITOR") {
    return prog;
  }
  if cfg!(windows) {
    "notepad.exe".into()
  } else {
    "vi".into()
  }
}

// edit file with default editor
pub fn edit_file(file: &str) -> Result<()> {
  let editor = get_default_editor();
  std::process::Command::new(&editor).arg(file).spawn()?.wait()?;
  Ok(())
}
