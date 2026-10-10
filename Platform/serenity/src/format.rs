//! A Vulkan format by the name the C# binding gives it, for a pass's `#pragma format`.
//!
//! The names and values are Silk.NET.Vulkan 2.23.0's `Format` enumeration, every one of its 322 names, taken on
//! 2026-10-09; the C# resolves a pragma's word through that enumeration, so which words are formats is the
//! binding's answer and not Vulkan's. WiseMan holds this table to the enumeration. See EmuSen_RustPlatform.md §17.3.

/// Each name and its value, in the order of the names' bytes.
pub const FORMATS: [(&str, i32); 322] = [
    ("A1B5G5R5UnormPack16", 1000470000), ("A1B5G5R5UnormPack16Khr", 1000470000), ("A1R5G5B5UnormPack16", 8), ("A2B10G10R10SNormPack32", 65),
    ("A2B10G10R10SintPack32", 69), ("A2B10G10R10SscaledPack32", 67), ("A2B10G10R10UintPack32", 68), ("A2B10G10R10UnormPack32", 64),
    ("A2B10G10R10UscaledPack32", 66), ("A2R10G10B10SNormPack32", 59), ("A2R10G10B10SintPack32", 63), ("A2R10G10B10SscaledPack32", 61),
    ("A2R10G10B10UintPack32", 62), ("A2R10G10B10UnormPack32", 58), ("A2R10G10B10UscaledPack32", 60), ("A4B4G4R4UnormPack16", 1000340001),
    ("A4B4G4R4UnormPack16Ext", 1000340001), ("A4R4G4B4UnormPack16", 1000340000), ("A4R4G4B4UnormPack16Ext", 1000340000), ("A8B8G8R8SNormPack32", 52),
    ("A8B8G8R8SintPack32", 56), ("A8B8G8R8SrgbPack32", 57), ("A8B8G8R8SscaledPack32", 54), ("A8B8G8R8UintPack32", 55), ("A8B8G8R8UnormPack32", 51),
    ("A8B8G8R8UscaledPack32", 53), ("A8Unorm", 1000470001), ("A8UnormKhr", 1000470001), ("Astc10x10SfloatBlock", 1000066011),
    ("Astc10x10SfloatBlockExt", 1000066011), ("Astc10x10SrgbBlock", 180), ("Astc10x10UnormBlock", 179), ("Astc10x5SfloatBlock", 1000066008),
    ("Astc10x5SfloatBlockExt", 1000066008), ("Astc10x5SrgbBlock", 174), ("Astc10x5UnormBlock", 173), ("Astc10x6SfloatBlock", 1000066009),
    ("Astc10x6SfloatBlockExt", 1000066009), ("Astc10x6SrgbBlock", 176), ("Astc10x6UnormBlock", 175), ("Astc10x8SfloatBlock", 1000066010),
    ("Astc10x8SfloatBlockExt", 1000066010), ("Astc10x8SrgbBlock", 178), ("Astc10x8UnormBlock", 177), ("Astc12x10SfloatBlock", 1000066012),
    ("Astc12x10SfloatBlockExt", 1000066012), ("Astc12x10SrgbBlock", 182), ("Astc12x10UnormBlock", 181), ("Astc12x12SfloatBlock", 1000066013),
    ("Astc12x12SfloatBlockExt", 1000066013), ("Astc12x12SrgbBlock", 184), ("Astc12x12UnormBlock", 183), ("Astc4x4SfloatBlock", 1000066000),
    ("Astc4x4SfloatBlockExt", 1000066000), ("Astc4x4SrgbBlock", 158), ("Astc4x4UnormBlock", 157), ("Astc5x4SfloatBlock", 1000066001),
    ("Astc5x4SfloatBlockExt", 1000066001), ("Astc5x4SrgbBlock", 160), ("Astc5x4UnormBlock", 159), ("Astc5x5SfloatBlock", 1000066002),
    ("Astc5x5SfloatBlockExt", 1000066002), ("Astc5x5SrgbBlock", 162), ("Astc5x5UnormBlock", 161), ("Astc6x5SfloatBlock", 1000066003),
    ("Astc6x5SfloatBlockExt", 1000066003), ("Astc6x5SrgbBlock", 164), ("Astc6x5UnormBlock", 163), ("Astc6x6SfloatBlock", 1000066004),
    ("Astc6x6SfloatBlockExt", 1000066004), ("Astc6x6SrgbBlock", 166), ("Astc6x6UnormBlock", 165), ("Astc8x5SfloatBlock", 1000066005),
    ("Astc8x5SfloatBlockExt", 1000066005), ("Astc8x5SrgbBlock", 168), ("Astc8x5UnormBlock", 167), ("Astc8x6SfloatBlock", 1000066006),
    ("Astc8x6SfloatBlockExt", 1000066006), ("Astc8x6SrgbBlock", 170), ("Astc8x6UnormBlock", 169), ("Astc8x8SfloatBlock", 1000066007),
    ("Astc8x8SfloatBlockExt", 1000066007), ("Astc8x8SrgbBlock", 172), ("Astc8x8UnormBlock", 171), ("B10G11R11UfloatPack32", 122),
    ("B10X6G10X6R10X6G10X6422Unorm4Pack16", 1000156011), ("B10X6G10X6R10X6G10X6422Unorm4Pack16Khr", 1000156011),
    ("B12X4G12X4R12X4G12X4422Unorm4Pack16", 1000156021), ("B12X4G12X4R12X4G12X4422Unorm4Pack16Khr", 1000156021), ("B16G16R16G16422Unorm", 1000156028),
    ("B16G16R16G16422UnormKhr", 1000156028), ("B4G4R4A4UnormPack16", 3), ("B5G5R5A1UnormPack16", 7), ("B5G6R5UnormPack16", 5), ("B8G8R8A8SNorm", 45),
    ("B8G8R8A8Sint", 49), ("B8G8R8A8Srgb", 50), ("B8G8R8A8Sscaled", 47), ("B8G8R8A8Uint", 48), ("B8G8R8A8Unorm", 44), ("B8G8R8A8Uscaled", 46),
    ("B8G8R8G8422Unorm", 1000156001), ("B8G8R8G8422UnormKhr", 1000156001), ("B8G8R8SNorm", 31), ("B8G8R8Sint", 35), ("B8G8R8Srgb", 36),
    ("B8G8R8Sscaled", 33), ("B8G8R8Uint", 34), ("B8G8R8Unorm", 30), ("B8G8R8Uscaled", 32), ("BC1RgbSrgbBlock", 132), ("BC1RgbUnormBlock", 131),
    ("BC1RgbaSrgbBlock", 134), ("BC1RgbaUnormBlock", 133), ("BC2SrgbBlock", 136), ("BC2UnormBlock", 135), ("BC3SrgbBlock", 138),
    ("BC3UnormBlock", 137), ("BC4SNormBlock", 140), ("BC4UnormBlock", 139), ("BC5SNormBlock", 142), ("BC5UnormBlock", 141), ("BC6HSfloatBlock", 144),
    ("BC6HUfloatBlock", 143), ("BC7SrgbBlock", 146), ("BC7UnormBlock", 145), ("D16Unorm", 124), ("D16UnormS8Uint", 128), ("D24UnormS8Uint", 129),
    ("D32Sfloat", 126), ("D32SfloatS8Uint", 130), ("E5B9G9R9UfloatPack32", 123), ("EacR11G11SNormBlock", 156), ("EacR11G11UnormBlock", 155),
    ("EacR11SNormBlock", 154), ("EacR11UnormBlock", 153), ("Etc2R8G8B8A1SrgbBlock", 150), ("Etc2R8G8B8A1UnormBlock", 149),
    ("Etc2R8G8B8A8SrgbBlock", 152), ("Etc2R8G8B8A8UnormBlock", 151), ("Etc2R8G8B8SrgbBlock", 148), ("Etc2R8G8B8UnormBlock", 147),
    ("G10X6B10X6G10X6R10X6422Unorm4Pack16", 1000156010), ("G10X6B10X6G10X6R10X6422Unorm4Pack16Khr", 1000156010),
    ("G10X6B10X6R10X62Plane420Unorm3Pack16", 1000156013), ("G10X6B10X6R10X62Plane420Unorm3Pack16Khr", 1000156013),
    ("G10X6B10X6R10X62Plane422Unorm3Pack16", 1000156015), ("G10X6B10X6R10X62Plane422Unorm3Pack16Khr", 1000156015),
    ("G10X6B10X6R10X62Plane444Unorm3Pack16", 1000330001), ("G10X6B10X6R10X62Plane444Unorm3Pack16Ext", 1000330001),
    ("G10X6B10X6R10X63Plane420Unorm3Pack16", 1000156012), ("G10X6B10X6R10X63Plane420Unorm3Pack16Khr", 1000156012),
    ("G10X6B10X6R10X63Plane422Unorm3Pack16", 1000156014), ("G10X6B10X6R10X63Plane422Unorm3Pack16Khr", 1000156014),
    ("G10X6B10X6R10X63Plane444Unorm3Pack16", 1000156016), ("G10X6B10X6R10X63Plane444Unorm3Pack16Khr", 1000156016),
    ("G12X4B12X4G12X4R12X4422Unorm4Pack16", 1000156020), ("G12X4B12X4G12X4R12X4422Unorm4Pack16Khr", 1000156020),
    ("G12X4B12X4R12X42Plane420Unorm3Pack16", 1000156023), ("G12X4B12X4R12X42Plane420Unorm3Pack16Khr", 1000156023),
    ("G12X4B12X4R12X42Plane422Unorm3Pack16", 1000156025), ("G12X4B12X4R12X42Plane422Unorm3Pack16Khr", 1000156025),
    ("G12X4B12X4R12X42Plane444Unorm3Pack16", 1000330002), ("G12X4B12X4R12X42Plane444Unorm3Pack16Ext", 1000330002),
    ("G12X4B12X4R12X43Plane420Unorm3Pack16", 1000156022), ("G12X4B12X4R12X43Plane420Unorm3Pack16Khr", 1000156022),
    ("G12X4B12X4R12X43Plane422Unorm3Pack16", 1000156024), ("G12X4B12X4R12X43Plane422Unorm3Pack16Khr", 1000156024),
    ("G12X4B12X4R12X43Plane444Unorm3Pack16", 1000156026), ("G12X4B12X4R12X43Plane444Unorm3Pack16Khr", 1000156026),
    ("G14X2B14X2R14X22Plane420Unorm3Pack16Arm", 1000609012), ("G14X2B14X2R14X22Plane422Unorm3Pack16Arm", 1000609013),
    ("G16B16G16R16422Unorm", 1000156027), ("G16B16G16R16422UnormKhr", 1000156027), ("G16B16R162Plane420Unorm", 1000156030),
    ("G16B16R162Plane420UnormKhr", 1000156030), ("G16B16R162Plane422Unorm", 1000156032), ("G16B16R162Plane422UnormKhr", 1000156032),
    ("G16B16R162Plane444Unorm", 1000330003), ("G16B16R162Plane444UnormExt", 1000330003), ("G16B16R163Plane420Unorm", 1000156029),
    ("G16B16R163Plane420UnormKhr", 1000156029), ("G16B16R163Plane422Unorm", 1000156031), ("G16B16R163Plane422UnormKhr", 1000156031),
    ("G16B16R163Plane444Unorm", 1000156033), ("G16B16R163Plane444UnormKhr", 1000156033), ("G8B8G8R8422Unorm", 1000156000),
    ("G8B8G8R8422UnormKhr", 1000156000), ("G8B8R82Plane420Unorm", 1000156003), ("G8B8R82Plane420UnormKhr", 1000156003),
    ("G8B8R82Plane422Unorm", 1000156005), ("G8B8R82Plane422UnormKhr", 1000156005), ("G8B8R82Plane444Unorm", 1000330000),
    ("G8B8R82Plane444UnormExt", 1000330000), ("G8B8R83Plane420Unorm", 1000156002), ("G8B8R83Plane420UnormKhr", 1000156002),
    ("G8B8R83Plane422Unorm", 1000156004), ("G8B8R83Plane422UnormKhr", 1000156004), ("G8B8R83Plane444Unorm", 1000156006),
    ("G8B8R83Plane444UnormKhr", 1000156006), ("Pvrtc12BppSrgbBlockImg", 1000054004), ("Pvrtc12BppUnormBlockImg", 1000054000),
    ("Pvrtc14BppSrgbBlockImg", 1000054005), ("Pvrtc14BppUnormBlockImg", 1000054001), ("Pvrtc22BppSrgbBlockImg", 1000054006),
    ("Pvrtc22BppUnormBlockImg", 1000054002), ("Pvrtc24BppSrgbBlockImg", 1000054007), ("Pvrtc24BppUnormBlockImg", 1000054003),
    ("R10X6G10X6B10X6A10X6Uint4Pack16Arm", 1000609002), ("R10X6G10X6B10X6A10X6Unorm4Pack16", 1000156009),
    ("R10X6G10X6B10X6A10X6Unorm4Pack16Khr", 1000156009), ("R10X6G10X6Uint2Pack16Arm", 1000609001), ("R10X6G10X6Unorm2Pack16", 1000156008),
    ("R10X6G10X6Unorm2Pack16Khr", 1000156008), ("R10X6UintPack16Arm", 1000609000), ("R10X6UnormPack16", 1000156007),
    ("R10X6UnormPack16Khr", 1000156007), ("R12X4G12X4B12X4A12X4Uint4Pack16Arm", 1000609005), ("R12X4G12X4B12X4A12X4Unorm4Pack16", 1000156019),
    ("R12X4G12X4B12X4A12X4Unorm4Pack16Khr", 1000156019), ("R12X4G12X4Uint2Pack16Arm", 1000609004), ("R12X4G12X4Unorm2Pack16", 1000156018),
    ("R12X4G12X4Unorm2Pack16Khr", 1000156018), ("R12X4UintPack16Arm", 1000609003), ("R12X4UnormPack16", 1000156017),
    ("R12X4UnormPack16Khr", 1000156017), ("R14X2G14X2B14X2A14X2Uint4Pack16Arm", 1000609008), ("R14X2G14X2B14X2A14X2Unorm4Pack16Arm", 1000609011),
    ("R14X2G14X2Uint2Pack16Arm", 1000609007), ("R14X2G14X2Unorm2Pack16Arm", 1000609010), ("R14X2UintPack16Arm", 1000609006),
    ("R14X2UnormPack16Arm", 1000609009), ("R16G16B16A16SNorm", 92), ("R16G16B16A16Sfloat", 97), ("R16G16B16A16Sint", 96), ("R16G16B16A16Sscaled", 94),
    ("R16G16B16A16Uint", 95), ("R16G16B16A16Unorm", 91), ("R16G16B16A16Uscaled", 93), ("R16G16B16SNorm", 85), ("R16G16B16Sfloat", 90),
    ("R16G16B16Sint", 89), ("R16G16B16Sscaled", 87), ("R16G16B16Uint", 88), ("R16G16B16Unorm", 84), ("R16G16B16Uscaled", 86),
    ("R16G16S105NV", 1000464000), ("R16G16SNorm", 78), ("R16G16Sfixed5NV", 1000464000), ("R16G16Sfloat", 83), ("R16G16Sint", 82),
    ("R16G16Sscaled", 80), ("R16G16Uint", 81), ("R16G16Unorm", 77), ("R16G16Uscaled", 79), ("R16SNorm", 71), ("R16Sfloat", 76), ("R16Sint", 75),
    ("R16Sscaled", 73), ("R16Uint", 74), ("R16Unorm", 70), ("R16Uscaled", 72), ("R32G32B32A32Sfloat", 109), ("R32G32B32A32Sint", 108),
    ("R32G32B32A32Uint", 107), ("R32G32B32Sfloat", 106), ("R32G32B32Sint", 105), ("R32G32B32Uint", 104), ("R32G32Sfloat", 103), ("R32G32Sint", 102),
    ("R32G32Uint", 101), ("R32Sfloat", 100), ("R32Sint", 99), ("R32Uint", 98), ("R4G4B4A4UnormPack16", 2), ("R4G4UnormPack8", 1),
    ("R5G5B5A1UnormPack16", 6), ("R5G6B5UnormPack16", 4), ("R64G64B64A64Sfloat", 121), ("R64G64B64A64Sint", 120), ("R64G64B64A64Uint", 119),
    ("R64G64B64Sfloat", 118), ("R64G64B64Sint", 117), ("R64G64B64Uint", 116), ("R64G64Sfloat", 115), ("R64G64Sint", 114), ("R64G64Uint", 113),
    ("R64Sfloat", 112), ("R64Sint", 111), ("R64Uint", 110), ("R8BoolArm", 1000460000), ("R8G8B8A8SNorm", 38), ("R8G8B8A8Sint", 42),
    ("R8G8B8A8Srgb", 43), ("R8G8B8A8Sscaled", 40), ("R8G8B8A8Uint", 41), ("R8G8B8A8Unorm", 37), ("R8G8B8A8Uscaled", 39), ("R8G8B8SNorm", 24),
    ("R8G8B8Sint", 28), ("R8G8B8Srgb", 29), ("R8G8B8Sscaled", 26), ("R8G8B8Uint", 27), ("R8G8B8Unorm", 23), ("R8G8B8Uscaled", 25), ("R8G8SNorm", 17),
    ("R8G8Sint", 21), ("R8G8Srgb", 22), ("R8G8Sscaled", 19), ("R8G8Uint", 20), ("R8G8Unorm", 16), ("R8G8Uscaled", 18), ("R8SNorm", 10),
    ("R8Sint", 14), ("R8Srgb", 15), ("R8Sscaled", 12), ("R8Uint", 13), ("R8Unorm", 9), ("R8Uscaled", 11), ("S8Uint", 127), ("Undefined", 0),
    ("X8D24UnormPack32", 125),
];

