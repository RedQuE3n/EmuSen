use crate::naming::{check, snake};
use crate::{State, StateReader, StateWriter, StringError, Truncated};

#[derive(Debug, Default, PartialEq)]
struct Pair {
    a: u8,
    b: u16,
}

impl State for Pair {
    type Error = Truncated;
    fn write_state(&self, w: &mut StateWriter) {
        w.u8("A", self.a);
        w.u16("B", self.b);
    }
    fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        self.a = r.u8()?;
        self.b = r.u16()?;
        Ok(())
    }
}

#[test]
fn primitives_are_little_endian_and_bools_one_byte() {
    let mut buf = [0u8; 20];
    let mut w = StateWriter::new(&mut buf);
    w.u32("a", 0x1122_3344);
    w.bool("b", true);
    w.i16("c", -2);
    w.u64("d", 0x0102_0304_0506_0708);
    w.i8("e", -1);
    w.i32("f", -3);
    assert_eq!(w.len(), 20);
    assert!(!w.overflowed());
    assert_eq!(buf, [0x44, 0x33, 0x22, 0x11, 1, 0xFE, 0xFF, 8, 7, 6, 5, 4, 3, 2, 1, 0xFF, 0xFD, 0xFF, 0xFF, 0xFF]);

    let mut r = StateReader::new(&buf);
    assert_eq!(r.u32(), Ok(0x1122_3344));
    assert_eq!(r.bool(), Ok(true));
    assert_eq!(r.i16(), Ok(-2));
    assert_eq!(r.u64(), Ok(0x0102_0304_0506_0708));
    assert_eq!(r.i8(), Ok(-1));
    assert_eq!(r.i32(), Ok(-3));
    assert_eq!(r.position(), 20);
    assert_eq!(r.u8(), Err(Truncated { at: 20, wanted: 1 }));
}

#[test]
fn arrays_carry_no_length_and_read_back() {
    let mut buf = [0u8; 2 + 8 + 4 + 8 + 2];
    let mut w = StateWriter::new(&mut buf);
    w.u16s("a", &[0x0102]);
    w.i32s("b", &[-1, 2]);
    w.u32s("c", &[0x0A0B_0C0D]);
    w.u64s("d", &[5]);
    w.bools("e", &[true, false]);
    assert_eq!(w.len(), buf.len());
    assert_eq!(buf[..6], [0x02, 0x01, 0xFF, 0xFF, 0xFF, 0xFF]);
    let (mut a, mut b, mut c, mut d, mut e) = ([0u16; 1], [0i32; 2], [0u32; 1], [0u64; 1], [true, true]);
    let mut odd = buf;
    odd[buf.len() - 2] = 2;
    let mut r = StateReader::new(&odd);
    r.u16s(&mut a).unwrap();
    r.i32s(&mut b).unwrap();
    r.u32s(&mut c).unwrap();
    r.u64s(&mut d).unwrap();
    r.bools(&mut e).unwrap();
    assert_eq!((a, b, c, d, e), ([0x0102], [-1, 2], [0x0A0B_0C0D], [5], [true, false]));
}

#[test]
fn a_counter_and_a_layout_count_what_a_writer_writes() {
    let pairs = [Pair { a: 1, b: 2 }, Pair { a: 3, b: 4 }];
    let write = |w: &mut StateWriter| {
        w.bytes("Ram", &[0; 3]);
        w.class("Cpu", &pairs[0]);
        w.structure("Reg", &pairs[1]);
        w.structures("Slots", &pairs);
        w.group("Bus", |w| w.bool("Open", true));
        w.string("Path", Some("ab"));
        w.bools("Flags", &[true, false]);
    };
    let mut counter = StateWriter::counter();
    write(&mut counter);
    let mut layout = StateWriter::layout();
    write(&mut layout);
    let mut buf = vec![0u8; counter.len()];
    let mut real = StateWriter::new(&mut buf);
    write(&mut real);
    assert_eq!((counter.len(), layout.len(), real.len()), (22, 22, 22));
    assert_eq!(buf, [0, 0, 0, 1, 1, 2, 0, 3, 4, 0, 1, 2, 0, 3, 4, 0, 1, 2, b'a', b'b', 1, 0]);
    assert!(counter.into_layout().is_empty());
    assert_eq!(
        layout.into_layout(),
        "0 3 u8[3] Ram\n3 1 class Cpu\n4 1 u8 Cpu.A\n5 2 u16 Cpu.B\n7 1 u8 Reg.A\n8 2 u16 Reg.B\n\
         10 1 u8 Slots[0].A\n11 2 u16 Slots[0].B\n13 1 u8 Slots[1].A\n14 2 u16 Slots[1].B\n16 1 bool Bus.Open\n17 3 string Path\n20 2 bool[2] Flags\n"
    );
}

