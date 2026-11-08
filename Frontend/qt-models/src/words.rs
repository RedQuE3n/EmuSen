//! The words the screens show that no row carries, written once (EmuSen_Qt.md §1.2: QML holds no words), with the
//! counted phrases the rows' words are made of.

/// The wordmark: Moonrise's title and Home's (EmuSen_Interface.md §22, §24.3).
pub const TITLE: &str = "EMUSEN";

/// "1 game", "1,566 games": a count with its noun, grouped by thousands as the mockups write them.
pub fn counted(n: usize, one: &str, many: &str) -> String {
    format!("{} {}", grouped(n), if n == 1 { one } else { many })
}

/// A number grouped by thousands with commas.
pub fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Moonrise's caption: the count of planets in capitals.
pub fn splash_caption(planets: usize) -> String {
    counted(planets, "CONSOLE", "CONSOLES")
}
