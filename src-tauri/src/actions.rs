#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::apple_intelligence;
use crate::audio_feedback::{play_feedback_sound, play_feedback_sound_blocking, SoundType};
use crate::audio_toolkit::is_microphone_access_denied;
use crate::capture;
use crate::managers::audio::AudioRecordingManager;
use crate::managers::history::HistoryManager;
use crate::managers::transcription::TranscriptionManager;
use crate::settings::{
    get_settings, AppSettings, CaptureAction as ConfiguredCaptureAction, RecognitionProfile,
    APPLE_INTELLIGENCE_PROVIDER_ID,
};
use crate::shortcut;
use crate::tray::{change_tray_icon, TrayIconState};
use crate::utils::{
    self, show_processing_overlay, show_recording_overlay, show_transcribing_overlay,
};
use crate::TranscriptionCoordinator;
use ferrous_opencc::{config::BuiltinConfig, OpenCC};
use log::{debug, error, warn};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tauri::Manager;
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct RecordingErrorEvent {
    error_type: String,
    detail: Option<String>,
}

struct FinishGuard(AppHandle);
impl Drop for FinishGuard {
    fn drop(&mut self) {
        if let Some(c) = self.0.try_state::<TranscriptionCoordinator>() {
            c.notify_processing_finished();
        }
    }
}

pub trait ShortcutAction: Send + Sync {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str);
}

struct TranscribeAction {
    post_process: bool,
}

const TRANSCRIPTION_FIELD: &str = "transcription";

fn strip_invisible_chars(s: &str) -> String {
    s.replace(['\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}'], "")
}

fn build_system_prompt(prompt_template: &str) -> String {
    prompt_template.replace("${output}", "").trim().to_string()
}

fn resolve_post_process_prompt(
    settings: &AppSettings,
    prompt_override: Option<&str>,
) -> Option<String> {
    if let Some(prompt) = prompt_override {
        let trimmed = prompt.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let selected_prompt_id = settings.post_process_selected_prompt_id.as_ref()?;
    let prompt = settings
        .post_process_prompts
        .iter()
        .find(|prompt| &prompt.id == selected_prompt_id)?
        .prompt
        .trim()
        .to_string();

    if prompt.is_empty() {
        None
    } else {
        Some(prompt)
    }
}

async fn post_process_transcription(
    settings: &AppSettings,
    transcription: &str,
    prompt_override: Option<&str>,
) -> Option<String> {
    let provider = match settings.active_post_process_provider().cloned() {
        Some(provider) => provider,
        None => {
            debug!("Post-processing enabled but no provider is selected");
            return None;
        }
    };

    let model = settings
        .post_process_models
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();

    if model.trim().is_empty() {
        debug!(
            "Post-processing skipped because provider '{}' has no model configured",
            provider.id
        );
        return None;
    }

    let prompt = match resolve_post_process_prompt(settings, prompt_override) {
        Some(prompt) => prompt,
        None => {
            debug!("Post-processing skipped because no prompt is selected");
            return None;
        }
    };

    debug!(
        "Starting LLM post-processing with provider '{}' (model: {})",
        provider.id, model
    );

    let api_key = settings
        .post_process_api_keys
        .get(&provider.id)
        .cloned()
        .unwrap_or_default();

    if provider.supports_structured_output {
        debug!("Using structured outputs for provider '{}'", provider.id);

        let system_prompt = build_system_prompt(&prompt);
        let user_content = transcription.to_string();

        if provider.id == APPLE_INTELLIGENCE_PROVIDER_ID {
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            {
                if !apple_intelligence::check_apple_intelligence_availability() {
                    debug!(
                        "Apple Intelligence selected but not currently available on this device"
                    );
                    return None;
                }

                let token_limit = model.trim().parse::<i32>().unwrap_or(0);
                return match apple_intelligence::process_text_with_system_prompt(
                    &system_prompt,
                    &user_content,
                    token_limit,
                ) {
                    Ok(result) => {
                        if result.trim().is_empty() {
                            debug!("Apple Intelligence returned an empty response");
                            None
                        } else {
                            let result = strip_invisible_chars(&result);
                            debug!(
                                "Apple Intelligence post-processing succeeded. Output length: {} chars",
                                result.len()
                            );
                            Some(result)
                        }
                    }
                    Err(err) => {
                        error!("Apple Intelligence post-processing failed: {}", err);
                        None
                    }
                };
            }

            #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
            {
                debug!("Apple Intelligence provider selected on unsupported platform");
                return None;
            }
        }

        let json_schema = serde_json::json!({
            "type": "object",
            "properties": {
                (TRANSCRIPTION_FIELD): {
                    "type": "string",
                    "description": "The cleaned and processed transcription text"
                }
            },
            "required": [TRANSCRIPTION_FIELD],
            "additionalProperties": false
        });

        match crate::llm_client::send_chat_completion_with_schema(
            &provider,
            api_key.clone(),
            &model,
            user_content,
            Some(system_prompt),
            Some(json_schema),
        )
        .await
        {
            Ok(Some(content)) => {
                return match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(json) => {
                        if let Some(transcription_value) =
                            json.get(TRANSCRIPTION_FIELD).and_then(|t| t.as_str())
                        {
                            let result = strip_invisible_chars(transcription_value);
                            debug!(
                                "Structured output post-processing succeeded for provider '{}'. Output length: {} chars",
                                provider.id,
                                result.len()
                            );
                            Some(result)
                        } else {
                            error!("Structured output response missing 'transcription' field");
                            Some(strip_invisible_chars(&content))
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to parse structured output JSON: {}. Returning raw content.",
                            e
                        );
                        Some(strip_invisible_chars(&content))
                    }
                };
            }
            Ok(None) => {
                error!("LLM API response has no content");
                return None;
            }
            Err(e) => {
                warn!(
                    "Structured output failed for provider '{}': {}. Falling back to legacy mode.",
                    provider.id, e
                );
            }
        }
    }

    let processed_prompt = prompt.replace("${output}", transcription);
    debug!("Processed prompt length: {} chars", processed_prompt.len());

    match crate::llm_client::send_chat_completion(&provider, api_key, &model, processed_prompt).await
    {
        Ok(Some(content)) => {
            let content = strip_invisible_chars(&content);
            debug!(
                "LLM post-processing succeeded for provider '{}'. Output length: {} chars",
                provider.id,
                content.len()
            );
            Some(content)
        }
        Ok(None) => {
            error!("LLM API response has no content");
            None
        }
        Err(e) => {
            error!(
                "LLM post-processing failed for provider '{}': {}. Falling back to original transcription.",
                provider.id,
                e
            );
            None
        }
    }
}