#[test]
fn a_short_buffer_overflows_and_a_short_input_is_truncated() {
    let mut buf = [0u8; 3];
    let mut w = StateWriter::new(&mut buf);
    w.u32("a", 1);
    assert!(w.overflowed());
    assert_eq!(w.len(), 4);
    assert_eq!(buf, [0; 3]);
    assert_eq!(StateReader::new(&buf).u32(), Err(Truncated { at: 0, wanted: 4 }));
    let mut r = StateReader::new(&buf);
    assert_eq!(r.skip(2), Ok(()));
    assert_eq!(r.skip(2), Err(Truncated { at: 2, wanted: 2 }));
    assert_eq!(r.bytes(&mut [0; 2]), Err(Truncated { at: 2, wanted: 2 }));
}

#[test]
fn a_class_flag_is_read_as_csharp_reads_it() {
    let bytes = [1, 7, 8, 0, 0, 9, 9, 2, 5, 6, 0];
    let mut r = StateReader::new(&bytes);
    let mut first = Pair::default();
    r.class(&mut first).unwrap();
    assert_eq!(first, Pair { a: 7, b: 8 });
    let mut absent = Pair { a: 1, b: 1 };
    r.class(&mut absent).unwrap();
    assert_eq!(absent, Pair { a: 1, b: 1 });
    assert_eq!(r.u16(), Ok(0x0909));
    let mut odd = Pair::default();
    r.class(&mut odd).unwrap();
    assert_eq!(odd, Pair { a: 5, b: 6 });
    assert_eq!(r.present(), Err(Truncated { at: 11, wanted: 1 }));
}

#[test]
fn structures_read_every_element_with_no_flags() {
    let mut into = [Pair::default(), Pair::default()];
    StateReader::new(&[1, 2, 0, 3, 4, 0]).structures(&mut into).unwrap();
    assert_eq!(into, [Pair { a: 1, b: 2 }, Pair { a: 3, b: 4 }]);
}

#[test]
fn a_reader_is_current_until_the_header_names_a_version() {
    let mut r = StateReader::new(&[]);
    assert!(!r.before(i32::MAX));
    r.set_version(5);
    assert!(r.before(6));
    assert!(!r.before(5));
}

#[test]
fn strings_carry_a_seven_bit_length_as_binarywriter_writes_it() {
    for (text, prefix) in [("", vec![0u8]), ("a", vec![1]), (&"x".repeat(127)[..], vec![0x7F]), (&"x".repeat(128)[..], vec![0x80, 0x01]), (&"y".repeat(16384)[..], vec![0x80, 0x80, 0x01])] {
        let mut buf = vec![0u8; text.len() + 5];
        let mut w = StateWriter::new(&mut buf);
        w.string("s", Some(text));
        let n = w.len();
        assert_eq!(n, prefix.len() + text.len());
        assert_eq!(&buf[..prefix.len()], &prefix[..]);
        assert_eq!(StateReader::new(&buf[..n]).string().as_deref(), Ok(text));
    }
    let mut buf = [0u8; 8];
    let mut w = StateWriter::new(&mut buf);
    w.string("s", None);
    assert_eq!(w.len(), 1);
    assert_eq!(buf[0], 0);
}

#[test]
fn a_string_length_binaryreader_refuses_is_refused() {
    assert_eq!(StateReader::new(&[0x80, 0x80, 0x80, 0x80, 0x10]).string(), Err(StringError::BadLength { at: 0 }));
    assert_eq!(StateReader::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]).string(), Err(StringError::BadLength { at: 0 }));
    assert_eq!(StateReader::new(&[0x05, b'a']).string(), Err(StringError::Truncated(Truncated { at: 1, wanted: 5 })));
    assert_eq!(StateReader::new(&[0x02, 0xC3, 0x28]).string().as_deref(), Ok("\u{FFFD}("));
}

#[test]
fn the_rule_reads_the_csharp_names_as_the_cores_spell_them() {
    for (cs, rust) in [
        ("<TextureMemory>k__BackingField", "texture_memory"),
        ("<LastInstructionPC>k__BackingField", "last_instruction_pc"),
        ("_attributeDe", "attribute_de"),
        ("EntryLo0", "entry_lo0"),
        ("SH", "sh"),
        ("ShiftS", "shift_s"),
        ("IsViewer", "is_viewer"),
        ("HdmaIsHBlankDriven", "hdma_is_h_blank_driven"),
        ("SuppressVBlank", "suppress_v_blank"),
        ("_chrBank0", "chr_bank0"),
        ("_bank0Mode", "bank0_mode"),
        ("_a12", "a12"),
        ("PC", "pc"),
    ] {
        assert_eq!(snake(cs), rust, "{cs}");
    }
}

