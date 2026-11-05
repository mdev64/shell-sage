use sage::setup::{detect_ram_gb, detect_shell};

#[test]
fn ram_detection_returns_positive_value() {
    // Falls back to 8 GB when `sysctl` is unavailable, so it is never 0.
    assert!(detect_ram_gb() > 0);
}

#[test]
fn shell_detection_returns_absolute_path() {
    let shell = detect_shell();
    assert!(!shell.is_empty());
    assert!(shell.starts_with('/'));
}

#[test]
fn recommends_larger_model_on_16gb_or_more() {
    use sage::setup::recommended_model;
    assert_eq!(recommended_model(8).name, "E2B");
    assert_eq!(recommended_model(15).name, "E2B");
    assert_eq!(recommended_model(16).name, "E4B");
    assert_eq!(recommended_model(64).name, "E4B");
}
