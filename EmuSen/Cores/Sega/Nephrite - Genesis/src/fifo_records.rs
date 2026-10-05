//! The VDPFIFOTesting reader (Nephrite_Plan.md §3.3): Nemesis's program leaves a record per test in the 68000's RAM
//! from `$FF0000` (lines, text length, data length, the name, a flag, the expected data, the data read), with `$8000`
//! between pages and `$FFFF` after the last; the program draws each page and waits for a button, and A runs it to the
//! end. Nephrite_Native.md §13 is the record.

/// One test: its name, and the expected and read data when it carries an expectation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
    pub name: String,
    pub expected: Vec<u8>,
    pub actual: Vec<u8>,
}

impl Record {
    pub fn passed(&self) -> bool {
        self.expected.is_empty() || self.expected == self.actual
    }
}

fn word(b: &[u8], a: usize) -> u16 {
    u16::from_be_bytes([b[a & 0xFFFF], b[(a + 1) & 0xFFFF]])
}

/// The records in `ram`, in order, up to the end of the results or the first empty one.
pub fn records(ram: &[u8]) -> Vec<Record> {
    let mut out = Vec::new();
    let mut a = 0usize;
    while a < 0xF000 {
        let lines = word(ram, a);
        if lines == 0 || lines == 0xFFFF {
            break;
        }
        if lines == 0x8000 {
            a += 2;
            continue;
        }
        let (text, data) = (word(ram, a + 2) as usize, word(ram, a + 4) as usize);
        let name = String::from_utf8_lossy(&ram[a + 6..a + 6 + text]).trim().to_string();
        let flag = word(ram, a + 6 + text);
        let e = a + 8 + text;
        let (expected, actual) = if flag != 0 { (ram[e..e + data].to_vec(), ram[e + data..e + 2 * data].to_vec()) } else { (Vec::new(), ram[e..e + data].to_vec()) };
        out.push(Record { name, expected, actual });
        a = e + if flag != 0 { 2 * data } else { data };
    }
    out
}

/// The A button held for ten frames at frame 60, which takes the program through every page to its last.
pub fn press(frame: i64) -> u32 {
    if (60..70).contains(&frame) { 1 << 6 } else { 0 }
}
