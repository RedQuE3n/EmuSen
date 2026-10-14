//! Disassembles a 68000 program image, or finds where it names a string: `dis <image> <hex start> <count>`, or
//! `dis <image> find <text>`. A development tool for reading test programs (Nephrite_Native.md §50).

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let image = std::fs::read(&args[1]).expect("the image");
    let word = |a: u32| -> u16 {
        let a = a as usize;
        if a + 1 < image.len() { u16::from_be_bytes([image[a], image[a + 1]]) } else { 0 }
    };
    if args[2] == "find" {
        let needle = args[3].as_bytes();
        for (i, w) in image.windows(needle.len()).enumerate() {
            if w == needle {
                println!("{i:06X}");
            }
        }
        return;
    }
    let mut a = u32::from_str_radix(&args[2], 16).expect("a hex address");
    for _ in 0..args[3].parse::<u32>().expect("a count") {
        let i = beryl_m68k::disasm::disassemble(&word, a);
        println!("{a:06X}  {:<8} {}", i.mnemonic, i.operands);
        a += i.bytes.len().max(2) as u32;
    }
}
