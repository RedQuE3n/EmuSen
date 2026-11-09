// The figures chart held to Constellation's Figures canvas: its packer's every node, walk and figure at both widths (EmuSen_Kit.md §7.31).

use emusen_kit_core::figures::{FigureGeometry, FigureLayout, Node, Point, figure_layout};

const G: FigureGeometry = FigureGeometry::CHART;

fn node(name: &str) -> Option<Node> {
    match name.split_at(1) {
        ("-", _) => None,
        ("p", n) => Some(Node::Pointer(n.parse().unwrap())),
        (_, n) => Some(Node::Item(n.parse().unwrap())),
    }
}

/// Lays out the fixture's catalogue (each group's count, in order) and compares every number the canvas gave.
fn held(fixture: &str, width: f64) -> FigureLayout {
    let mut keys: Vec<usize> = Vec::new();
    let mut expected: Vec<Vec<&str>> = Vec::new();
    for line in fixture.lines().filter(|l| !l.starts_with('#')) {
        let cells: Vec<&str> = line.split('\t').collect();
        if cells[0] == "groups" {
            for (g, n) in cells.get(1).map_or("", |c| *c).split_whitespace().enumerate() {
                keys.extend(std::iter::repeat_n(g, n.parse().unwrap()));
            }
        } else {
            expected.push(cells);
        }
    }
    let l = figure_layout(width, &keys, &G);
    let near = |a: f64, b: &str| (a - b.parse::<f64>().unwrap()).abs() < 0.01;
    let mut figures = l.figures.iter();
    for e in &expected {
        match e[0] {
            "chart" => {
                assert_eq!(
                    (l.compact as u8, l.lines, l.scrolls as u8),
                    (e[1].parse().unwrap(), e[2].parse().unwrap(), e[4].parse().unwrap()),
                    "{width}: tier, lines, scrolling"
                );
                assert!(near(l.height, e[3]), "{width}: height {}", l.height);
            }
            "figure" => {
                let f = figures.next().expect("as many figures as the canvas");
                assert!(near(f.x, e[1]) && near(f.y, e[2]) && near(f.width, e[3]), "{width}: figure {f:?} against {e:?}");
                assert_eq!(f.per_line.to_string(), e[4], "{width}: figure {f:?}");
            }
            name => {
                let p: Point = match node(name).unwrap() {
                    Node::Item(i) => l.items[i],
                    Node::Pointer(i) => l.pointers[i],
                };
                assert!(near(p.x, e[1]) && near(p.y, e[2]), "{width}: {name} at ({}, {}) against {e:?}", p.x, p.y);
                assert_eq!((p.raised as u8).to_string(), e[3], "{width}: {name} raised");
                assert_eq!(p.line.to_string(), e[4], "{width}: {name}'s line");
                assert_eq!((p.left, p.right, p.up, p.down), (node(e[5]), node(e[6]), node(e[7]), node(e[8])), "{width}: {name}'s walk");
            }
        }
    }
    assert!(figures.next().is_none(), "{width}: no figure the canvas lacks");
    l
}

#[test]
fn eleven_consoles_of_three_makers_in_the_full_tier() {
    let l = held(include_str!("figures/default-1280.tsv"), 1280.0);
    assert_eq!(l.prior[1], Some(0), "a figure's line runs from each body to the one before it");
    assert_eq!(l.prior[5], None, "a maker's first body starts its figure");
    held(include_str!("figures/default-1422.2.tsv"), 1422.2);
}

#[test]
fn one_console_beside_the_pointers_and_none_at_all() {
    held(include_str!("figures/one-console-1280.tsv"), 1280.0);
    held(include_str!("figures/one-console-1422.2.tsv"), 1422.2);
    let empty = held(include_str!("figures/empty-1280.tsv"), 1280.0);
    assert!(empty.items.is_empty() && empty.figures.is_empty());
    held(include_str!("figures/empty-1422.2.tsv"), 1422.2);
}

#[test]
fn thirty_consoles_of_twelve_makers_take_the_compact_tier_and_scroll() {
    held(include_str!("figures/n30-1280.tsv"), 1280.0);
    held(include_str!("figures/n30-1422.2.tsv"), 1422.2);
}

#[test]
fn a_hundred_and_twenty_consoles_one_figure_a_row() {
    let l = held(include_str!("figures/n120-1280.tsv"), 1280.0);
    assert_eq!(l.figures[0].width, 1078.0);
    held(include_str!("figures/n120-1422.2.tsv"), 1422.2);
}

#[test]
fn a_maker_of_forty_wraps_into_lines_chained_end_to_end() {
    let l = held(include_str!("figures/maker40-1280.tsv"), 1280.0);
    assert_eq!((l.figures[0].per_line, l.items[39].line), (10, 4), "four lines of ten");
    assert_eq!(l.prior[10], Some(9), "a wrapped line goes on from the last body of the line above");
    held(include_str!("figures/maker40-1422.2.tsv"), 1422.2);
}

#[test]
fn an_interleaved_table_is_grouped_by_first_appearance() {
    let l = figure_layout(1280.0, &["b", "a", "b", "a", "b"], &G);
    assert_eq!(l.group_of, vec![0, 1, 0, 1, 0]);
    assert_eq!((l.figures[0].count, l.figures[0].first, l.figures[1].count, l.figures[1].first), (3, 0, 2, 1));
    assert_eq!((l.prior[2], l.prior[4], l.prior[3]), (Some(0), Some(2), Some(1)));
    assert!(l.items[0].x < l.items[2].x && l.items[2].x < l.items[4].x, "a figure's bodies follow its group's order");
}
