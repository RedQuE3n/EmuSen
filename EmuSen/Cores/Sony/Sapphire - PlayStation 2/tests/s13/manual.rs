//! What the VU User's Manual gives for micro mode, transcribed apart from the interpreter (`src/vu`): the pipeline's
//! visible delays (§3.4, §7.4), the EFU as §7.2's nano code computes it, and the micro-only lower instructions' pages
//! (§4.3) where a generator needs a value. A lane's arithmetic is the FPU's chapter-5 transcription (`s9::manual`),
//! as for the macro instructions (`s12::manual`); the EFU's accumulator adds a product as ADD does, which the
//! `vu/lower/efu` console text decides (Sapphire_Disputes.md D-84).

use crate::s9::manual as fpu;

/// The flags an FMAC instruction writes at its S stage reach an instruction issued this many cycles later (§3.4.4,
/// Figures 3-16 and 3-17).
pub const FLAGS: u64 = 4;
/// DIV's and SQRT's latency and RSQRT's: Q is read new from this many cycles on (§3.4.5, §7.4).
pub const DIV: u64 = 7;
pub const RSQRT: u64 = 13;
/// A load into an integer register is read this many cycles later (§3.4.1, ILW's "1/4").
pub const LOAD: u64 = 4;

/// §7.4's EFU latencies by micro name: P is read new from this many cycles on, and another EFU operation may issue a
/// cycle sooner (§3.4.6).
pub fn efu_latency(name: &str) -> u64 {
    match name {
        "esadd" => 11,
        "ersadd" | "eleng" | "ersqrt" => 18,
        "erleng" => 24,
        "eatan" | "eatanxy" | "eatanxz" => 54,
        "esum" | "esqrt" | "ercpr" => 12,
        "esin" => 29,
        "eexp" => 44,
        _ => panic!("{name}"),
    }
}

const ONE: u32 = 0x3F80_0000;
const S: [u32; 5] = [0x3F80_0000, 0xBE2A_AAA4, 0x3C08_873E, 0xB94F_B21F, 0x362E_9C14];
const T: [u32; 8] = [0x3F7F_FFF5, 0xBEAA_A61C, 0x3E4C_40A6, 0xBE0E_6C63, 0x3DC5_77DF, 0xBD65_01C4, 0x3CB3_1652, 0xBB84_D7E7];
const PI_4: u32 = 0x3F49_0FDB;
const E: [u32; 6] = [0x3E7F_FFA8, 0x3D00_07F4, 0x3B29_D3FF, 0x3933_E553, 0x36B6_3510, 0x3539_61AC];

fn mul(a: u32, b: u32) -> u32 {
    fpu::mul(a, b).0
}
/// The nano code's MADDA and MADD: the product, then ADD.
fn madd(acc: u32, a: u32, b: u32) -> u32 {
    fpu::add(acc, mul(a, b)).0
}
fn div(a: u32, b: u32) -> u32 {
    fpu::div(a, b).0
}

/// §7.2's nano code for each EFU instruction on VF[fs] (`v`) and its fsf field (`f`), in the code's order.
pub fn efu(name: &str, v: [u32; 4], f: usize) -> u32 {
    let x = v[f];
    let sadd = || madd(madd(mul(v[0], v[0]), v[1], v[1]), v[2], v[2]);
    let arctan = |num: u32, den: u32| {
        let t = div(num, den);
        let t2 = mul(t, t);
        let mut acc = mul(T[0], t);
        let mut odd = mul(t2, t);
        for k in T.iter().take(7).skip(1) {
            let next = mul(odd, t2);
            acc = madd(acc, odd, *k);
            odd = next;
        }
        madd(madd(acc, ONE, PI_4), odd, T[7])
    };
    match name {
        "esadd" => sadd(),
        "ersadd" => div(ONE, sadd()),
        "eleng" => fpu::sqrt(sadd()),
        "erleng" => fpu::rsqrt(ONE, sadd()),
        "esum" => madd(madd(madd(mul(v[0], ONE), v[1], ONE), v[2], ONE), v[3], ONE),
        "ercpr" => div(ONE, x),
        "esqrt" => fpu::sqrt(x),
        "ersqrt" => fpu::rsqrt(ONE, x),
        "esin" => {
            let x2 = mul(x, x);
            let x3 = mul(x2, x);
            let x5 = mul(x3, x2);
            let x7 = mul(x5, x2);
            let x9 = mul(x7, x2);
            madd(madd(madd(madd(mul(x, S[0]), x3, S[1]), x5, S[2]), x7, S[3]), x9, S[4])
        }
        "eatan" => arctan(fpu::sub(x, ONE).0, fpu::add(x, ONE).0),
        "eatanxy" => arctan(fpu::sub(v[1], v[0]).0, fpu::add(v[1], v[0]).0),
        "eatanxz" => arctan(fpu::sub(v[2], v[0]).0, fpu::add(v[2], v[0]).0),
        "eexp" => {
            let mut pow = mul(x, x);
            let mut acc = mul(x, E[0]);
            for e in &E[1..5] {
                let next = mul(pow, x);
                acc = madd(acc, pow, *e);
                pow = next;
            }
            let p = madd(madd(acc, ONE, ONE), pow, E[5]);
            let p = mul(p, p);
            div(ONE, mul(p, p))
        }
        _ => panic!("{name}"),
    }
}

