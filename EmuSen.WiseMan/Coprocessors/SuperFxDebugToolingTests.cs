using System.Linq;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Coprocessors
{
    // The debug toolchain's coverage of the SuperFX: the chip's own program
    // space, the disassembler that decodes it, and whole-run coverage of its
    // instruction stream - see Venus_SuperFX.md §8.1 and §8.2.
    public class SuperFxDebugToolingTests
    {
        private static (VenusCore Core, SnesDebugTarget Target) GsuTarget(params (int Offset, byte[] Bytes)[] patches)
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildSuperFx(patches));
            var target = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!);
            return (core, target);
        }

        private static string Text(EmuSen.DianaOS.DianaOS.Lib.DisassembledInstruction i)
            => i.OperandText.Length == 0 ? i.Mnemonic : $"{i.Mnemonic} {i.OperandText}";

        private static string[] Disassemble(int count, params byte[] program)
        {
            var rom = new byte[0x10000];
            program.CopyTo(rom, 0);
            return GsuDisassembler.Disassemble(a => rom[a & 0xFFFF], 0, count).Select(Text).ToArray();
        }

        // --- The GSUBUS space (§8.1) ---

        [Fact]
        public void A_superfx_cartridge_exposes_the_gsu_program_space()
        {
            var (_, target) = GsuTarget();
            var bus = target.GetMemorySpaces().Single(s => s.Name == "GSUBUS");
            Assert.Equal(0x1000000, bus.Size);
        }

        // Game Pak RAM is reachable at $70+ on the GSU's side, which is where
        // the S-CPU stages code for it - see Venus_SuperFX.md §5.1.
        [Fact]
        public void The_gsu_bus_space_reaches_both_rom_and_game_pak_ram()
        {
            var (core, target) = GsuTarget((0x1234, new byte[] { 0xA5 }));
            var gsu = core.Cart!.SuperFx!;
            gsu.DebugRam[0x40] = 0x5C;

            var bus = target.GetMemorySpaces().Single(s => s.Name == "GSUBUS");

            // Bank $00 is LoROM-style, so $00:9234 is ROM offset $1234.
            Assert.Equal(0xA5, bus.Read(0x009234));
            Assert.Equal(0x5C, bus.Read(0x700040));
        }

        // A debugger read must not fill a cache line - see §8.1.
        [Fact]
        public void Reading_the_gsu_bus_space_does_not_disturb_the_instruction_cache()
        {
            var (core, target) = GsuTarget((0x0000, new byte[] { 0xA1, 0x11 }));
            var gsu = core.Cart!.SuperFx!;
            var bus = target.GetMemorySpaces().Single(s => s.Name == "GSUBUS");

            for (int i = 0; i < 32; i++) bus.Read(i);

            Assert.All(gsu.Cache, b => Assert.Equal(0, b));
        }

        // --- The disassembler (§8.1) ---

        [Fact]
        public void The_disassembler_decodes_immediates_and_their_operand_widths()
        {
            Assert.Equal(
                new[] { "IWT R1,#$1234", "IBT R3,#$FE", "STOP" },
                Disassemble(3, 0xF1, 0x34, 0x12, 0xA3, 0xFE, 0x00));
        }

        // The whole reason a static per-opcode table cannot do this job: the
        // same byte is a different instruction under a different prefix.
        [Fact]
        public void An_alt1_prefix_changes_the_next_instructions_mnemonic()
        {
            Assert.Equal(new[] { "LDW (R1)" }, Disassemble(1, 0x41));
            Assert.Equal(new[] { "ALT1", "LDB (R1)" }, Disassemble(2, 0x3D, 0x41));
        }

        // WITH turns the $10-$1F and $B0-$BF slots from prefixes into real
        // instructions - see Venus_SuperFX.md §4.2.
        [Fact]
        public void The_with_prefix_turns_to_and_from_into_move_and_moves()
        {
            Assert.Equal(new[] { "TO R5", "FROM R6" }, Disassemble(2, 0x15, 0xB6));
            Assert.Equal(new[] { "WITH R2", "MOVE R5,R2" }, Disassemble(2, 0x22, 0x15));
        }

        // §10.4's fix, restated as a decode: the prefix has to survive the
        // branch or the delay slot decodes as AND R15 instead of AND #15.
        [Fact]
        public void A_prefix_survives_a_branch_into_its_delay_slot()
        {
            Assert.Equal(
                // The real bytes from Yoshi's Island's $0A:8146; the
                // displacement is -69, so from $0003 it wraps to $FFC0.
                new[] { "FROM R6", "TO R5", "ALT2", "BRA $FFC0", "AND #15" },
                Disassemble(5, 0xB6, 0x15, 0x3E, 0x05, 0xBB, 0x7F));
        }

        // ...and every other instruction has to clear it, or every later line
        // decodes under a prefix that is long gone.
        [Fact]
        public void A_non_branch_instruction_clears_the_prefix_for_the_next_line()
        {
            Assert.Equal(
                new[] { "ALT1", "LDB (R1)", "LDW (R1)" },
                Disassemble(3, 0x3D, 0x41, 0x41));
        }

        // A branch displacement is relative to the delay slot, not to the
        // branch - see Venus_SuperFX.md §4.1.
        [Fact]
        public void A_branch_target_is_relative_to_its_delay_slot()
        {
            Assert.Equal(new[] { "BRA $0004", "NOP" }, Disassemble(2, 0x05, 0x02, 0x01));
        }

        // --- Coverage of the chip's own instruction stream (§8.2) ---

        [Fact]
        public void Gsu_coverage_records_only_the_addresses_the_chip_executed()
        {
            var (core, target) = GsuTarget((0x0000, new byte[] { 0xA1, 0x11, 0x00 })); // IBT R1,#$11 : STOP
            var coverage = target.CoprocessorCoverage!;
            coverage.Arm();

            var gsu = core.Cart!.SuperFx!;
            gsu.WriteRegister(0x3039, 0x01);
            gsu.WriteRegister(0x301E, 0x00);
            gsu.WriteRegister(0x301F, 0x00); // starts the chip at $00:0000
            gsu.Run(2000);

            Assert.True(coverage.WasExecuted(0x000000)); // IBT
            Assert.True(coverage.WasExecuted(0x000002)); // STOP
            Assert.False(coverage.WasExecuted(0x000001)); // the immediate operand, not an opcode
            Assert.False(coverage.WasExecuted(0x000003)); // never reached
        }

        // Disarmed is the default, so a normal run pays nothing and reports
        // nothing - the property that lets the hook sit in the dispatcher.
        [Fact]
        public void Coverage_records_nothing_until_it_is_armed()
        {
            var (core, target) = GsuTarget((0x0000, new byte[] { 0xA1, 0x11, 0x00 }));
            var coverage = target.CoprocessorCoverage!;

            var gsu = core.Cart!.SuperFx!;
            gsu.WriteRegister(0x3039, 0x01);
            gsu.WriteRegister(0x301E, 0x00);
            gsu.WriteRegister(0x301F, 0x00);
            gsu.Run(2000);

            Assert.False(coverage.IsArmed);
            Assert.Equal(0, coverage.InstructionsRecorded);
            Assert.False(coverage.WasExecuted(0x000000));
        }
    }
}
