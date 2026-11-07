//! The kit's enumerations for QML, each a mirror of a kit-core type, so QML says `Kit.Accept` or `Kit.BigScreen` (EmuSen_Kit.md §7.1).

use emusen_kit_core::input::{RepeatTiers, Role as WalkRole, UiButton, WalkForm};
use emusen_kit_core::sound::Cue as SoundCue;
use emusen_kit_core::tokens::{Form as TokenForm, Variant as TokenVariant};
use emusen_kit_core::world::{WorldRing, WorldShape};

#[cxx_qt::bridge(namespace = "Kit")]
pub mod ffi {
    #[qml_element]
    qnamespace!("Kit");

    /// What a pad asks of the interface: kit-core's `UiButton`, in its order.
    #[qenum]
    enum PadButton {
        Up,
        Down,
        Left,
        Right,
        Accept,
        Back,
        Menu,
        Options,
        PageUp,
        PageDown,
        First,
        Last,
        Guide,
        Search,
        Random,
        Screensaver,
    }

    /// The frame's three forms; the tokens read Card as the big screen's column.
    #[qenum]
    enum Form {
        Desktop,
        BigScreen,
        Card,
    }

    /// The frame's roles, and NoRole when nothing is focused.
    #[qenum]
    enum Role {
        Navigation,
        Content,
        Detail,
        Actions,
        Search,
        NoRole,
    }

    #[qenum]
    enum Variant {
        Dark,
        Light,
    }

    /// The pad's repeat tiers: kit-core's four measured sets.
    #[qenum]
    enum Tiers {
        Own,
        EsdeList,
        EsdeCarouselFast,
        EsdeCarousel,
    }

    /// The kind of body a world is drawn as: kit-core's `WorldShape`.
    #[qenum]
    enum BodyShape {
        Planet,
        Shards,
        Gem,
    }

    /// A planet's ring: kit-core's `WorldRing`.
    #[qenum]
    enum Ring {
        NoRing,
        Subtle,
        Big,
    }

    /// The interface's nine cues: kit-core's `Cue`.
    #[qenum]
    enum Cue {
        Move,
        Section,
        Choose,
        Back,
        Favourite,
        Launch,
        Saved,
        Loaded,
        Refused,
    }
}

pub use ffi::{BodyShape, Cue, Form, PadButton, Ring, Role, Tiers, Variant};

pub fn world_shape(shape: BodyShape) -> WorldShape {
    match shape {
        BodyShape::Shards => WorldShape::Shards,
        BodyShape::Gem => WorldShape::Gem,
        _ => WorldShape::Planet,
    }
}

pub fn body_shape(shape: WorldShape) -> BodyShape {
    match shape {
        WorldShape::Planet => BodyShape::Planet,
        WorldShape::Shards => BodyShape::Shards,
        WorldShape::Gem => BodyShape::Gem,
    }
}

pub fn world_ring(ring: Ring) -> WorldRing {
    match ring {
        Ring::Subtle => WorldRing::Subtle,
        Ring::Big => WorldRing::Big,
        _ => WorldRing::None,
    }
}

pub fn ring_of(ring: WorldRing) -> Ring {
    match ring {
        WorldRing::None => Ring::NoRing,
        WorldRing::Subtle => Ring::Subtle,
        WorldRing::Big => Ring::Big,
    }
}

pub fn sound_cue(cue: Cue) -> Option<SoundCue> {
    usize::try_from(cue.repr).ok().and_then(|i| SoundCue::ALL.get(i).copied())
}

pub fn cue_of(cue: SoundCue) -> Cue {
    Cue { repr: SoundCue::ALL.iter().position(|c| *c == cue).map_or(0, |i| i as i32) }
}

pub fn pad_button(button: UiButton) -> PadButton {
    PadButton { repr: button.index() as i32 }
}

pub fn ui_button(button: PadButton) -> Option<UiButton> {
    usize::try_from(button.repr).ok().and_then(|i| UiButton::ALL.get(i).copied())
}

pub fn walk_form(form: Form) -> WalkForm {
    match form {
        Form::Desktop => WalkForm::Desktop,
        Form::Card => WalkForm::Card,
        _ => WalkForm::BigScreen,
    }
}

pub fn form_of(form: WalkForm) -> Form {
    match form {
        WalkForm::Desktop => Form::Desktop,
        WalkForm::BigScreen => Form::BigScreen,
        WalkForm::Card => Form::Card,
    }
}

pub fn token_form(form: Form) -> TokenForm {
    match form {
        Form::Desktop => TokenForm::Desktop,
        _ => TokenForm::BigScreen,
    }
}

pub fn token_variant(variant: Variant) -> TokenVariant {
    match variant {
        Variant::Light => TokenVariant::Light,
        _ => TokenVariant::Dark,
    }
}

pub fn role_of(role: Option<WalkRole>) -> Role {
    match role {
        Some(WalkRole::Navigation) => Role::Navigation,
        Some(WalkRole::Content) => Role::Content,
        Some(WalkRole::Detail) => Role::Detail,
        Some(WalkRole::Actions) => Role::Actions,
        Some(WalkRole::Search) => Role::Search,
        None => Role::NoRole,
    }
}

pub fn walk_role(role: Role) -> Option<WalkRole> {
    match role {
        Role::Navigation => Some(WalkRole::Navigation),
        Role::Content => Some(WalkRole::Content),
        Role::Detail => Some(WalkRole::Detail),
        Role::Actions => Some(WalkRole::Actions),
        Role::Search => Some(WalkRole::Search),
        _ => None,
    }
}

pub fn repeat_tiers(tiers: Tiers) -> RepeatTiers {
    match tiers {
        Tiers::EsdeList => RepeatTiers::ESDE_LIST,
        Tiers::EsdeCarouselFast => RepeatTiers::ESDE_CAROUSEL_FAST,
        Tiers::EsdeCarousel => RepeatTiers::ESDE_CAROUSEL,
        _ => RepeatTiers::OWN,
    }
}