pub const EFU: [&str; 13] = ["eatan", "eatanxy", "eatanxz", "eexp", "eleng", "ercpr", "erleng", "ersadd", "ersqrt", "esadd", "esin", "esqrt", "esum"];

/// Whether an EFU instruction takes a field (`p, vfNf`) rather than the vector.
pub fn efu_scalar(name: &str) -> bool {
    matches!(name, "eatan" | "eexp" | "ercpr" | "ersqrt" | "esin" | "esqrt")
}

/// The inputs `vu/lower/efu`'s console text names, as its lines show them (Sapphire_Native.md §23): each constant in
/// every lane but the two ramps, a scalar operation reading z.
pub const EFU_INPUTS: [(&str, [u32; 4]); 16] = [
    ("CVF_ZERO", [0; 4]),
    ("CVF_NEGZERO", [0x8000_0000; 4]),
    ("CVF_MAX", [0x7FFF_FFFF; 4]),
    ("CVF_MIN", [0xFFFF_FFFF; 4]),
    ("CVF_MAX_MANTISSA", [0x3FFF_FFFF; 4]),
    ("CVF_MAX_EXP", [0x7F80_0000; 4]),
    ("CVF_MIN_EXP", [0x007F_FFFF; 4]),
    ("CVF_ONE", [ONE; 4]),
    ("CVF_NEGONE", [0xBF80_0000; 4]),
    ("CVF_GARBAGE1", [0x0012_3456; 4]),
    ("CVF_GARBAGE2", [0xDEAD_BEEF; 4]),
    ("CVF_INCREASING", [ONE, 0x4000_0000, 0x4040_0000, 0x4080_0000]),
    ("CVF_DECREASING", [0x4080_0000, 0x4040_0000, 0x4000_0000, ONE]),
    ("CVF_PI_OVER2", [0x3FC9_0FDB; 4]),
    ("CVF_PI", [0x4049_0FDB; 4]),
    ("CVF_3PI_OVER2", [0x4096_CBE4; 4]),
];

