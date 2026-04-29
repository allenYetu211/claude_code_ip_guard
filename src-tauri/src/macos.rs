//! macOS appearance helpers.

#[cfg(target_os = "macos")]
pub fn is_dark_mode() -> bool {
    use objc2_foundation::{NSString, NSUserDefaults};
    let defaults = NSUserDefaults::standardUserDefaults();
    let key = NSString::from_str("AppleInterfaceStyle");
    let val = defaults.stringForKey(&key);
    match val {
        Some(s) => s.to_string().eq_ignore_ascii_case("Dark"),
        None => false,
    }
}

#[cfg(not(target_os = "macos"))]
pub fn is_dark_mode() -> bool { false }