async fn maybe_convert_chinese_variant(
    language_hint: Option<&str>,
    transcription: &str,
) -> Option<String> {
    let Some(language) = language_hint else {
        return None;
    };

    let is_simplified = language == "zh-Hans";
    let is_traditional = language == "zh-Hant";

    if !is_simplified && !is_traditional {
        return None;
    }

    let config = if is_simplified {
        BuiltinConfig::Tw2sp
    } else {
        BuiltinConfig::S2twp
    };

    match OpenCC::from_config(config) {
        Ok(converter) => Some(converter.convert(transcription)),
        Err(e) => {
            error!("Failed to initialize OpenCC converter: {}", e);
            None
        }
    }
}

fn finish_capture_ui(app: &AppHandle) {
    utils::hide_recording_overlay(app);
    change_tray_icon(app, TrayIconState::Idle);
}

fn start_recording_session(app: &AppHandle, binding_id: &str) {
    let start_time = Instant::now();
    debug!("Starting recording session for binding: {}", binding_id);

    let tm = app.state::<Arc<TranscriptionManager>>();
    tm.initiate_model_load();

    change_tray_icon(app, TrayIconState::Recording);
    show_recording_overlay(app);

    let rm = app.state::<Arc<AudioRecordingManager>>();
    let settings = get_settings(app);
    let is_always_on = settings.always_on_microphone;

    let mut recording_error: Option<String> = None;
    if is_always_on {
        let rm_clone = Arc::clone(&rm);
        let app_clone = app.clone();
        std::thread::spawn(move || {
            play_feedback_sound_blocking(&app_clone, SoundType::Start);
            rm_clone.apply_mute();
        });

        if let Err(e) = rm.try_start_recording(binding_id) {
            recording_error = Some(e);
        }
    } else {
        match rm.try_start_recording(binding_id) {
            Ok(()) => {
                let app_clone = app.clone();
                let rm_clone = Arc::clone(&rm);
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    play_feedback_sound_blocking(&app_clone, SoundType::Start);
                    rm_clone.apply_mute();
                });
            }
            Err(e) => recording_error = Some(e),
        }
    }

    if recording_error.is_none() {
        shortcut::register_cancel_shortcut(app);
    } else {
        utils::hide_recording_overlay(app);
        change_tray_icon(app, TrayIconState::Idle);
        if let Some(err) = recording_error {
            let error_type = if is_microphone_access_denied(&err) {
                "microphone_permission_denied"
            } else {
                "unknown"
            };
            let _ = app.emit(
                "recording-error",
                RecordingErrorEvent {
                    error_type: error_type.to_string(),
                    detail: Some(err),
                },
            );
        }
    }

    debug!("Recording session started in {:?}", start_time.elapsed());
}

