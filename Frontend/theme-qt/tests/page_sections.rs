//! `Page.PageSection` is qt-models' `game_page::TABS`, name for name and place for place, so a theme's `pageSections` flags
//! are the model's `Tab::flag` (EmuSen_Qt.md §3.2, 1.13).

use emusen_qt_models::game_page::TABS;
use emusen_theme_qt::page::qobject::PageSection;

const TYPES: &str = include_str!("../../../Testing/qmltypes/EmuSen.Theme-1.13.qmltypes");

#[test]
fn qt_3_2_the_page_sections_are_the_models_tabs_in_their_order() {
    let at = TYPES.find("name: \"PageSection\"").expect("PageSection in the registry");
    let block = &TYPES[at..TYPES[at..].find('}').map_or(TYPES.len(), |e| at + e)];
    let values = &block[block.find('[').expect("its values") + 1..block.find(']').expect("its values' end")];
    let names: Vec<&str> = values.split(',').map(|v| v.trim().trim_matches('"')).filter(|v| !v.is_empty()).collect();
    let tabs: Vec<String> = TABS.iter().map(|t| format!("{t:?}")).collect();
    assert_eq!(names, tabs, "the registry's PageSection names are TABS' in order");
    let reprs = [PageSection::Overview, PageSection::States, PageSection::Media, PageSection::Details, PageSection::Options, PageSection::Cheats].map(|s| s.repr);
    for (i, (repr, tab)) in reprs.iter().zip(TABS).enumerate() {
        assert_eq!((*repr, 1 << repr), (i as i32, tab.flag()), "{tab:?}: its place, and the flag a theme writes as 1 << Page.{tab:?}");
    }
}
