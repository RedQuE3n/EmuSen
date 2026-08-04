using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.WiseMan.Fixtures;
using Sm83 = EmuSen.Cores.Nintendo.Mercury.Cpu.Core.Cpu;

namespace EmuSen.WiseMan.Cores
{
    // The SM83, which is neither an 8080 nor a Z80 - see Mercury_Cpu.md.
    public class MercuryCpuTests
    {
        private static (MercuryCore Core, Sm83 Cpu, MemoryBus Bus) Machine(params byte[] code)
        {
            var core = new MercuryCore();
            string path = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(patches: (0, code)));

            try { core.LoadRom(path); }
            finally { System.IO.File.Delete(path); }

            core.Cpu!.PC = SyntheticGbRom.EntryPoint;
            return (core, core.Cpu!, core.Bus!);
        }

        private static int StepOne(MercuryCore core)
        {
            int cycles = core.Cpu!.Step(core.Bus!.InterruptEnable, core.Bus.InterruptFlags, out int serviced);
            if (serviced >= 0) core.Bus.InterruptFlags &= (byte)~(1 << serviced);
            core.Bus.Tick(cycles);
            return cycles;
        }

        [Fact]
        public void Reset_lands_on_the_post_boot_rom_state()
        {
            var (_, cpu, _) = Machine(0x00);

            Assert.Equal(0x01, cpu.A);
            Assert.Equal(0xFFFE, cpu.SP);
            Assert.Equal(0x014D, cpu.HL);
        }

        // The low nibble of F is not wired up, however it is written.
        [Fact]
        public void The_flag_register_never_keeps_its_low_nibble()
        {
            var (_, cpu, _) = Machine(0x00);

            cpu.AF = 0x12FF;
            Assert.Equal(0xF0, cpu.F);
        }

        [Fact]
        public void Loading_an_immediate_and_adding_sets_the_half_carry()
        {
            // LD A,$0F ; ADD A,$01
            var (core, cpu, _) = Machine(0x3E, 0x0F, 0xC6, 0x01);

            StepOne(core);
            StepOne(core);

            Assert.Equal(0x10, cpu.A);
            Assert.True((cpu.F & Sm83.FlagH) != 0);
            Assert.False((cpu.F & Sm83.FlagZ) != 0);
        }

        [Fact]
        public void Adding_past_a_byte_sets_carry_and_zero_together()
        {
            // LD A,$FF ; ADD A,$01
            var (core, cpu, _) = Machine(0x3E, 0xFF, 0xC6, 0x01);

            StepOne(core);
            StepOne(core);

            Assert.Equal(0x00, cpu.A);
            Assert.True((cpu.F & Sm83.FlagC) != 0);
            Assert.True((cpu.F & Sm83.FlagZ) != 0);
        }

        // INC must leave carry alone, which is what makes it usable inside a multi-byte add.
        [Fact]
        public void Inc_preserves_the_carry_flag()
        {
            // SCF ; INC B
            var (core, cpu, _) = Machine(0x37, 0x04);

            StepOne(core);
            StepOne(core);

            Assert.True((cpu.F & Sm83.FlagC) != 0);
        }

        [Fact]
        public void Daa_fixes_up_a_bcd_addition()
        {
            // LD A,$09 ; ADD A,$01 ; DAA  -> 09 + 01 is 10 in BCD, not 0A
            var (core, cpu, _) = Machine(0x3E, 0x09, 0xC6, 0x01, 0x27);

            StepOne(core);
            StepOne(core);
            StepOne(core);

            Assert.Equal(0x10, cpu.A);
        }

        // RLCA always clears Z; only the CB-prefixed RLC sets it from the result.
        [Fact]
        public void The_accumulator_rotate_never_sets_zero()
        {
            // XOR A ; RLCA
            var (core, cpu, _) = Machine(0xAF, 0x07);

            StepOne(core);
            Assert.True((cpu.F & Sm83.FlagZ) != 0);

            StepOne(core);
            Assert.Equal(0x00, cpu.A);
            Assert.False((cpu.F & Sm83.FlagZ) != 0);
        }

        [Fact]
        public void The_prefixed_rotate_does_set_zero()
        {
            // XOR A ; RLC A
            var (core, cpu, _) = Machine(0xAF, 0xCB, 0x07);

            StepOne(core);
            StepOne(core);

            Assert.True((cpu.F & Sm83.FlagZ) != 0);
        }

