use crate::settings::{
    AppSettings, CaptureAction, CaptureFileSlot, FileTemplateType, OutputTargetType,
    RecognitionCleanupOptions, RecognitionProfile,
};
use chrono::Local;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct CaptureActionNotification {
    pub level: String,
    pub title: String,
    pub detail: Option<String>,
}

pub fn emit_capture_notification(
    app: &AppHandle,
    level: impl Into<String>,
    title: impl Into<String>,
    detail: Option<String>,
) {
    let _ = app.emit(
        "capture-action-notification",
        CaptureActionNotification {
            level: level.into(),
            title: title.into(),
            detail,
        },
    );
}

pub fn cleanup_text(text: &str, cleanup: &RecognitionCleanupOptions) -> String {
    let mut value = text.replace("\r\n", "\n").replace('\r', "\n");

    if cleanup.trim_whitespace {
        value = value.trim().to_string();
    }

    if cleanup.collapse_whitespace {
        let normalized_lines = value
            .lines()
            .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
            .collect::<Vec<_>>();
        value = if cleanup.preserve_newlines {
            normalized_lines.join("\n")
        } else {
            normalized_lines.join(" ")
        };
    } else if !cleanup.preserve_newlines {
        value = value.lines().map(str::trim).collect::<Vec<_>>().join(" ");
    }

    if cleanup.capitalize_first_letter {
        value = capitalize_first_letter(&value);
    }

    value.trim().to_string()
}

pub fn prepare_capture_text(text: &str, profile: &RecognitionProfile) -> String {
    cleanup_text(text, &profile.cleanup_options)
}

pub fn route_capture_result(
    app: &AppHandle,
    settings: &AppSettings,
    action: &CaptureAction,
    text: &str,
) -> Result<(), String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    match action.output_target.r#type {
        OutputTargetType::Paste => crate::utils::paste(trimmed.to_string(), app.clone()),
        OutputTargetType::AppendFile => {
            let file_slot = action
                .output_target
                .file_slot
                .ok_or_else(|| format!("Capture action '{}' is missing file_slot", action.id))?;
            let template_type = action.output_target.template_type.ok_or_else(|| {
                format!("Capture action '{}' is missing template_type", action.id)
            })?;
            let path = resolve_file_path(settings, file_slot);
            let entry = format_entry(template_type, trimmed, Local::now());
            append_entry(&path, &entry).map_err(|err| {
                format!("Failed to append capture action '{}' to '{}': {}", action.id, path.display(), err)
            })?;
            emit_capture_notification(
                app,
                "success",
                action.name.clone(),
                Some(path.display().to_string()),
            );
            Ok(())
        }
    }
}

pub fn resolve_file_path(settings: &AppSettings, slot: CaptureFileSlot) -> PathBuf {
    PathBuf::from(settings.file_path_for_slot(slot))
}

pub fn format_entry(
    template_type: FileTemplateType,
    text: &str,
    timestamp: chrono::DateTime<Local>,
) -> String {
    let stamp = timestamp.format("%Y-%m-%d %H:%M").to_string();
    match template_type {
        FileTemplateType::NoteBullet => format!("- {} \u{2014} {}\n", stamp, flatten_linebreaks(text)),
        FileTemplateType::TaskCheckbox => {
            format!("- [ ] {} \u{2014} {}\n", stamp, flatten_linebreaks(text))
        }
        FileTemplateType::ShoppingCheckbox => {
            format!("- [ ] {} \u{2014} {}\n", stamp, flatten_linebreaks(text))
        }
        FileTemplateType::IdeaEntry => format!("## {}\n{}\n\n", stamp, text.trim()),
    }
}

fn append_entry(path: &Path, entry: &str) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(entry.as_bytes())?;
    Ok(())
}

fn flatten_linebreaks(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize_first_letter(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::{RecognitionCleanupOptions, RecognitionProfile};
    use chrono::TimeZone;
    use tempfile::tempdir;

    #[test]
    fn note_entry_uses_expected_format() {
        let timestamp = Local.with_ymd_and_hms(2026, 3, 16, 19, 42, 0).unwrap();
        let entry = format_entry(FileTemplateType::NoteBullet, "buy milk", timestamp);
        assert_eq!(entry, "- 2026-03-16 19:42 \u{2014} buy milk\n");
    }

    #[test]
    fn idea_entry_keeps_paragraphs() {
        let timestamp = Local.with_ymd_and_hms(2026, 3, 16, 19, 44, 0).unwrap();
        let entry = format_entry(FileTemplateType::IdeaEntry, "Line one\nLine two", timestamp);
        assert_eq!(entry, "## 2026-03-16 19:44\nLine one\nLine two\n\n");
    }

    #[test]
    fn append_entry_creates_directories() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("notes").join("Voice Notes.md");
        append_entry(&nested, "hello\n").unwrap();
        assert_eq!(std::fs::read_to_string(nested).unwrap(), "hello\n");
    }

    #[test]
    fn cleanup_respects_profile_options() {
        let profile = RecognitionProfile {
            id: "raw".to_string(),
            name: "Raw".to_string(),
            language_hint: None,
            instruction_prompt: String::new(),
            cleanup_options: RecognitionCleanupOptions {
                trim_whitespace: true,
                collapse_whitespace: false,
                preserve_newlines: true,
                capitalize_first_letter: false,
            },
        };
        assert_eq!(prepare_capture_text("  first\nsecond  ", &profile), "first\nsecond");
    }
}