/// The formats a chain names itself.
pub const R8G8B8A8_UNORM: i32 = 37;
pub const R8G8B8A8_SRGB: i32 = 43;
pub const R16G16B16A16_SFLOAT: i32 = 97;

fn named(name: &str) -> Option<i32> {
    FORMATS.binary_search_by(|(known, _)| known.as_bytes().cmp(name.as_bytes())).ok().map(|at| FORMATS[at].1)
}

/// A pragma's word as the C# spells it before asking the enumeration: the text before the first underscore as it
/// is, and each part after one with its first letter raised and the rest lowered, the underscores gone.
///
/// Only ASCII is cased, with the two characters .NET's invariant casing sends into ASCII: the long s raised to S,
/// and the Kelvin sign lowered to k (measured over every character, 2026-10-09). No other casing can make or
/// unmake a name, a number or white space, which are all the enumeration reads.
fn spelled(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for (index, part) in name.split('_').enumerate() {
        if index == 0 {
            out.push_str(part);
            continue;
        }
        for (at, c) in part.chars().enumerate() {
            out.push(match (at, c) {
                (0, '\u{17F}') => 'S',
                (0, _) => c.to_ascii_uppercase(),
                (_, '\u{212A}') => 'k',
                _ => c.to_ascii_lowercase(),
            });
        }
    }
    out
}