        [Fact]
        public void Bit_tests_without_disturbing_carry()
        {
            // SCF ; LD A,$01 ; BIT 0,A
            var (core, cpu, _) = Machine(0x37, 0x3E, 0x01, 0xCB, 0x47);

            StepOne(core);
            StepOne(core);
            StepOne(core);

            Assert.False((cpu.F & Sm83.FlagZ) != 0);
            Assert.True((cpu.F & Sm83.FlagH) != 0);
            Assert.True((cpu.F & Sm83.FlagC) != 0);
        }

        [Fact]
        public void Swap_exchanges_the_nibbles()
        {
            // LD A,$AB ; SWAP A
            var (core, cpu, _) = Machine(0x3E, 0xAB, 0xCB, 0x37);

            StepOne(core);
            StepOne(core);

            Assert.Equal(0xBA, cpu.A);
        }

        [Fact]
        public void The_hl_post_increment_store_moves_the_pointer()
        {
            // LD HL,$C000 ; LD A,$77 ; LD (HL+),A
            var (core, cpu, bus) = Machine(0x21, 0x00, 0xC0, 0x3E, 0x77, 0x22);

            StepOne(core);
            StepOne(core);
            StepOne(core);

            Assert.Equal(0x77, bus.Read(0xC000));
            Assert.Equal(0xC001, cpu.HL);
        }

        [Fact]
        public void A_taken_branch_costs_more_than_an_untaken_one()
        {
            // XOR A (sets Z) ; JR NZ,+2
            var (core, _, _) = Machine(0xAF, 0x20, 0x02);

            StepOne(core);
            Assert.Equal(8, StepOne(core));

            // SCF ; JR C,+2
            var (taken, _, _) = Machine(0x37, 0x38, 0x02);
            StepOne(taken);
            Assert.Equal(12, StepOne(taken));
        }

        [Fact]
        public void Call_pushes_the_return_address_and_ret_takes_it_back()
        {
            // CALL $0160
            var (core, cpu, _) = Machine(0xCD, 0x60, 0x01);
            ushort expectedReturn = (ushort)(SyntheticGbRom.EntryPoint + 3);

            StepOne(core);

            Assert.Equal(0x0160, cpu.PC);
            Assert.Equal(0xFFFC, cpu.SP);

            cpu.PC = 0x0160;
            core.Bus!.Write(0x0160, 0xC9);   // RET, into RAM-free ROM space is fine for a read
            Assert.Equal(expectedReturn, ReadStackWord(core));
        }

        private static ushort ReadStackWord(MercuryCore core) =>
            (ushort)(core.Bus!.Read(core.Cpu!.SP) | (core.Bus.Read((ushort)(core.Cpu.SP + 1)) << 8));

        [Fact]
        public void Echo_ram_is_the_same_storage_as_work_ram()
        {
            var (_, _, bus) = Machine(0x00);

            bus.Write(0xC005, 0x5A);
            Assert.Equal(0x5A, bus.Read(0xE005));

            bus.Write(0xE105, 0xA5);
            Assert.Equal(0xA5, bus.Read(0xC105));
        }

        [Fact]
        public void An_enabled_interrupt_vectors_and_clears_its_own_flag()
        {
            var (core, cpu, bus) = Machine(0xFB, 0x00, 0x00);   // EI ; NOP ; NOP

            StepOne(core);   // EI
            StepOne(core);   // the instruction EI's delay lets through

            bus.InterruptEnable = 0x01;
            bus.Request(Interrupt.VBlank);
            StepOne(core);

            Assert.Equal(0x0040, cpu.PC);
            Assert.False(cpu.Ime);
            Assert.Equal(0x00, bus.InterruptFlags & 0x01);
        }

        // EI must not take effect until after the instruction that follows it.
        [Fact]
        public void Ei_is_delayed_by_one_instruction()
        {
            var (core, cpu, _) = Machine(0xFB, 0x00);

            StepOne(core);
            Assert.False(cpu.Ime);

            StepOne(core);
            Assert.True(cpu.Ime);
        }

        [Fact]
        public void Halt_wakes_on_a_pending_interrupt_even_with_interrupts_disabled()
        {
            var (core, cpu, bus) = Machine(0xF3, 0x76, 0x00);   // DI ; HALT ; NOP

            StepOne(core);
            StepOne(core);
            Assert.True(cpu.Halted);

            bus.InterruptEnable = 0x01;
            bus.Request(Interrupt.VBlank);
            StepOne(core);

            Assert.False(cpu.Halted);
            Assert.NotEqual(0x0040, cpu.PC);
        }

        [Fact]
        public void An_unimplemented_opcode_is_reported_rather_than_silently_skipped()
        {
            var (core, _, _) = Machine(0xD3);
            Assert.Throws<System.NotSupportedException>(() => StepOne(core));
        }
    }
}
