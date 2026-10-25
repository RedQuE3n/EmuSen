//! VU1's elementary function unit, each function as the VU User's Manual's §7.2 gives its "nano code": a sequence of
//! the FMAC's and the divider's own operations on its constants, computed here on `crate::float` in the code's order
//! with its operands in the code's places (the multiplier recodes the second, Sapphire_Disputes.md D-26). The
//! constants are the pages' of §4.3; the flags are untouched (every page's "-"). Sapphire_Native.md §23 records how
//! the console's `vu/lower/efu` text grades the reading.

use crate::float::{add, div, mul, rsqrt, sqrt, sub};

const ONE: u32 = 0x3F80_0000;
/// ESIN's S1 to S5.
const S: [u32; 5] = [0x3F80_0000, 0xBE2A_AAA4, 0x3C08_873E, 0xB94F_B21F, 0x362E_9C14];
/// EATAN's T1 to T8 and π/4.
const T: [u32; 8] = [0x3F7F_FFF5, 0xBEAA_A61C, 0x3E4C_40A6, 0xBE0E_6C63, 0x3DC5_77DF, 0xBD65_01C4, 0x3CB3_1652, 0xBB84_D7E7];
const PI_4: u32 = 0x3F49_0FDB;
/// EEXP's E1 to E6.
const E: [u32; 6] = [0x3E7F_FFA8, 0x3D00_07F4, 0x3B29_D3FF, 0x3933_E553, 0x36B6_3510, 0x3539_61AC];

fn m(a: u32, b: u32) -> u32 {
    mul(a, b).bits
}
/// MADD/MADDA: ACC + a × b, the product clamped and then added as ADD adds.
fn ma(acc: u32, a: u32, b: u32) -> u32 {
    add(acc, mul(a, b).bits).bits
}

/// x² + y² + z²: MULA, MADDA, MADD.
pub fn esadd(v: [u32; 4]) -> u32 {
    let acc = m(v[0], v[0]);
    let acc = ma(acc, v[1], v[1]);
    ma(acc, v[2], v[2])
}
/// 1 / ESADD's sum: DIV p, 1, p.
pub fn ersadd(v: [u32; 4]) -> u32 {
    div(ONE, esadd(v)).value.bits
}
/// √ of ESADD's sum: SQRT p, 1, p.
pub fn eleng(v: [u32; 4]) -> u32 {
    sqrt(esadd(v)).bits
}
/// 1 / √ of ESADD's sum: RSQRT p, 1, p.
pub fn erleng(v: [u32; 4]) -> u32 {
    rsqrt(ONE, esadd(v)).value.bits
}
/// x + y + z + w: each term multiplied by 1 as the second operand, MULA, MADDA, MADDA, MADD.
pub fn esum(v: [u32; 4]) -> u32 {
    let acc = m(v[0], ONE);
    let acc = ma(acc, v[1], ONE);
    let acc = ma(acc, v[2], ONE);
    ma(acc, v[3], ONE)
}
pub fn ercpr(x: u32) -> u32 {
    div(ONE, x).value.bits
}
pub fn esqrt(x: u32) -> u32 {
    sqrt(x).bits
}
pub fn ersqrt(x: u32) -> u32 {
    rsqrt(ONE, x).value.bits
}

/// sin x by the odd series to x⁹.
pub fn esin(x: u32) -> u32 {
    let x2 = m(x, x);
    let acc = m(x, S[0]);
    let x3 = m(x2, x);
    let x5 = m(x3, x2);
    let acc = ma(acc, x3, S[1]);
    let x7 = m(x5, x2);
    let acc = ma(acc, x5, S[2]);
    let x9 = m(x7, x2);
    let acc = ma(acc, x7, S[3]);
    ma(acc, x9, S[4])
}

/// The arctangent series after the quotient t = `num` / `den`: π/4 added before the last term.
fn arctan(num: u32, den: u32) -> u32 {
    let t = div(num, den).value.bits;
    let t2 = m(t, t);
    let mut acc = m(T[0], t);
    let mut odd = m(t2, t);
    for term in &T[1..7] {
        let next = m(odd, t2);
        acc = ma(acc, odd, *term);
        odd = next;
    }
    let acc = ma(acc, ONE, PI_4);
    ma(acc, odd, T[7])
}

/// arctan x: t = (x − 1) / (x + 1).
pub fn eatan(x: u32) -> u32 {
    arctan(sub(x, ONE).bits, add(x, ONE).bits)
}
/// arctan y/x: t = (y − x) / (y + x).
pub fn eatanxy(v: [u32; 4]) -> u32 {
    arctan(sub(v[1], v[0]).bits, add(v[1], v[0]).bits)
}
/// arctan z/x: t = (z − x) / (z + x).
pub fn eatanxz(v: [u32; 4]) -> u32 {
    arctan(sub(v[2], v[0]).bits, add(v[2], v[0]).bits)
}

/// e^−x: 1 / (1 + E1 x + … + E6 x⁶)⁴, the fourth power by two squarings.
pub fn eexp(x: u32) -> u32 {
    let mut pow = m(x, x);
    let mut acc = m(x, E[0]);
    for e in &E[1..5] {
        let next = m(pow, x);
        acc = ma(acc, pow, *e);
        pow = next;
    }
    let acc = ma(acc, ONE, ONE);
    let p = ma(acc, pow, E[5]);
    let p = m(p, p);
    let p = m(p, p);
    div(ONE, p).value.bits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::float;

    /// The nano code's own values: arctan 1 is π/4 as the multiplier forms 1 × π/4, and sin of π/2 is 1.
    #[test]
    fn the_series_land_where_the_code_puts_them() {
        assert_eq!(eatan(ONE), 0x3F49_0FDA, "1 × π/4 with π/4 recoded loses its last bit (D-26)");
        assert_eq!(esin(0x3FC9_0FDB), ONE);
        assert_eq!(eexp(0), ONE);
        assert_eq!(esum([ONE; 4]), 0x4080_0000);
        assert!(float::is_zero(esadd([0; 4])));
    }
}
