//! The registers psx-spx's "Parameters, Registers, Stack" says a call leaves unchanged (R16-R23, SP, FP), and GP,
//! which games address their small data through and the kernel must never disturb (Diamond_OpenKernel.md §6).

use diamond_kernel_harness::image;
use diamond_kernel_harness::machine::{Machine, STACK};

#[test]
fn every_implemented_call_that_returns_leaves_gp_and_the_saved_registers_unchanged() {
    let calls = diamond_kernel_harness::calls::parse(&diamond_kernel_harness::calls::source());
    let mut m = Machine::boot(&image::committed());
    m.a(0x39, &[0x8012_0000, 0x4000]);
    m.code(0x8015_0000, "jr ra\naddiu v0, zero, 0");
    m.text(0x8010_0000, "tty:");
    // Calls that do not return to their caller, or whose generic arguments would run for long or wreck the kernel's
    // RAM (an element count of 80100100h, a DCB copied from text); SetConf is checked with real arguments below.
    let skip = [
        ('A', 0x06), ('A', 0x14), ('A', 0x31), ('A', 0x3D), ('A', 0x3F), ('A', 0x43), ('A', 0x46), ('A', 0x4A), ('A', 0x51),
        ('A', 0x9C), ('A', 0xB2), ('B', 0x10), ('B', 0x12), ('B', 0x17), ('B', 0x19), ('B', 0x38), ('B', 0x3E),
        ('B', 0x47), ('C', 0x06), ('C', 0x19),
    ];
    let mut real = Machine::boot(&image::committed());
    real.cpu.regs.r[28] = 0x2828_2828;
    real.a(0x9C, &[0x10, 4, 0x801F_FF00]);
    assert_eq!(real.cpu.regs.r[28], 0x2828_2828, "A(9Ch) SetConf: GP");
    for c in calls.iter().filter(|c| c.status == 'D' && !skip.contains(&(c.table, c.number))) {
        let r = &mut m.cpu.regs;
        for (i, v) in (16..24).zip(0x1600_0000..) {
            r.r[i] = v;
        }
        r.r[28] = 0x2828_2828;
        r.r[30] = 0x3030_3030;
        let args = [0x8010_0000, 0x8010_0100, 4, 0x8015_0000, 0x8015_0000, 0, 0, 0];
        let vector = match c.table {
            'A' => 0xA0,
            'B' => 0xB0,
            _ => 0xC0,
        };
        m.call(vector, c.number, &args);
        let r = &m.cpu.regs;
        let saved: Vec<u32> = (16..24).map(|i| r.r[i]).collect();
        assert_eq!(saved, (0x1600_0000..0x1600_0008).collect::<Vec<u32>>(), "{}({:02X}h) {}: S0-S7", c.table, c.number, c.name);
        assert_eq!((r.r[28], r.r[29], r.r[30]), (0x2828_2828, STACK, 0x3030_3030), "{}({:02X}h) {}: GP, SP, FP", c.table, c.number, c.name);
        m.b(0x09, &[0xF100_0000]);
    }
}
