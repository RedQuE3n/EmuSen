//! The printing on a pad's buttons: SDL's type first, the pad's name where SDL does not know (EmuSen_RustPlatform.md §48).

/// What a pad prints on its face buttons; the numbers are `EMUSEN_ENDYMION_FAMILY_*`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    /// Not known: a front end draws the buttons by position.
    Generic = 0,
    Xbox = 1,
    PlayStation = 2,
    Nintendo = 3,
}

/// `SDL_GamepadType`'s numbers (SDL_gamepad.h, SDL 3.4).
mod sdl_type {
    pub const XBOX360: i32 = 2;
    pub const XBOXONE: i32 = 3;
    pub const PS3: i32 = 4;
    pub const PS5: i32 = 6;
    pub const NINTENDO_SWITCH_PRO: i32 = 7;
    pub const GAMECUBE: i32 = 11;
}

const XBOX_NAMES: [&str; 6] = [
    "xbox",
    "x-box",
    "xinput",
    "legion go",
    "steam deck",
    "steam virtual gamepad",
];
const PLAYSTATION_NAMES: [&str; 7] = [
    "playstation",
    "dualshock",
    "dualsense",
    "ps3",
    "ps4",
    "ps5",
    "sony",
];
const NINTENDO_NAMES: [&str; 6] = [
    "nintendo",
    "switch",
    "joy-con",
    "joycon",
    "pro controller",
    "gamecube",
];

/// The family of a pad SDL reports as `kind`, named `name`.
pub fn of(kind: i32, name: &str) -> Family {
    match kind {
        sdl_type::XBOX360..=sdl_type::XBOXONE => Family::Xbox,
        sdl_type::PS3..=sdl_type::PS5 => Family::PlayStation,
        sdl_type::NINTENDO_SWITCH_PRO..=sdl_type::GAMECUBE => Family::Nintendo,
        _ => by_name(name),
    }
}

/// For a pad SDL reports as Standard or Unknown; the Legion Go S and a Steam Deck print an Xbox layout.
pub fn by_name(name: &str) -> Family {
    let name = name.to_lowercase();
    let any = |words: &[&str]| words.iter().any(|w| name.contains(w));
    if any(&XBOX_NAMES) {
        Family::Xbox
    } else if any(&PLAYSTATION_NAMES) {
        Family::PlayStation
    } else if any(&NINTENDO_NAMES) {
        Family::Nintendo
    } else {
        Family::Generic
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every type SDL 3.4 has, Unknown to GameCube, as the C# table classified them.
    #[test]
    fn every_sdl_type_gives_its_family_before_the_name() {
        let families = [
            Family::Generic,
            Family::Generic,
            Family::Xbox,
            Family::Xbox,
            Family::PlayStation,
            Family::PlayStation,
            Family::PlayStation,
            Family::Nintendo,
            Family::Nintendo,
            Family::Nintendo,
            Family::Nintendo,
            Family::Nintendo,
        ];
        for (kind, family) in families.into_iter().enumerate() {
            assert_eq!(of(kind as i32, "Some pad"), family, "type {kind}");
        }
        assert_eq!(
            of(5, "Xbox-style adapter"),
            Family::PlayStation,
            "SDL's type wins over a name that says otherwise"
        );
        assert_eq!(
            of(12, "Some pad"),
            Family::Generic,
            "a type newer than this table is read by its name"
        );
    }

    #[test]
    fn a_standard_or_unknown_pad_is_read_by_its_name() {
        assert_eq!(of(1, "Lenovo Legion Go S"), Family::Xbox);
        assert_eq!(of(0, "Steam Deck"), Family::Xbox);
        assert_eq!(of(0, "Steam Virtual Gamepad"), Family::Xbox);
        assert_eq!(
            of(0, "Sony DualSense Wireless Controller"),
            Family::PlayStation
        );
        assert_eq!(of(1, "Nintendo Switch Pro Controller"), Family::Nintendo);
        assert_eq!(of(1, "8BitDo Zero 2"), Family::Generic);
        assert_eq!(of(0, ""), Family::Generic);
    }

    #[test]
    fn the_numbers_are_the_header_s() {
        let numbers = [
            Family::Generic as u32,
            Family::Xbox as u32,
            Family::PlayStation as u32,
            Family::Nintendo as u32,
        ];
        assert_eq!(numbers, [0, 1, 2, 3]);
    }
}