/// An integer as `Enum.TryParse` reads one: a sign, digits, then the white space number parsing knows, then NULs.
fn number(text: &str) -> Option<i32> {
    let (negative, rest) = match text.as_bytes().first()? {
        b'-' => (true, &text[1..]),
        b'+' => (false, &text[1..]),
        _ => (false, text),
    };
    let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
    if digits == 0 {
        return None;
    }
    let after = rest[digits..].trim_start_matches(['\u{9}', '\u{A}', '\u{B}', '\u{C}', '\u{D}', ' ']);
    if !after.chars().all(|c| c == '\0') {
        return None;
    }
    let mut value: i64 = 0;
    for digit in rest[..digits].bytes() {
        value = value * 10 + (digit - b'0') as i64;
        if value > 1 << 31 {
            return None;
        }
    }
    i32::try_from(if negative { -value } else { value }).ok()
}

/// `SlangChain.ParseFormat`, less its fallback: the format a pass's `#pragma format` names, or none for a word that
/// names no format, when the C# falls back to the format the preset's own keys give.
///
/// The word is spelled the binding's way and handed to .NET's `Enum.TryParse`, whose rules these are, each
/// measured: white space before the word is dropped; a word that then begins with a digit or a sign is a number
/// and any 32-bit number is a format, defined or not; anything else is names between commas, each with the white
/// space round it dropped, every one of which must be a name, and the answer is their values or-ed together.
pub fn parse(name: &str) -> Option<i32> {
    if name.is_empty() {
        return None;
    }
    let spelled = spelled(name);
    let text = spelled.trim_start_matches(crate::text::is_space);
    if text.starts_with(|c: char| c.is_ascii_digit() || c == '-' || c == '+') {
        return number(text);
    }
    let mut value = 0;
    for name in text.split(',') {
        value |= named(name.trim_matches(crate::text::is_space))?;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_are_in_order_and_the_chain_s_own_formats_are_among_them() {
        assert!(FORMATS.windows(2).all(|pair| pair[0].0.as_bytes() < pair[1].0.as_bytes()));
        assert_eq!((named("R8G8B8A8Unorm"), named("R8G8B8A8Srgb"), named("R16G16B16A16Sfloat")), (Some(R8G8B8A8_UNORM), Some(R8G8B8A8_SRGB), Some(R16G16B16A16_SFLOAT)));
        assert_eq!(named("Undefined"), Some(0));
        assert_eq!(named("R8Snorm"), None);
    }

    /// Each answer is .NET 10's for the same word, measured 2026-10-09; WiseMan asks both for more.
    #[test]
    fn a_pragma_s_word_is_read_as_the_csharp_reads_it() {
        for (word, format) in [
            ("R8G8B8A8_UNORM", Some(37)),
            ("R8G8B8A8_SRGB", Some(43)),
            ("R16G16B16A16_SFLOAT", Some(97)),
            ("A2B10G10R10_UNORM_PACK32", Some(64)),
            ("r8g8b8a8_unorm", None),
            ("R8G8B8A8Unorm", Some(37)),
            ("R8G8B8A8_unorm", Some(37)),
            ("R8G8B8A8__UNORM", Some(37)),
            ("R8G8B8A8_UNORM_", Some(37)),
            ("_R8G8B8A8_UNORM", None),
            ("ASTC_4x4_UNORM_BLOCK", None),
            ("BC1_RGB_UNORM_BLOCK", Some(131)),
            ("G8B8G8R8_422_UNORM", Some(1_000_156_000)),
            ("R8", None),
            ("_", None),
            ("", None),
            ("37", Some(37)),
            (" 37", Some(37)),
            ("37 ", Some(37)),
            ("\u{A0}37", Some(37)),
            ("37\u{A0}", None),
            ("\u{2003}37", Some(37)),
            ("37\u{2003}", None),
            ("\u{85}37", Some(37)),
            ("37\u{85}", None),
            ("+37", Some(37)),
            ("-37", Some(-37)),
            ("-1", Some(-1)),
            ("-0", Some(0)),
            ("007", Some(7)),
            ("00000000000000000037", Some(37)),
            ("1_000", Some(1000)),
            ("2147483647", Some(i32::MAX)),
            ("2147483648", None),
            ("-2147483648", Some(i32::MIN)),
            ("-2147483649", None),
            ("99999999999", None),
            ("37\0", Some(37)),
            ("37\0\0", Some(37)),
            ("37 \0", Some(37)),
            ("37\0 ", None),
            ("\x0037", None),
            ("37\r\n", Some(37)),
            ("37\u{B}", Some(37)),
            ("0x25", None),
            ("1e2", None),
            ("3.0", None),
            ("37abc", None),
            ("- 37", None),
            ("--1", None),
            ("+", None),
            ("-", None),
            ("37,43", None),
            ("1, 2", None),
            ("\u{FF11}\u{FF12}", None),
            ("R8G8B8A8Unorm,R8G8B8A8Srgb", Some(47)),
            ("R8G8B8A8Unorm, R8G8B8A8Srgb", Some(47)),
            ("R8G8B8A8Unorm ,R8G8B8A8Srgb", Some(47)),
            ("R8G8B8A8Unorm\u{2003},\u{A0}R8G8B8A8Srgb", Some(47)),
            ("R8Unorm,R8Unorm", Some(9)),
            ("R8Unorm,R8Snorm", None),
            (" R8G8B8A8Unorm ", Some(37)),
            ("R8G8B8A8Unorm\u{A0}", Some(37)),
            ("R8G8B8A8Unorm\t", Some(37)),
            ("\nR8G8B8A8Unorm", Some(37)),
            ("R8G8B8A8Unorm\0", None),
            ("R8G8B8A8Unorm,", None),
            (",R8G8B8A8Unorm", None),
            ("R8G8B8A8Unorm,,R8G8B8A8Srgb", None),
            ("R8G8B8A8Unorm, ", None),
            (",", None),
            (" ", None),
            ("R8G8B8A8Unorm,37", None),
            ("37,R8G8B8A8Unorm", None),
            ("Undefined", Some(0)),
            ("Format", None),
            ("R8G8B8A8_\u{17F}RGB", Some(43)),
            ("BC1_RGB_UNORM_BLOC\u{212A}", Some(131)),
            ("BC1_RGB_UNORM_\u{212A}LOCK", None),
            ("R8G8B8A8_S\u{212A}GB", None),
            ("R8G8B8A8_U\u{130}NORM", None),
            ("R8G8B8A8_\u{131}", None),
        ] {
            assert_eq!(parse(word), format, "{word:?}");
        }
    }
}
