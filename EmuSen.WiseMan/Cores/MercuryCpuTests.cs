using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The SM83, which is neither an 8080 nor a Z80 - see Mercury_Cpu.md; held on MercuryRT - see Mercury_Native.md §12, §15.
    public sealed class MercuryRtCpuTests : IDisposable
    {
        private readonly List<MercuryRig> _rigs = new();


        public void Dispose()
        {
            foreach (MercuryRig rig in _rigs) rig.Dispose();
        }

        // The program at the entry point, the CPU parked in front of it.
        private MercuryRig Machine(params byte[] code)
        {
            var core = MercuryRig.Create(SyntheticGbRom.Build(patches: (0, code)));
            _rigs.Add(core);
            core.SetField("Cpu.PC", SyntheticGbRom.EntryPoint);
            return core;
        }

        private static int StepOne(MercuryRig core) => core.Step();

        [Fact]
        public void Reset_lands_on_the_post_boot_rom_state()
        {
            var core = Machine(0x00);

            Assert.Equal(0x01, core.Field("Cpu.A"));
            Assert.Equal(0xFFFE, core.Field("Cpu.SP"));
            Assert.Equal(0x014D, core.Pair("HL"));
        }

        // The low nibble of F is not wired up, however it is written: here by POP AF.
        [Fact]
        public void The_flag_register_never_keeps_its_low_nibble()
        {
            // LD BC,$12FF ; PUSH BC ; POP AF
            var core = Machine(0x01, 0xFF, 0x12, 0xC5, 0xF1);

            StepOne(core);
            StepOne(core);
            StepOne(core);

            Assert.Equal(0x12, core.Field("Cpu.A"));
            Assert.Equal(0xF0, core.Field("Cpu.F"));
        }

        [Fact]
        public void Loading_an_immediate_and_adding_sets_the_half_carry()
        {
            // LD A,$0F ; ADD A,$01
            var core = Machine(0x3E, 0x0F, 0xC6, 0x01);

            StepOne(core);
            StepOne(core);

            Assert.Equal(0x10, core.Field("Cpu.A"));
            Assert.True((core.Field("Cpu.F") & MercuryRig.FlagH) != 0);
            Assert.False((core.Field("Cpu.F") & MercuryRig.FlagZ) != 0);
        }

        [Fact]
        public void Adding_past_a_byte_sets_carry_and_zero_together()
        {
            // LD A,$FF ; ADD A,$01
            var core = Machine(0x3E, 0xFF, 0xC6, 0x01);

            StepOne(core);
            StepOne(core);

            Assert.Equal(0x00, core.Field("Cpu.A"));
            Assert.True((core.Field("Cpu.F") & MercuryRig.FlagC) != 0);
            Assert.True((core.Field("Cpu.F") & MercuryRig.FlagZ) != 0);
        }

        // INC must leave carry alone, which is what makes it usable inside a multi-byte add.
        [Fact]
        public void Inc_preserves_the_carry_flag()
        {
            // SCF ; INC B
            var core = Machine(0x37, 0x04);

            StepOne(core);
            StepOne(core);

            Assert.True((core.Field("Cpu.F") & MercuryRig.FlagC) != 0);
        }

        [Fact]
        public void Daa_fixes_up_a_bcd_addition()
        {
            // LD A,$09 ; ADD A,$01 ; DAA  -> 09 + 01 is 10 in BCD, not 0A
            var core = Machine(0x3E, 0x09, 0xC6, 0x01, 0x27);

            StepOne(core);
            StepOne(core);
            StepOne(core);

            Assert.Equal(0x10, core.Field("Cpu.A"));
        }

        // RLCA always clears Z; only the CB-prefixed RLC sets it from the result.
        [Fact]
        public void The_accumulator_rotate_never_sets_zero()
        {
            // XOR A ; RLCA
            var core = Machine(0xAF, 0x07);

            StepOne(core);
            Assert.True((core.Field("Cpu.F") & MercuryRig.FlagZ) != 0);

            StepOne(core);
            Assert.Equal(0x00, core.Field("Cpu.A"));
            Assert.False((core.Field("Cpu.F") & MercuryRig.FlagZ) != 0);
        }

        [Fact]
        public void The_prefixed_rotate_does_set_zero()
        {
            // XOR A ; RLC A
            var core = Machine(0xAF, 0xCB, 0x07);

            StepOne(core);
            StepOne(core);

            Assert.True((core.Field("Cpu.F") & MercuryRig.FlagZ) != 0);
        }

        [Fact]
        public void Bit_tests_without_disturbing_carry()
        {
            // SCF ; LD A,$01 ; BIT 0,A
            var core = Machine(0x37, 0x3E, 0x01, 0xCB, 0x47);

            StepOne(core);
            StepOne(core);
            StepOne(core);

            Assert.False((core.Field("Cpu.F") & MercuryRig.FlagZ) != 0);
            Assert.True((core.Field("Cpu.F") & MercuryRig.FlagH) != 0);
            Assert.True((core.Field("Cpu.F") & MercuryRig.FlagC) != 0);
        }

        [Fact]
        public void Swap_exchanges_the_nibbles()
        {
            // LD A,$AB ; SWAP A
            var core = Machine(0x3E, 0xAB, 0xCB, 0x37);

            StepOne(core);
            StepOne(core);

            Assert.Equal(0xBA, core.Field("Cpu.A"));
        }

        [Fact]
        public void The_hl_post_increment_store_moves_the_pointer()
        {
            // LD HL,$C000 ; LD A,$77 ; LD (HL+),A
            var core = Machine(0x21, 0x00, 0xC0, 0x3E, 0x77, 0x22);

            StepOne(core);
            StepOne(core);
            StepOne(core);

            Assert.Equal(0x77, core.Read(0xC000));
            Assert.Equal(0xC001, core.Pair("HL"));
        }

        [Fact]
        public void A_taken_branch_costs_more_than_an_untaken_one()
        {
            // XOR A (sets Z) ; JR NZ,+2
            var core = Machine(0xAF, 0x20, 0x02);

            StepOne(core);
            Assert.Equal(8, StepOne(core));

            // SCF ; JR C,+2
            var taken = Machine(0x37, 0x38, 0x02);
            StepOne(taken);
            Assert.Equal(12, StepOne(taken));
        }

        [Fact]
        public void Call_pushes_the_return_address_and_ret_takes_it_back()
        {
            // CALL $0160
            var core = Machine(0xCD, 0x60, 0x01);
            ushort expectedReturn = (ushort)(SyntheticGbRom.EntryPoint + 3);

            StepOne(core);

            Assert.Equal(0x0160, core.Field("Cpu.PC"));
            Assert.Equal(0xFFFC, core.Field("Cpu.SP"));

            core.SetField("Cpu.PC", 0x0160);
            core.Write(0x0160, 0xC9);   // RET, into RAM-free ROM space is fine for a read
            Assert.Equal(expectedReturn, ReadStackWord(core));
        }

        private static ushort ReadStackWord(MercuryRig core) =>
            (ushort)(core.Read((int)core.Field("Cpu.SP")) | (core.Read((int)core.Field("Cpu.SP") + 1) << 8));

        [Fact]
        public void Echo_ram_is_the_same_storage_as_work_ram()
        {
            var core = Machine(0x00);

            core.Write(0xC005, 0x5A);
            Assert.Equal(0x5A, core.Read(0xE005));

            core.Write(0xE105, 0xA5);
            Assert.Equal(0xA5, core.Read(0xC105));
        }

        [Fact]
        public void An_enabled_interrupt_vectors_and_clears_its_own_flag()
        {
            var core = Machine(0xFB, 0x00, 0x00);   // EI ; NOP ; NOP

            StepOne(core);   // EI
            StepOne(core);   // the instruction EI's delay lets through

            core.SetField("Bus.InterruptEnable", 0x01);
            core.Request(MercuryRig.VBlank);
            StepOne(core);

            Assert.Equal(0x0040, core.Field("Cpu.PC"));
            Assert.False(core.Flag("Cpu.Ime"));
            Assert.Equal(0x00, core.Field("Bus.InterruptFlags") & 0x01);
        }

        // EI must not take effect until after the instruction that follows it.
        [Fact]
        public void Ei_is_delayed_by_one_instruction()
        {
            var core = Machine(0xFB, 0x00);

            StepOne(core);
            Assert.False(core.Flag("Cpu.Ime"));

            StepOne(core);
            Assert.True(core.Flag("Cpu.Ime"));
        }

        [Fact]
        public void Halt_wakes_on_a_pending_interrupt_even_with_interrupts_disabled()
        {
            var core = Machine(0xF3, 0x76, 0x00);   // DI ; HALT ; NOP

            StepOne(core);
            StepOne(core);
            Assert.True(core.Flag("Cpu.Halted"));

            core.SetField("Bus.InterruptEnable", 0x01);
            core.Request(MercuryRig.VBlank);
            StepOne(core);

            Assert.False(core.Flag("Cpu.Halted"));
            Assert.NotEqual(0x0040, core.Field("Cpu.PC"));
        }

        [Fact]
        public void An_unimplemented_opcode_is_reported_rather_than_silently_skipped()
        {
            var core = Machine(0xD3);
            Assert.Throws<System.NotSupportedException>(() => StepOne(core));
        }
    }
}