async fn apply_capture_profile(
    settings: &AppSettings,
    profile: &RecognitionProfile,
    transcription: &str,
) -> (String, Option<String>, Option<String>) {
    let converted = maybe_convert_chinese_variant(profile.language_hint.as_deref(), transcription)
        .await
        .unwrap_or_else(|| transcription.to_string());
    let mut final_text = capture::prepare_capture_text(&converted, profile);
    let mut post_processed_text = None;
    let mut prompt_used = None;
    let should_try_profile_post_process = !profile.instruction_prompt.trim().is_empty();

    if should_try_profile_post_process {
        if let Some(processed) =
            post_process_transcription(settings, &final_text, Some(&profile.instruction_prompt)).await
        {
            final_text = capture::prepare_capture_text(&processed, profile);
            post_processed_text = Some(final_text.clone());
            prompt_used = Some(profile.instruction_prompt.clone());
        } else if final_text != transcription {
            post_processed_text = Some(final_text.clone());
        }
    } else if final_text != transcription {
        post_processed_text = Some(final_text.clone());
    }

    (final_text, post_processed_text, prompt_used)
}

async fn run_capture_pipeline(app: AppHandle, binding_id: String, capture_action: ConfiguredCaptureAction) {
    let _guard = FinishGuard(app.clone());
    let rm = Arc::clone(&app.state::<Arc<AudioRecordingManager>>());
    let tm = Arc::clone(&app.state::<Arc<TranscriptionManager>>());
    let hm = Arc::clone(&app.state::<Arc<HistoryManager>>());

    let stop_recording_time = Instant::now();
    let Some(samples) = rm.stop_recording(&binding_id) else {
        debug!("No samples retrieved from recording stop");
        finish_capture_ui(&app);
        return;
    };
    debug!(
        "Recording stopped and samples retrieved in {:?}, sample count: {}",
        stop_recording_time.elapsed(),
        samples.len()
    );

    let settings = get_settings(&app);
    let action = settings
        .capture_action_for_binding(&binding_id)
        .cloned()
        .unwrap_or(capture_action);
    let profile = settings
        .recognition_profile(&action.profile_id)
        .cloned()
        .or_else(|| settings.recognition_profiles.first().cloned());
    let Some(profile) = profile else {
        capture::emit_capture_notification(
            &app,
            "error",
            action.name.clone(),
            Some("No recognition profile configured".to_string()),
        );
        finish_capture_ui(&app);
        return;
    };

    let samples_for_history = samples.clone();
    let transcription_time = Instant::now();
    match tm.transcribe_with_language_hint(samples, profile.language_hint.clone()) {
        Ok(transcription) => {
            debug!(
                "Capture transcription completed in {:?}: '{}'",
                transcription_time.elapsed(),
                transcription
            );
            if transcription.trim().is_empty() {
                finish_capture_ui(&app);
                return;
            }

            let (final_text, post_processed_text, prompt_used) =
                apply_capture_profile(&settings, &profile, &transcription).await;

            let hm_clone = Arc::clone(&hm);
            let transcription_for_history = transcription.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = hm_clone
                    .save_transcription(
                        samples_for_history,
                        transcription_for_history,
                        post_processed_text,
                        prompt_used,
                    )
                    .await
                {
                    error!("Failed to save transcription to history: {}", e);
                }
            });

            let route_result = match action.output_target.r#type {
                crate::settings::OutputTargetType::Paste => {
                    let app_clone = app.clone();
                    let settings_clone = settings.clone();
                    let action_clone = action.clone();
                    let final_text_clone = final_text.clone();
                    app.run_on_main_thread(move || {
                        if let Err(e) = capture::route_capture_result(
                            &app_clone,
                            &settings_clone,
                            &action_clone,
                            &final_text_clone,
                        ) {
                            capture::emit_capture_notification(
                                &app_clone,
                                "error",
                                action_clone.name.clone(),
                                Some(e.clone()),
                            );
                            error!("Failed to route capture result: {}", e);
                        }
                        finish_capture_ui(&app_clone);
                    })
                    .map_err(|e| e.to_string())
                }
                crate::settings::OutputTargetType::AppendFile => {
                    let result = capture::route_capture_result(&app, &settings, &action, &final_text);
                    if let Err(e) = &result {
                        capture::emit_capture_notification(
                            &app,
                            "error",
                            action.name.clone(),
                            Some(e.clone()),
                        );
                        error!("Failed to route capture result: {}", e);
                    }
                    finish_capture_ui(&app);
                    result
                }
            };

            if let Err(e) = route_result {
                if matches!(action.output_target.r#type, crate::settings::OutputTargetType::Paste) {
                    capture::emit_capture_notification(
                        &app,
                        "error",
                        action.name.clone(),
                        Some(e.clone()),
                    );
                    finish_capture_ui(&app);
                }
                error!("Capture action '{}' failed: {}", action.id, e);
            }
        }
        Err(err) => {
            debug!("Capture transcription error: {}", err);
            capture::emit_capture_notification(
                &app,
                "error",
                action.name.clone(),
                Some(err.to_string()),
            );
            finish_capture_ui(&app);
        }
    }
}

