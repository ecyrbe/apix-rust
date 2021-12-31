use indicatif::{ProgressBar, ProgressStyle};

pub struct FileProgress {
  path: String,
  progress: ProgressBar,
}

pub enum FileProgressComponent {
  Download(FileProgress),
  Upload(FileProgress),
}

impl FileProgress {
  fn new(path: String, size_hint: u64) -> Self {
    let progress = ProgressBar::new(size_hint);
    progress.set_style(ProgressStyle::default_bar().template(
      "{msg} - {percent}%\n{spinner:.green} [{elapsed_precise}] {wide_bar:.cyan/blue} {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
    ));
    Self { path, progress }
  }
}

impl FileProgressComponent {
  pub fn new_download(path: String, size_hint: u64) -> Self {
    let progress = FileProgress::new(path, size_hint);
    FileProgressComponent::Download(progress)
  }
  pub fn new_upload(path: String, size_hint: u64) -> Self {
    let progress = FileProgress::new(path, size_hint);
    FileProgressComponent::Upload(progress)
  }
  pub fn update_progress(&self, bytes: u64) {
    match self {
      FileProgressComponent::Download(component) => {
        component
          .progress
          .set_message(format!("Downloading File {}", component.path));
        component.progress.inc(bytes);
        if component.progress.is_finished() {
          component.progress.finish_with_message("Download Complete");
        }
      }
      FileProgressComponent::Upload(component) => {
        component
          .progress
          .set_message(format!("Uploading File {}", component.path));
        component.progress.inc(bytes);
        if component.progress.is_finished() {
          component.progress.finish_with_message("Upload Complete");
        }
      }
    }
  }
}
