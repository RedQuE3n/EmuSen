using System;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The signal processor's scalar half, which is MIPS with the hard parts removed - see Mars_Rsp.md.
    public class MarsRspTests
    {
        [Fact]
        public void The_program_counter_keeps_only_the_bits_instruction_memory_has()
        {
            var bus = new MemoryBus();

            bus.Write32(MemoryMap.SpPcBase, 0xFFFF_FFFF);

            Assert.Equal(0xFFCu, bus.Read32(MemoryMap.SpPcBase));
        }

        // The whole life of a program: it starts halted, runs, breaks, and says so - see §5.1.
        [Fact]
        public void A_program_runs_until_it_breaks_and_leaves_the_counter_past_the_break()
        {
            var bus = Loaded(a => a.Nop().Break());

            Assert.Equal(SpInterface.StatusHalt, bus.Sp.StatusWord);

            Run(bus);

            Assert.Equal(0x8u, bus.Sp.Pc);
            Assert.Equal(SpInterface.StatusHalt | SpInterface.StatusBroke, bus.Sp.StatusWord);

            bus.Write32(MemoryMap.SpRegistersBase + SpInterface.Status, 0x04);

            Assert.Equal(SpInterface.StatusHalt, bus.Sp.StatusWord);
        }

        // A break in a delay slot still lets the branch decide where the counter stops - see §2.1.
        [Theory]
        [InlineData(true, 0x1Cu)]
        [InlineData(false, 0x8u)]
        public void A_break_in_a_delay_slot_reports_where_the_branch_sent_it(bool taken, uint expected)
        {
            var bus = Loaded(a => (taken ? a.Beq(0, 0, 6) : a.Bne(0, 0, 6)).Break());

            Run(bus);

            Assert.Equal(expected, bus.Sp.Pc);
        }

        // Instruction memory is four kilobytes and the counter wraps inside it rather than leaving - see §2.
        [Fact]
        public void The_counter_wraps_at_the_end_of_instruction_memory()
        {
            var bus = new MemoryBus();

            Load(bus, 0xFF8, a => a.Nop().Nop());
            Load(bus, 0, a => a.Break());

            Run(bus, 0xFF8);

            Assert.Equal(0x4u, bus.Sp.Pc);
        }

        // The corpus's own vectors: neither addition traps, so the signed and unsigned forms agree - see §3.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void Addition_never_traps_on_overflow_whichever_form_is_written(bool unsigned)
        {
            var bus = Loaded(a => a
                .Lui(8, 0x1234).Ori(8, 8, 0x5678)
                .Lui(9, 0xFFFF).Ori(9, 9, 0xEDCB)
                .Word(Add(16, 8, 9, unsigned))
                .Sw(16, 0, 0)
                .Break());

            Run(bus);

            Assert.Equal(0x1234_4443u, bus.Read32(MemoryMap.SpDmemBase));
        }

        [Fact]
        public void A_write_to_register_zero_is_dropped_here_too()
        {
            var bus = Loaded(a => a.Lui(1, 0x1234).Word(Add(0, 1, 1, unsigned: true)).Sw(0, 0, 0x20).Break());

            Run(bus);

            Assert.Equal(0u, bus.Read32(MemoryMap.SpDmemBase + 0x20));
        }

        // The corpus's own load table, whose point is that none of these addresses is an error - see §4.
        [Theory]
        [InlineData(0x000, 0xBADD_ECAFu)]
        [InlineData(0x001, 0xDDEC_AF01u)]
        [InlineData(0x003, 0xAF01_2345u)]
        [InlineData(0x006, 0x4567_0000u)]
        [InlineData(0xFFC, 0xBCAD_7E8Fu)]
        [InlineData(0xFFD, 0xAD7E_8FBAu)]
        [InlineData(0xFFE, 0x7E8F_BADDu)]
        public void An_unaligned_load_takes_its_bytes_in_order_and_wraps(short offset, uint expected)
        {
            var bus = new MemoryBus();

            bus.Write32(MemoryMap.SpDmemBase + 0x000, 0xBADD_ECAF);
            bus.Write32(MemoryMap.SpDmemBase + 0x004, 0x0123_4567);
            bus.Write32(MemoryMap.SpDmemBase + 0x008, 0x0000_0000);
            bus.Write32(MemoryMap.SpDmemBase + 0xFFC, 0xBCAD_7E8F);

            Load(bus, 0, a => a.Lw(16, 0, offset).Sw(16, 0, 0x10).Break());
            Run(bus);

            Assert.Equal(expected, bus.Read32(MemoryMap.SpDmemBase + 0x10));
        }

        // An address is twelve bits wide, so everything above them is thrown away rather than faulting.
        [Theory]
        [InlineData(0x1FFC)]
        [InlineData(0x7FFC)]
        public void An_address_keeps_only_the_bits_data_memory_has(short offset)
        {
            var bus = new MemoryBus();

            bus.Write32(MemoryMap.SpDmemBase + 0xFFC, 0xFEED_FACE);

            Load(bus, 0, a => a.Lw(16, 0, offset).Sw(16, 0, 0x10).Break());
            Run(bus);

            Assert.Equal(0xFEED_FACEu, bus.Read32(MemoryMap.SpDmemBase + 0x10));
        }

        // Coprocessor zero from this side is the interface registers, and the signals are how it talks - see §5.
        [Fact]
        public void The_processor_can_raise_a_signal_the_main_processor_reads()
        {
            var bus = Loaded(a => a
                .Lui(4, 0x0000).Ori(4, 4, 1 << 10)
                .Word(Mtc0(4, 4))
                .Break());

            Run(bus);

            Assert.Equal(1u << 7, bus.Sp.StatusWord & (1u << 7));
        }

        [Fact]
        public void The_processor_can_read_the_status_the_main_processor_wrote()
        {
            var bus = Loaded(a => a.Word(Mfc0(16, 4)).Sw(16, 0, 0).Break());

            bus.Write32(MemoryMap.SpRegistersBase + SpInterface.Status, 1u << 12);
            Run(bus);

            Assert.Equal(1u << 8, bus.Read32(MemoryMap.SpDmemBase) & (1u << 8));
        }

        // Halting is a write away, and the counter stops where it was rather than resetting - see §5.1.
        [Fact]
        public void The_main_processor_can_halt_a_running_program()
        {
            var bus = Loaded(a => a.Nop().Nop().Nop().Nop().Nop().Break());

            bus.Write32(MemoryMap.SpPcBase, 0);
            bus.Write32(MemoryMap.SpRegistersBase + SpInterface.Status, 0x01);
            bus.Tick(2);
            bus.Write32(MemoryMap.SpRegistersBase + SpInterface.Status, 0x02);
            bus.Tick(100);

            Assert.Equal(SpInterface.StatusHalt, bus.Sp.StatusWord & SpInterface.StatusHalt);
            Assert.Equal(0u, bus.Sp.StatusWord & SpInterface.StatusBroke);
            Assert.Equal(0x8u, bus.Sp.Pc);
        }

        // Each field has a clear bit and a set bit, and naming both leaves it alone - see Mars_Rsp.md §5.1.
        [Theory]
        [InlineData(0, 9)]
        [InlineData(7, 23)]
        public void A_write_naming_both_the_clear_and_the_set_bit_changes_nothing(int signal, int clearBit)
        {
            var bus = new MemoryBus();

            Write(bus, 1u << (clearBit + 1));
            Write(bus, (1u << clearBit) | (1u << (clearBit + 1)));

            uint bit = 1u << (SpInterface.SignalShift + signal);

            Assert.Equal(bit, bus.Sp.StatusWord & bit);

            Write(bus, 1u << clearBit);
            Write(bus, (1u << clearBit) | (1u << (clearBit + 1)));

            Assert.Equal(0u, bus.Sp.StatusWord & bit);
        }

        [Fact]
        public void The_rule_holds_for_the_interrupt_on_break_bit_as_well()
        {
            var bus = new MemoryBus();

            Write(bus, 0x100);
            Write(bus, 0x180);

            Assert.Equal(SpInterface.StatusInterruptOnBreak, bus.Sp.StatusWord & SpInterface.StatusInterruptOnBreak);

            Write(bus, 0x080);
            Write(bus, 0x180);

            Assert.Equal(0u, bus.Sp.StatusWord & SpInterface.StatusInterruptOnBreak);
        }

        // What the FPGA core, Project64 and mupen64plus all do: the break raises the interrupt only when asked to - see Mars_Performance.md §15.
        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void A_break_raises_the_interrupt_only_when_interrupt_on_break_is_set(bool asked)
        {
            var bus = Loaded(a => a.Nop().Break());
            if (asked) Write(bus, 0x100);

            Run(bus);

            Assert.Equal(SpInterface.StatusBroke, bus.Sp.StatusWord & SpInterface.StatusBroke);
            Assert.Equal(asked, bus.Mi.Pending.HasFlag(MiInterrupt.SignalProcessor));
        }

        // A jump-and-link whose link register is its own target must read the target first - see §2.1.
        [Fact]
        public void A_jump_and_link_register_reads_its_target_before_it_writes_the_link()
        {
            var bus = Loaded(a => a
                .Addiu(4, 0, 0x10)
                .Word(Jalr(4, 4))
                .Nop()
                .Addiu(16, 0, 1)
                .Sw(16, 0, 0)
                .Sw(4, 0, 4)
                .Break());

            Run(bus);

            Assert.Equal(0u, bus.Read32(MemoryMap.SpDmemBase));
            Assert.Equal(0x0Cu, bus.Read32(MemoryMap.SpDmemBase + 4));
        }

        private static void Write(MemoryBus bus, uint value) =>
            bus.Write32(MemoryMap.SpRegistersBase + SpInterface.Status, value);

        private static uint Jalr(int rd, int rs) =>
            ((uint)rs << 21) | ((uint)rd << 11) | 0x09;

        private static uint Add(int rd, int rs, int rt, bool unsigned) =>
            ((uint)rs << 21) | ((uint)rt << 16) | ((uint)rd << 11) | (unsigned ? 0x21u : 0x20u);

        private static uint Mfc0(int rt, int rd) => (0x10u << 26) | ((uint)rt << 16) | ((uint)rd << 11);

        private static uint Mtc0(int rt, int rd) => (0x10u << 26) | (4u << 21) | ((uint)rt << 16) | ((uint)rd << 11);

        private static MemoryBus Loaded(Func<MipsAssembler, MipsAssembler> program)
        {
            var bus = new MemoryBus();

            Load(bus, 0, program);
            return bus;
        }

        private static void Load(MemoryBus bus, uint offset, Func<MipsAssembler, MipsAssembler> program)
        {
            uint[] words = program(new MipsAssembler()).ToArray();
            for (int i = 0; i < words.Length; i++) bus.Write32(MemoryMap.SpImemBase + offset + (uint)(i * 4), words[i]);
        }

        // Start, then let the machine's own clock carry the program to its break.
        private static void Run(MemoryBus bus, uint pc = 0)
        {
            bus.Write32(MemoryMap.SpPcBase, pc);
            bus.Write32(MemoryMap.SpRegistersBase + SpInterface.Status, 0x01);
            bus.Tick(10_000);
        }
    }
}
