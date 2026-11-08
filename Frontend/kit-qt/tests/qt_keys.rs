// EmuSen_Kit.md §7.3: Qt's key, modifier and button numbers against kit-core's names, both ways.

use emusen_kit_core::input::{Key, PointerButton, UiButton};
use emusen_kit_qt::qt_keys::{kit_key, pointer_button, press_of, qt_key, KEY_BACKTAB, KEY_ENTER, SHIFT};

#[test]
fn kit_7_3_every_declinable_press_has_a_qt_key() {
    for button in UiButton::ALL {
        if let Some(key) = button.key() {
            let code = qt_key(key).expect("a Qt key for every key a press becomes");
            assert_eq!(kit_key(code, 0), (key, false), "{button:?}");
        }
    }
    assert_eq!(qt_key(Key::Return), Some(0x0100_0004));
    assert_eq!(qt_key(Key::Escape), Some(0x0100_0000));
    assert_eq!(qt_key(Key::Other), None);
}

#[test]
fn kit_7_3_shift_tab_enter_and_unknown_keys() {
    assert_eq!(kit_key(0x0100_0001, SHIFT), (Key::Tab, true));
    assert_eq!(kit_key(KEY_BACKTAB, 0), (Key::Tab, true), "Backtab is Shift+Tab however the modifiers arrived");
    assert_eq!(kit_key(KEY_ENTER, 0), (Key::Return, false));
    assert_eq!(kit_key(0x41, 0), (Key::Other, false));
}

#[test]
fn kit_7_3_the_mouses_buttons() {
    assert_eq!(pointer_button(1), Some(PointerButton::Primary));
    assert_eq!(pointer_button(2), Some(PointerButton::Secondary));
    assert_eq!(pointer_button(8), Some(PointerButton::Back));
    assert_eq!(pointer_button(4), None, "the middle button means nothing to the kit");
}

#[test]
fn kit_7_3_a_key_means_a_press_only_in_the_map() {
    assert_eq!(press_of(0x0100_0013, 0), Some(UiButton::Up));
    assert_eq!(press_of(KEY_ENTER, 0), Some(UiButton::Accept));
    assert_eq!(press_of(0x0100_0000, 0), Some(UiButton::Back));
    assert_eq!(press_of(0x0100_0001, 0), None, "Tab walks, it is no press");
    assert_eq!(press_of(KEY_BACKTAB, SHIFT), None);
    assert_eq!(press_of(0x0100_0020, SHIFT), None, "Shift alone");
    assert_eq!(press_of(0x41, 0), None, "a letter");
}