#[test]
fn the_check_finds_a_label_or_a_comment_that_names_another_field() {
    let source = "w.u8(\"A\", self.a);\nw.bool(\"Loop\", self.r#loop);\nw.u8(\"X\", self.y);\nw.i32(\"Type\", self.kind);\n\
                  self.a = r.u8()?; // A\nself.r#loop = r.bool()?; // Loop\nmachine.pc = r.u32()?; // PC\nself.y = r.u8()?; // X\n\
                  w.u8(\"Z\", self.z + 1);\nlet _ = 1; // not a read\n";
    let found = check(&[("x.rs", source)], &[("Type", "kind")], &["self.", "machine."]);
    assert_eq!((found.writes, found.reads), (4, 4));
    assert_eq!(found.wrong, ["x.rs:3 writes self.y as \"X\"", "x.rs:8 reads self.y under \"// X\""]);
    assert_eq!(check(&[("x.rs", source)], &[], &["self."]).wrong.len(), 3);
}

#[test]
fn a_skipped_field_derefs_to_its_value_and_never_differs() {
    let mut a = crate::Skip(3);
    *a += 1;
    assert_eq!(*a, 4);
    assert_eq!(a, crate::Skip(9));
}

#[test]
fn the_shared_codes_fill_minus_one_to_minus_eight_and_are_distinct() {
    use crate::ffi::status::*;
    let codes = [NULL, TRUNCATED, FOREIGN, VERSION, BAD_STRING, BUFFER_TOO_SMALL];
    assert_eq!(codes, [-1, -2, -3, -4, -5, -7]);
    assert!(codes.iter().all(|&c| c > FIRST_CORE));
    assert_eq!((FIRST_CORE, LAST_CORE), (-9, -255));
}

pub struct Toy {
    pair: Pair,
}

#[derive(Debug, PartialEq)]
pub struct ToyError(i32);

impl From<Truncated> for ToyError {
    fn from(_: Truncated) -> Self {
        ToyError(crate::ffi::status::TRUNCATED)
    }
}

impl crate::ffi::Status for ToyError {
    fn status(&self) -> i32 {
        self.0
    }
}

impl crate::ffi::StateMachine for Toy {
    type Error = ToyError;
    fn load_state(&mut self, data: &[u8]) -> Result<(), ToyError> {
        let mut next = Pair::default();
        let mut r = StateReader::new(data);
        next.a = r.u8()?;
        next.b = r.u16()?;
        self.pair = next;
        Ok(())
    }
    fn state_size(&self) -> usize {
        3
    }
    fn save_state(&self, out: &mut [u8]) -> Result<usize, ToyError> {
        let mut w = StateWriter::new(out);
        self.pair.write_state(&mut w);
        if w.overflowed() { Err(ToyError(crate::ffi::status::BUFFER_TOO_SMALL)) } else { Ok(w.len()) }
    }
    fn layout(&self) -> String {
        let mut w = StateWriter::layout();
        self.pair.write_state(&mut w);
        w.into_layout()
    }
}

crate::state_exports!(Toy, toy_load_state, toy_save_state_size, toy_save_state, toy_state_layout);

#[test]
fn the_exported_state_functions_load_save_size_and_list_through_the_abi() {
    let mut toy = Toy { pair: Pair::default() };
    let t: *mut Toy = &mut toy;
    unsafe {
        assert_eq!(toy_load_state(t, [7u8, 1, 2].as_ptr(), 3), 0);
        assert_eq!(toy_load_state(t, [9u8, 9].as_ptr(), 2), -2);
        assert_eq!(toy_load_state(std::ptr::null_mut(), [0u8].as_ptr(), 1), -1);
        assert_eq!(toy_load_state(t, std::ptr::null(), 5), -2);
        assert_eq!(toy_save_state_size(t), 3);
        assert_eq!(toy_save_state_size(std::ptr::null()), -1);
        let mut out = [0u8; 3];
        assert_eq!(toy_save_state(t, out.as_mut_ptr(), 3), 3);
        assert_eq!(out, [7, 1, 2]);
        assert_eq!(toy_save_state(t, out.as_mut_ptr(), 2), -7);
        assert_eq!(toy_save_state(t, std::ptr::null_mut(), 3), -1);
        assert_eq!(toy_save_state(std::ptr::null(), out.as_mut_ptr(), 3), -1);
        let whole = "0 1 u8 A\n1 2 u16 B\n";
        assert_eq!(toy_state_layout(t, std::ptr::null_mut(), 0), whole.len() as i64);
        let mut text = [b'#'; 12];
        assert_eq!(toy_state_layout(t, text.as_mut_ptr(), 5), whole.len() as i64);
        assert_eq!(&text[..6], b"0 1 u#");
        assert_eq!(toy_state_layout(std::ptr::null(), text.as_mut_ptr(), 5), -1);
    }
    assert_eq!(toy.pair, Pair { a: 7, b: 0x0201 });
}

#[test]
fn a_result_crosses_as_its_count_or_its_status() {
    assert_eq!(crate::ffi::result::<ToyError>(Ok(12)), 12);
    assert_eq!(crate::ffi::result::<ToyError>(Err(ToyError(-12))), -12);
}
