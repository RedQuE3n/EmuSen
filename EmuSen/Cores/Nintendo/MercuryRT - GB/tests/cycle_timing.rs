//! WiseMan's `MercuryCycleTimingTests` on the Rust SM83: the CPU against a bus that records the order it is touched in, so
//! the machine is seen to run during an instruction and not after it. The C# claims drive C#'s `Cpu` through a fake
//! `ICpuBus`, which a WiseMan test cannot hand MercuryRT; the same claims run here through `CpuBus`. See Mercury_Cpu.md §6
//! and Mercury_Native.md §12.1.

use mercuryrt::cpu::{Cpu, CpuBus};

/// Every access and every tick in order, as `R:`, `W:` and `T:` lines, over 64K of flat memory.
struct RecordingBus {
    trace: Vec<String>,
    memory: Vec<u8>,
}

impl CpuBus for RecordingBus {
    fn read(&mut self, address: u16) -> u8 {
        self.trace.push(format!("R:{address:04X}"));
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, data: u8) {
        self.trace.push(format!("W:{address:04X}"));
        self.memory[address as usize] = data;
    }

    fn tick(&mut self, cycles: i32) {
        self.trace.push(format!("T:{cycles}"));
    }

    fn stop(&mut self) {}

    fn pending_interrupts(&self) -> u8 {
        self.memory[0xFFFF] & self.memory[0xFF0F] & 0x1F
    }
}

impl RecordingBus {
    fn ticked(&self) -> i32 {
        self.trace.iter().filter_map(|t| t.strip_prefix("T:")).map(|n| n.parse::<i32>().unwrap()).sum()
    }
}

/// The program at $0100 on a Game Boy's reset registers.
fn machine(program: &[u8]) -> (Cpu, RecordingBus) {
    let mut bus = RecordingBus { trace: Vec::new(), memory: vec![0; 0x10000] };
    bus.memory[0x100..0x100 + program.len()].copy_from_slice(program);
    let mut cpu = Cpu::default();
    cpu.reset(false);
    (cpu, bus)
}

#[test]
fn each_memory_access_is_preceded_by_its_own_machine_cycle() {
    let (mut cpu, mut bus) = machine(&[0xFA, 0x23, 0xC1]);
    let (cycles, _) = cpu.step(&mut bus, 0, 0).unwrap();
    assert_eq!(cycles, 16);
    assert_eq!(bus.trace, ["T:4", "R:0100", "T:4", "R:0101", "T:4", "R:0102", "T:4", "R:C123"]);
}

#[test]
fn internal_cycles_settle_at_the_end_of_the_instruction() {
    let (mut cpu, mut bus) = machine(&[0xC9]);
    let (cycles, _) = cpu.step(&mut bus, 0, 0).unwrap();
    assert_eq!(cycles, 16);
    assert_eq!(bus.trace, ["T:4", "R:0100", "T:4", "R:FFFE", "T:4", "R:FFFF", "T:4"]);
}

#[test]
fn a_read_modify_write_ticks_between_its_read_and_its_write() {
    let (mut cpu, mut bus) = machine(&[0x34]);
    let (cycles, _) = cpu.step(&mut bus, 0, 0).unwrap();
    assert_eq!(cycles, 12);
    assert_eq!(bus.trace, ["T:4", "R:0100", "T:4", "R:014D", "T:4", "W:014D"]);
}

#[test]
fn the_bus_advances_by_exactly_what_the_step_reports() {
    for opcode in [0x00, 0x34, 0xC5, 0xE1, 0xFA, 0xCD, 0x18, 0xE8, 0xF0, 0x76, 0x10] {
        let (mut cpu, mut bus) = machine(&[opcode, 0x04, 0x01]);
        let (cycles, _) = cpu.step(&mut bus, 0, 0).unwrap();
        assert_eq!(cycles, bus.ticked(), "opcode ${opcode:02X}");
    }
}

#[test]
fn servicing_an_interrupt_ticks_its_full_cost() {
    let (mut cpu, mut bus) = machine(&[0x00]);
    (bus.memory[0xFFFF], bus.memory[0xFF0F]) = (0x01, 0x01);
    cpu.ime = true;
    let (cycles, serviced) = cpu.step(&mut bus, 0x01, 0x01).unwrap();
    assert_eq!(serviced, 0);
    assert_eq!(cycles, 20);
    assert_eq!(bus.ticked(), 20);
    assert_eq!(cpu.pc, 0x0040);
}

/// Pan Docs' dispatch: two wait states, the push on the third and fourth machine cycles, the jump on the fifth (Mercury_Native.md §13.3).
#[test]
fn a_dispatch_pushes_on_its_third_and_fourth_machine_cycles() {
    let (mut cpu, mut bus) = machine(&[0x00]);
    (bus.memory[0xFFFF], bus.memory[0xFF0F]) = (0x01, 0x01);
    cpu.ime = true;
    cpu.step(&mut bus, 0x01, 0x01).unwrap();
    let mut sum = 0;
    let mut before = Vec::new();
    for t in &bus.trace {
        match t.strip_prefix("T:") {
            Some(n) => sum += n.parse::<i32>().unwrap(),
            None => before.push(sum),
        }
    }
    assert_eq!(before, [12, 16]);
    assert_eq!(bus.ticked(), 20);
}

/// Where each access lands, as the T-cycles ticked before it, then the total.
fn access_times(bus: &RecordingBus) -> Vec<i32> {
    let mut sum = 0;
    let mut times = Vec::new();
    for t in &bus.trace {
        match t.strip_prefix("T:") {
            Some(n) => sum += n.parse::<i32>().unwrap(),
            None => times.push(sum),
        }
    }
    times.push(sum);
    times
}

/// mooneye's *_timing headers and GBCTR: the internal cycle that decrements SP comes before a push's writes, and RET cc's
/// condition cycle before its pops (Mercury_Native.md §13.5).
#[test]
fn each_stack_access_lands_on_the_machine_cycle_the_hardware_gives_it() {
    let cases: [(&[u8], &[i32]); 5] = [
        (&[0xC5], &[4, 12, 16, 16]),
        (&[0xCF], &[4, 12, 16, 16]),
        (&[0xCD, 0x34, 0x12], &[4, 8, 12, 20, 24, 24]),
        (&[0xCC, 0x34, 0x12], &[4, 8, 12, 20, 24, 24]),
        (&[0xC8], &[4, 12, 16, 20]),
    ];
    for (program, times) in cases {
        let (mut cpu, mut bus) = machine(program);
        cpu.step(&mut bus, 0, 0).unwrap();
        assert_eq!(access_times(&bus), times, "opcode ${:02X}", program[0]);
    }
}

#[test]
fn a_halted_cpu_still_advances_the_bus() {
    let (mut cpu, mut bus) = machine(&[0x00]);
    cpu.halted = true;
    let (cycles, _) = cpu.step(&mut bus, 0, 0).unwrap();
    assert_eq!(cycles, 4);
    assert_eq!(bus.trace, ["T:4"]);
}
