//! A revision's box is also looked for under the plainer names, trailing tags dropped one at a time: the C#
//! `OnlineCoverTests`' case, held here since the C# failover was deleted (EmuSen_Backend.md §40.5).

use emusen_scrape::openemu::names;

#[test]
fn a_revisions_box_is_also_looked_for_under_the_plainer_names() {
    let cases: [(&str, &[&str]); 4] = [
        ("Wave Race 64 (USA) (Rev A)", &["Wave Race 64 (USA) (Rev A)", "Wave Race 64 (USA)"]),
        (
            "Legend of Zelda, The - Ocarina of Time (Europe) (En,Fr,De) (Rev A)",
            &[
                "Legend of Zelda, The - Ocarina of Time (Europe) (En,Fr,De) (Rev A)",
                "Legend of Zelda, The - Ocarina of Time (Europe) (En,Fr,De)",
                "Legend of Zelda, The - Ocarina of Time (Europe)",
            ],
        ),
        ("Tetris (World)", &["Tetris (World)"]),
        ("Untagged", &["Untagged"]),
    ];
    for (rom, expected) in cases {
        assert_eq!(names(rom), expected, "{rom}");
    }
}