async fn run_legacy_post_process_pipeline(app: AppHandle, binding_id: String) {
    let _guard = FinishGuard(app.clone());
    let rm = Arc::clone(&app.state::<Arc<AudioRecordingManager>>());
    let tm = Arc::clone(&app.state::<Arc<TranscriptionManager>>());
    let hm = Arc::clone(&app.state::<Arc<HistoryManager>>());

    let stop_recording_time = Instant::now();
    if let Some(samples) = rm.stop_recording(&binding_id) {
        debug!(
            "Recording stopped and samples retrieved in {:?}, sample count: {}",
            stop_recording_time.elapsed(),
            samples.len()
        );

        let transcription_time = Instant::now();
        let samples_clone = samples.clone();
        match tm.transcribe(samples) {
            Ok(transcription) => {
                debug!(
                    "Transcription completed in {:?}: '{}'",
                    transcription_time.elapsed(),
                    transcription
                );
                if !transcription.is_empty() {
                    let settings = get_settings(&app);
                    let mut final_text = transcription.clone();
                    let mut post_processed_text: Option<String> = None;
                    let mut post_process_prompt: Option<String> = None;

                    if let Some(converted_text) =
                        maybe_convert_chinese_variant(Some(settings.selected_language.as_str()), &transcription)
                            .await
                    {
                        final_text = converted_text;
                    }

                    show_processing_overlay(&app);
                    let processed = post_process_transcription(&settings, &final_text, None).await;
                    if let Some(processed_text) = processed {
                        post_processed_text = Some(processed_text.clone());
                        final_text = processed_text;

                        if let Some(prompt_id) = &settings.post_process_selected_prompt_id {
                            if let Some(prompt) = settings
                                .post_process_prompts
                                .iter()
                                .find(|p| &p.id == prompt_id)
                            {
                                post_process_prompt = Some(prompt.prompt.clone());
                            }
                        }
                    } else if final_text != transcription {
                        post_processed_text = Some(final_text.clone());
                    }

                    let hm_clone = Arc::clone(&hm);
                    let transcription_for_history = transcription.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = hm_clone
                            .save_transcription(
                                samples_clone,
                                transcription_for_history,
                                post_processed_text,
                                post_process_prompt,
                            )
                            .await
                        {
                            error!("Failed to save transcription to history: {}", e);
                        }
                    });

                    let app_clone = app.clone();
                    let paste_time = Instant::now();
                    app.run_on_main_thread(move || {
                        match utils::paste(final_text, app_clone.clone()) {
                            Ok(()) => debug!("Text pasted successfully in {:?}", paste_time.elapsed()),
                            Err(e) => error!("Failed to paste transcription: {}", e),
                        }
                        finish_capture_ui(&app_clone);
                    })
                    .unwrap_or_else(|e| {
                        error!("Failed to run paste on main thread: {:?}", e);
                        finish_capture_ui(&app);
                    });
                } else {
                    finish_capture_ui(&app);
                }
            }
            Err(err) => {
                debug!("Global Shortcut Transcription error: {}", err);
                finish_capture_ui(&app);
            }
        }
    } else {
        debug!("No samples retrieved from recording stop");
        finish_capture_ui(&app);
    }
}

