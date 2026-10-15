//! The cartridge's header at `$100` as the Genesis reads it: its system, titles, serial, devices, markets and the save
//! RAM it declares, and the cartridges whose save RAM it does not declare. The layout is the cartridge header of
//! Nephrite_Plan.md §2.3; a disc's system area carries the same header from its `$100`.

/// The header's region field as the three markets it allows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Markets {
    pub(crate) japan: bool,
    pub(crate) americas: bool,
    pub(crate) europe: bool,
}

impl Markets {
    /// PAL only when Europe is the one market allowed; an empty field reads as NTSC.
    pub(crate) fn pal(self) -> bool {
        self.europe && !self.japan && !self.americas
    }

    /// The old form (`J`, `U`, `E` in any order) or the new single hex digit (bit 0 Japan, bit 2 Americas, bit 3
    /// Europe). A field that begins `EUR` spells the word and is Europe alone, its `U` no code: Superman's and Another
    /// World's say `EUROPE` and The Smurfs 2's `Europe`, and the first and last stop on an American console
    /// (Nephrite_Native.md §47.4).
    pub(crate) fn parse(field: &[u8]) -> Markets {
        if field.len() >= 3 && field[..3].eq_ignore_ascii_case(b"EUR") {
            return Markets { japan: false, americas: false, europe: true };
        }
        let text: Vec<u8> = field.iter().copied().filter(|b| !b.is_ascii_whitespace() && *b != 0).collect();
        if text.len() == 1 && text[0].is_ascii_hexdigit() && !b"EJU".contains(&text[0].to_ascii_uppercase()) {
            let v = (text[0] as char).to_digit(16).unwrap_or(0);
            return Markets { japan: v & 1 != 0, americas: v & 4 != 0, europe: v & 8 != 0 };
        }
        let has = |c: u8| text.iter().any(|b| b.to_ascii_uppercase() == c);
        Markets { japan: has(b'J'), americas: has(b'U'), europe: has(b'E') }
    }
}

/// The cartridge's declared save RAM: its battery, which byte lanes it occupies, and its bus range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SaveRam {
    pub(crate) battery: bool,
    /// 0 both lanes, 2 even addresses only, 3 odd addresses only.
    pub(crate) lanes: u8,
    pub(crate) start: u32,
    pub(crate) end: u32,
}

impl SaveRam {
    /// Bytes the RAM holds: the range, halved when it occupies one lane; zero for a range that runs backwards.
    pub(crate) fn bytes(&self) -> usize {
        if self.end < self.start {
            return 0;
        }
        let span = (self.end - self.start) as usize + 1;
        if self.lanes == 0 { span } else { span.div_ceil(2) }
    }
}

/// Cartridges that keep their saves in a RAM their header does not declare, by serial (Nephrite_Native.md §46.2,
/// §47.5). Each writes its saves' odd bytes above its ROM, and Sega Retro gives each a battery save. In the table's
/// order: Madden NFL 98, College Football USA 96, FIFA Soccer 97, the same program under a pirate's serial, NHL 96,
/// NHL 98, PGA Tour Golf, Starflight, Buck Rogers, HardBall III, Summer Challenge, Winter Challenge and Test Drive II.
/// The header's word is taken first where it has one.
pub(in crate::genesis) const UNDECLARED_SAVE: [&str; 13] = ["T-172196", "T-172046", "T-172156", "T-183457", "T-172036", "T-172176", "T-50086", "T-50216", "T-50286", "ACLD012", "ACLD013", "ACLD007", "ACLD008"];

/// The board such a cartridge is given: the one plutiedev's "Saving progress with SRAM" describes, 32 KiB on the odd
/// bytes of `$200001`-`$20FFFF` with a battery. A board with a smaller chip would repeat within the range.
pub(in crate::genesis) fn undeclared_save(serial: &str) -> Option<SaveRam> {
    let s = serial.get(2..).unwrap_or("").trim_start_matches([' ', '_']);
    UNDECLARED_SAVE.iter().any(|known| s.starts_with(known)).then_some(SaveRam { battery: true, lanes: 3, start: 0x20_0001, end: 0x20_FFFF })
}

/// The fields of the 256-byte header at $100 that the stub reads.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Header {
    pub(crate) system_type: String,
    pub(in crate::genesis) domestic_title: String,
    pub(crate) overseas_title: String,
    pub(crate) serial: String,
    pub(in crate::genesis) checksum: u16,
    pub(in crate::genesis) devices: String,
    pub(crate) markets: Markets,
    pub(crate) save: Option<SaveRam>,
}

fn text(b: &[u8]) -> String {
    let s: String = b.iter().map(|&c| if (0x20..0x7F).contains(&c) { c as char } else { ' ' }).collect();
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn long(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

impl Header {
    /// The header whose first byte (the cartridge's $100) is at `h`; None when the image is too short to hold it.
    pub(crate) fn read(image: &[u8], h: usize) -> Option<Header> {
        let b = image.get(h..h + 0x100)?;
        let save = (&b[0xB0..0xB2] == b"RA").then(|| SaveRam {
            battery: b[0xB2] & 0x40 != 0,
            lanes: (b[0xB2] >> 3) & 3,
            start: long(b, 0xB4),
            end: long(b, 0xB8),
        });
        let serial = text(&b[0x80..0x8E]);
        let save = save.or_else(|| undeclared_save(&serial));
        Some(Header {
            system_type: text(&b[0x00..0x10]),
            domestic_title: text(&b[0x20..0x50]),
            overseas_title: text(&b[0x50..0x80]),
            serial,
            checksum: u16::from_be_bytes([b[0x8E], b[0x8F]]),
            devices: text(&b[0x90..0xA0]),
            markets: Markets::parse(&b[0xF0..0xF3]),
            save,
        })
    }

    /// Whether the system field names the 32X.
    pub(crate) fn is_32x(&self) -> bool {
        self.system_type.contains("32X")
    }
}
