use sage::config;
use std::fs;
use std::path::PathBuf;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "sage-test-{}-{}-{}",
        std::process::id(),
        name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn sample_config() -> config::AppConfig {
    config::AppConfig {
        model: "E4B".to_string(),
        shell: "/bin/zsh".to_string(),
        show_hints: false,
        auto_copy: true,
    }
}

#[test]
fn resolves_known_models() {
    assert_eq!(
        config::get_model_config("E2B").unwrap().expected_filename,
        "gemma-4-E2B-it-Q4_K_M.gguf"
    );
    assert_eq!(
        config::get_model_config("E4B").unwrap().expected_filename,
        "gemma-4-E4B-it-Q4_K_M.gguf"
    );
}

#[test]
fn rejects_unknown_model() {
    assert!(config::get_model_config("UNKNOWN").is_err());
}

#[test]
fn cache_dir_lives_under_home() {
    let dir = config::get_cache_dir().unwrap();
    assert!(dir.ends_with(".cache/sage"));
    assert!(dir.exists());
}

#[test]
fn model_path_uses_expected_filename() {
    assert!(
        config::get_model_path(&config::GEMMA_E2B_CONFIG)
            .unwrap()
            .ends_with("gemma-4-E2B-it-Q4_K_M.gguf")
    );
    assert!(
        config::get_model_path(&config::GEMMA_E4B_CONFIG)
            .unwrap()
            .ends_with("gemma-4-E4B-it-Q4_K_M.gguf")
    );
}

#[test]
fn config_file_path_points_at_json() {
    assert!(config::config_file_path().unwrap().ends_with("config.json"));
}

#[test]
fn missing_config_is_none() {
    let dir = temp_dir("missing");
    let path = dir.join("config.json");
    assert!(config::load_config_from(&path).unwrap().is_none());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn save_and_load_roundtrip() {
    let dir = temp_dir("roundtrip");
    let path = dir.join("config.json");
    let cfg = sample_config();
    config::save_config_to(&cfg, &path).unwrap();

    let loaded = config::load_config_from(&path).unwrap().unwrap();
    assert_eq!(loaded.model, cfg.model);
    assert_eq!(loaded.shell, cfg.shell);
    assert_eq!(loaded.show_hints, cfg.show_hints);
    assert_eq!(loaded.auto_copy, cfg.auto_copy);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn invalid_json_is_none() {
    let dir = temp_dir("invalid");
    let path = dir.join("config.json");
    fs::write(&path, "{ not valid json").unwrap();
    assert!(config::load_config_from(&path).unwrap().is_none());
    fs::remove_dir_all(&dir).ok();
}