pub fn is_coordinated_binding(app: &AppHandle, binding_id: &str) -> bool {
    if binding_id == "transcribe_with_post_process" {
        return true;
    }
    get_settings(app).capture_action_for_binding(binding_id).is_some()
}

pub fn start_coordinated_action(app: &AppHandle, binding_id: &str, shortcut_str: &str) {
    if get_settings(app).capture_action_for_binding(binding_id).is_some() {
        start_recording_session(app, binding_id);
        return;
    }

    if let Some(action) = ACTION_MAP.get(binding_id) {
        action.start(app, binding_id, shortcut_str);
    } else {
        warn!("No action configured for '{}'", binding_id);
    }
}

pub fn stop_coordinated_action(app: &AppHandle, binding_id: &str, shortcut_str: &str) {
    if let Some(capture_action) = get_settings(app).capture_action_for_binding(binding_id).cloned() {
        shortcut::unregister_cancel_shortcut(app);
        let rm = Arc::clone(&app.state::<Arc<AudioRecordingManager>>());
        change_tray_icon(app, TrayIconState::Transcribing);
        show_transcribing_overlay(app);
        rm.remove_mute();
        play_feedback_sound(app, SoundType::Stop);
        tauri::async_runtime::spawn(run_capture_pipeline(
            app.clone(),
            binding_id.to_string(),
            capture_action,
        ));
        return;
    }

    if let Some(action) = ACTION_MAP.get(binding_id) {
        action.stop(app, binding_id, shortcut_str);
    } else {
        warn!("No action configured for '{}'", binding_id);
    }
}

impl ShortcutAction for TranscribeAction {
    fn start(&self, app: &AppHandle, binding_id: &str, _shortcut_str: &str) {
        start_recording_session(app, binding_id);
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, _shortcut_str: &str) {
        shortcut::unregister_cancel_shortcut(app);

        let rm = Arc::clone(&app.state::<Arc<AudioRecordingManager>>());
        change_tray_icon(app, TrayIconState::Transcribing);
        show_transcribing_overlay(app);
        rm.remove_mute();
        play_feedback_sound(app, SoundType::Stop);

        if self.post_process {
            tauri::async_runtime::spawn(run_legacy_post_process_pipeline(
                app.clone(),
                binding_id.to_string(),
            ));
        }
    }
}

struct CancelAction;

impl ShortcutAction for CancelAction {
    fn start(&self, app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {
        utils::cancel_current_operation(app);
    }

    fn stop(&self, _app: &AppHandle, _binding_id: &str, _shortcut_str: &str) {}
}

struct TestAction;

impl ShortcutAction for TestAction {
    fn start(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Started - {} (App: {})",
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }

    fn stop(&self, app: &AppHandle, binding_id: &str, shortcut_str: &str) {
        log::info!(
            "Shortcut ID '{}': Stopped - {} (App: {})",
            binding_id,
            shortcut_str,
            app.package_info().name
        );
    }
}

pub static ACTION_MAP: Lazy<HashMap<String, Arc<dyn ShortcutAction>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "transcribe_with_post_process".to_string(),
        Arc::new(TranscribeAction { post_process: true }) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "cancel".to_string(),
        Arc::new(CancelAction) as Arc<dyn ShortcutAction>,
    );
    map.insert(
        "test".to_string(),
        Arc::new(TestAction) as Arc<dyn ShortcutAction>,
    );
    map
});