/// ps2autotests' `vu/lower/efu` text, a console's output (Sapphire_References.md §2): P for each of `EFU`'s operations
/// over each of `EFU_INPUTS`, in the text's order.
pub const EFU_CONSOLE: [[u32; 16]; 13] = [
    [0x33E6_0000, 0x33E6_0000, 0x3FC9_0FD9, 0x3FC9_0FD9, 0x3F8D_B70B, 0x3FC9_0FD9, 0x33E6_0000, 0x3F49_0FDA, 0xFFC9_0FD7, 0x33E6_0000, 0x3FC9_0FD9, 0x3F9F_E0BA, 0x3F8D_B70B, 0x3F80_7F4C, 0x3FA1_9DC4, 0x3FAE_4BE7],
    [0x7FC9_0FD7, 0xFFC9_0FD7, 0x3F49_0FDA, 0x3F49_0FDA, 0x3F49_0FDA, 0x3F49_0FDA, 0x7FC9_0FD7, 0x3F49_0FDA, 0x3F49_0FDA, 0x7FC9_0FD7, 0x3F49_0FDA, 0x3F8D_B70B, 0x3F24_BC7D, 0x3F49_0FDA, 0x3F49_0FDA, 0x3F49_0FDA],
    [0x7FC9_0FD7, 0xFFC9_0FD7, 0x3F49_0FDA, 0x3F49_0FDA, 0x3F49_0FDA, 0x3F49_0FDA, 0x7FC9_0FD7, 0x3F49_0FDA, 0x3F49_0FDA, 0x7FC9_0FD7, 0x3F49_0FDA, 0x3F9F_E0BA, 0x3EED_6339, 0x3F49_0FDA, 0x3F49_0FDA, 0x3F49_0FDA],
    [0x3F80_0000, 0x3F80_0000, 0x0000_0000, 0x0000_0000, 0x3E0A_9554, 0x0000_0000, 0x3F80_0000, 0x3EBC_5ABF, 0x402D_F0AA, 0x3F80_0000, 0x0000_0000, 0x3D4B_ED7F, 0x3E0A_9554, 0x3E54_DE73, 0x3D31_0121, 0x3C13_2F25],
    [0x0000_0000, 0x0000_0000, 0x5FB5_04F3, 0x5FB5_04F3, 0x405D_B3D6, 0x5FB5_04F3, 0x0000_0000, 0x3FDD_B3D7, 0x3FDD_B3D7, 0x0000_0000, 0x5F16_77E2, 0x406F_7750, 0x40AC_5345, 0x402E_1FEF, 0x40AE_1FEF, 0x4102_97F3],
    [0x7FFF_FFFF, 0xFFFF_FFFF, 0x0000_0000, 0x8000_0000, 0x3F00_0001, 0x0000_0000, 0x7FFF_FFFF, 0x3F80_0000, 0xBF80_0000, 0x7FFF_FFFF, 0xA03C_98EB, 0x3EAA_AAAB, 0x3F00_0000, 0x3F22_F983, 0x3EA2_F983, 0x3E59_4CB0],
    [0x7FFF_FFFF, 0x7FFF_FFFF, 0x1F35_04F3, 0x1F35_04F3, 0x3E93_CD3B, 0x1F35_04F3, 0x7FFF_FFFF, 0x3F13_CD3A, 0x3F13_CD3A, 0x7FFF_FFFF, 0x1FD9_C601, 0x3E88_D677, 0x3E3E_26EB, 0x3EBC_2FDA, 0x3E3C_2FDA, 0x3DFA_EA7A],
    [0x7FFF_FFFF, 0x7FFF_FFFF, 0x0000_0000, 0x0000_0000, 0x3DAA_AAAC, 0x0000_0000, 0x7FFF_FFFF, 0x3EAA_AAAB, 0x3EAA_AAAB, 0x7FFF_FFFF, 0x0000_0000, 0x3D92_4925, 0x3D0D_3DCB, 0x3E0A_5652, 0x3D0A_5652, 0x3C75_EECB],
    [0x7FFF_FFFF, 0x7FFF_FFFF, 0x1F35_04F3, 0x1F35_04F3, 0x3F35_04F3, 0x1F80_0000, 0x7FFF_FFFF, 0x3F80_0000, 0x3F80_0000, 0x7FFF_FFFF, 0x2FDB_BAAC, 0x3F13_CD3A, 0x3F35_04F3, 0x3F4C_4229, 0x3F10_6EBB, 0x3EEB_DB7A],
    [0x0000_0000, 0x0000_0000, 0x7FFF_FFFF, 0x7FFF_FFFF, 0x413F_FFFE, 0x7FFF_FFFF, 0x0000_0000, 0x4040_0000, 0x4040_0000, 0x0000_0000, 0x7EB0_E16B, 0x4160_0000, 0x41E8_0000, 0x40EC_DED9, 0x41EC_DED9, 0x4285_3D5A],
    [0x0000_0000, 0x0000_0000, 0x7FD7_6AA2, 0xFFD7_6AA2, 0x3F68_C816, 0x7F2E_D54B, 0x0000_0000, 0x3F57_6AA4, 0xBF57_6AA4, 0x0000_0000, 0x7EA2_556C, 0x3E12_6230, 0x3F68_C815, 0x3F80_0000, 0x3B56_5DE0, 0xBF1A_9934],
    [0x0000_0000, 0x0000_0000, 0x5FB5_04F3, 0x5FB5_04F3, 0x3FB5_04F3, 0x5F80_0000, 0x0000_0000, 0x3F80_0000, 0x3F80_0000, 0x0000_0000, 0x4F15_2108, 0x3FDD_B3D7, 0x3FB5_04F3, 0x3FA0_6C99, 0x3FE2_DFC4, 0x400A_EE73],
    [0x0000_0000, 0x8000_0000, 0x7FFF_FFFF, 0xFFFF_FFFF, 0x40FF_FFFE, 0x7FFF_FFFF, 0x0000_0000, 0x4080_0000, 0xC080_0000, 0x0000_0000, 0xDFAD_BEEE, 0x4120_0000, 0x4120_0000, 0x40C9_0FDA, 0x4149_0FDA, 0x4196_CBE4],
];
