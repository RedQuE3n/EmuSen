//! The settings folder when the host names none, in a process of its own since it sets the environment.

#[cfg(all(unix, not(target_os = "macos")))]
#[test]
fn the_settings_folder_is_xdg_config_home_where_absolute_and_else_under_home() {
    let home = emusen_galaxia::tree::user_home();
    // SAFETY: this test is the only one in its process, so nothing reads the environment while it changes.
    unsafe { std::env::set_var("XDG_CONFIG_HOME", "/srv/player/settings") };
    assert_eq!(emusen_galaxia::tree::application_data(), "/srv/player/settings");
    unsafe { std::env::set_var("XDG_CONFIG_HOME", "relative/settings") };
    assert_eq!(emusen_galaxia::tree::application_data(), format!("{home}/.config"));
    unsafe { std::env::remove_var("XDG_CONFIG_HOME") };
    assert_eq!(emusen_galaxia::tree::application_data(), format!("{home}/.config"));
}
