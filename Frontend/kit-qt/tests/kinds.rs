// EmuSen_Kit.md §7.1: every QML enumeration mirrors its kit-core type one to one, so a reordering on either side fails here.

use emusen_kit_core::input::{RepeatTiers, Role as WalkRole, UiButton, WalkForm};
use emusen_kit_core::tokens::{Form as TokenForm, Variant as TokenVariant};
use emusen_kit_qt::kinds::{self, Form, PadButton, Role, Tiers, Variant};

#[test]
fn kit_7_1_every_pad_button_names_its_ui_button() {
    let pairs = [
        (PadButton::Up, UiButton::Up),
        (PadButton::Down, UiButton::Down),
        (PadButton::Left, UiButton::Left),
        (PadButton::Right, UiButton::Right),
        (PadButton::Accept, UiButton::Accept),
        (PadButton::Back, UiButton::Back),
        (PadButton::Menu, UiButton::Menu),
        (PadButton::Options, UiButton::Options),
        (PadButton::PageUp, UiButton::PageUp),
        (PadButton::PageDown, UiButton::PageDown),
        (PadButton::First, UiButton::First),
        (PadButton::Last, UiButton::Last),
        (PadButton::Guide, UiButton::Guide),
        (PadButton::Search, UiButton::Search),
        (PadButton::Random, UiButton::Random),
        (PadButton::Screensaver, UiButton::Screensaver),
    ];
    assert_eq!(pairs.len(), UiButton::ALL.len());
    for (qml, core) in pairs {
        assert_eq!(kinds::ui_button(qml), Some(core), "{core:?}");
        assert_eq!(kinds::pad_button(core).repr, qml.repr, "{core:?}");
    }
    assert_eq!(kinds::ui_button(PadButton { repr: 16 }), None);
    assert_eq!(kinds::ui_button(PadButton { repr: -1 }), None);
}

#[test]
fn kit_7_1_forms_roles_variants_and_tiers_round_trip() {
    for (qml, core) in [(Form::Desktop, WalkForm::Desktop), (Form::BigScreen, WalkForm::BigScreen), (Form::Card, WalkForm::Card)] {
        assert_eq!(kinds::walk_form(qml), core);
        assert_eq!(kinds::form_of(core).repr, qml.repr);
    }
    assert_eq!(kinds::token_form(Form::Desktop), TokenForm::Desktop);
    assert_eq!(kinds::token_form(Form::BigScreen), TokenForm::BigScreen);
    assert_eq!(kinds::token_form(Form::Card), TokenForm::BigScreen, "a card reads the big screen's column");
    assert_eq!(kinds::token_variant(Variant::Dark), TokenVariant::Dark);
    assert_eq!(kinds::token_variant(Variant::Light), TokenVariant::Light);
    for (qml, core) in [
        (Role::Navigation, WalkRole::Navigation),
        (Role::Content, WalkRole::Content),
        (Role::Detail, WalkRole::Detail),
        (Role::Actions, WalkRole::Actions),
        (Role::Search, WalkRole::Search),
    ] {
        assert_eq!(kinds::walk_role(qml), Some(core));
        assert_eq!(kinds::role_of(Some(core)).repr, qml.repr);
    }
    assert_eq!(kinds::role_of(None).repr, Role::NoRole.repr);
    assert_eq!(kinds::walk_role(Role::NoRole), None);
    assert_eq!(kinds::repeat_tiers(Tiers::Own), RepeatTiers::OWN);
    assert_eq!(kinds::repeat_tiers(Tiers::EsdeList), RepeatTiers::ESDE_LIST);
    assert_eq!(kinds::repeat_tiers(Tiers::EsdeCarouselFast), RepeatTiers::ESDE_CAROUSEL_FAST);
    assert_eq!(kinds::repeat_tiers(Tiers::EsdeCarousel), RepeatTiers::ESDE_CAROUSEL);
}
