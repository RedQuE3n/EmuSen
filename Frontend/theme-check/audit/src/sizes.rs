//! The window sizes a run judges (man theme-check, --sizes): the gate's sizes, all of them or those a list names, in the gate's order.

use emusen_qt_test::harness::SIZES;

/// A size as the lines and recordings write it, `1280x800`.
pub fn written((w, h): (i32, i32)) -> String {
    format!("{w}x{h}")
}

/// The sizes a --sizes list names, `all` for every one, in the gate's order; the error names one the gate does not have.
pub fn chosen(list: &str) -> Result<Vec<(i32, i32)>, String> {
    let names: Vec<&str> = list.split(',').map(str::trim).filter(|n| !n.is_empty()).collect();
    if names.is_empty() {
        return Err("theme-audit: --sizes names no size".to_owned());
    }
    if let Some(unknown) = names.iter().find(|n| **n != "all" && !SIZES.iter().any(|s| written(*s) == **n)) {
        let known: Vec<String> = SIZES.iter().map(|s| written(*s)).collect();
        return Err(format!("theme-audit: {unknown} is not one of the gate's sizes ({})", known.join(", ")));
    }
    Ok(SIZES.into_iter().filter(|s| names.contains(&"all") || names.contains(&written(*s).as_str())).collect())
}

#[cfg(test)]
mod tests {
    use super::{chosen, SIZES};

    #[test]
    fn qt_3_7_a_sizes_list_is_read_in_the_gate_s_order_and_an_unknown_size_refused() {
        assert_eq!(chosen("all").unwrap(), SIZES.to_vec());
        assert_eq!(chosen("3840x2160, 1280x800").unwrap(), [(1280, 800), (3840, 2160)]);
        assert!(chosen("1366x768").unwrap_err().contains("1366x768 is not one of the gate's sizes"));
        assert!(chosen("1280x800,1280").unwrap_err().contains("1280 is not one"));
        assert!(chosen(",").is_err());
    }
}
