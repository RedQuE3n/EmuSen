using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cheats;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The N64 GameShark codec, and its codes reaching a running Mars - see Mars_Cheats.md.
    public class MarsCheatTests : IDisposable
    {
        // beq zero, zero, -1 and its empty delay slot, so the processor never touches RDRAM.
        private static readonly byte[] SpinForever = { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 };

        // A recognisable cartridge byte past the boot code, at ROM offset 0x1040.
        private const byte Original = 0xCE;
        private const int RomOffset = 0x1040;

        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch { }
            }
        }

        private MarsCore Load(bool expansionPak = false)
        {
            string path = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(
                length: 0x2000,
                patches: new[] { (0, SpinForever), (RomOffset - 0x40, new byte[] { Original, 0x11, 0x22, 0x33 }) }));
            _temporaryFiles.Add(path);

            var core = new MarsCore(expansionPak, batteryRamDisabled: true);
            core.LoadRom(path);
            return core;
        }

        private static CheatWrite Only(string code) => Assert.Single(N64GameSharkCodec.DecodeWrites(code));

        private static void Apply(MarsCore core, string code)
        {
            core.Cheats.AddCheat(CheatKind.RamPoke, N64GameSharkCodec.DecodeWrites(code), null, code);
            core.ApplyCheats();
        }

        // --- The decode ---

        [Fact]
        public void An_8_bit_write_is_one_byte_at_the_commands_low_24_bits()
        {
            CheatWrite w = Only("8033B21E 0008");

            Assert.Equal("RDRAM", w.Space);
            Assert.Equal(0x33B21E, w.Address);
            Assert.Equal(0x08u, w.Value);
            Assert.Equal(1, w.EffectiveWidth);
            Assert.Equal(CheatWriteType.Set, w.Type);
            Assert.Equal(1, w.EffectiveRepeatCount);
        }

        // Both references cast an 8-bit code's value to its low byte - see Mars_Cheats.md §2.1.
        [Fact]
        public void An_8_bit_write_takes_the_low_byte_of_its_value()
        {
            Assert.Equal(0x42u, Only("80000100 FF42").Value);
        }

        [Fact]
        public void A_16_bit_write_is_two_bytes_wide_and_big_endian()
        {
            CheatWrite w = Only("81000100 1234");

            Assert.Equal(0x100, w.Address);
            Assert.Equal(0x1234u, w.Value);
            Assert.Equal(2, w.EffectiveWidth);
            Assert.True(w.BigEndian);
        }

        // KSEG0 and KSEG1 reach one physical byte, and Mars has no cache to tell them apart - see Mars_Cheats.md §3.
        [Fact]
        public void The_uncached_types_name_the_same_physical_byte()
        {
            CheatWrite a0 = Only("A033B21E 0008");
            CheatWrite a1 = Only("A1000100 1234");

            Assert.Equal(0x33B21E, a0.Address);
            Assert.Equal(0x08u, a0.Value);
            Assert.Equal(1, a0.EffectiveWidth);
            Assert.Equal(0x100, a1.Address);
            Assert.Equal(2, a1.EffectiveWidth);
            Assert.True(a1.BigEndian);
        }

        [Fact]
        public void The_type_byte_never_reaches_the_address()
        {
            Assert.Equal(0xFFFFFF, Only("80FFFFFF 0001").Address);
            Assert.Equal(0xFFFFFE, Only("A1FFFFFE 0001").Address);
            Assert.Equal(0x000000, Only("80000000 0001").Address);
        }

        [Fact]
        public void The_four_tests_decode_to_their_comparison_and_width()
        {
            var d0 = N64GameSharkCodec.DecodeWrites("D0000200 0005+80000100 0001")[0];
            var d1 = N64GameSharkCodec.DecodeWrites("D1000200 1234+80000100 0001")[0];
            var d2 = N64GameSharkCodec.DecodeWrites("D2000200 0005+80000100 0001")[0];
            var d3 = N64GameSharkCodec.DecodeWrites("D3000200 1234+80000100 0001")[0];

            Assert.Equal((CheatWriteType.IfEqual, 1, 0x05u), (d0.Type, d0.EffectiveWidth, d0.Value));
            Assert.Equal((CheatWriteType.IfEqual, 2, 0x1234u), (d1.Type, d1.EffectiveWidth, d1.Value));
            Assert.Equal((CheatWriteType.IfNotEqual, 1, 0x05u), (d2.Type, d2.EffectiveWidth, d2.Value));
            Assert.Equal((CheatWriteType.IfNotEqual, 2, 0x1234u), (d3.Type, d3.EffectiveWidth, d3.Value));
            Assert.All(new[] { d0, d1, d2, d3 }, t => Assert.Equal(0x200, t.Address));
            Assert.True(d1.BigEndian);
        }

        // mupen64plus compares the low byte and Project64 all sixteen bits; Mars follows mupen64plus - see Mars_Cheats.md §2.2.
        [Fact]
        public void An_8_bit_test_compares_its_low_byte()
        {
            Assert.Equal(0x05u, N64GameSharkCodec.DecodeWrites("D0000200 0305+80000100 0001")[0].Value);
        }

        [Fact]
        public void A_repeater_folds_into_the_write_it_repeats()
        {
            CheatWrite w = Only("50000302 0001+80000100 0010");

            Assert.Equal(0x100, w.Address);
            Assert.Equal(0x10u, w.Value);
            Assert.Equal(3, w.RepeatCount);
            Assert.Equal(2, w.RepeatAddAddress);
            Assert.Equal(1u, w.RepeatAddValue);
        }

        // Project64 reads the count from bits 8-15 whatever bits 16-23 hold - see Mars_Cheats.md §2.3.
        [Fact]
        public void A_repeaters_second_byte_is_ignored()
        {
            CheatWrite w = Only("50AA0301 0002+80000100 0010");

            Assert.Equal(3, w.RepeatCount);
            Assert.Equal(1, w.RepeatAddAddress);
            Assert.Equal(2u, w.RepeatAddValue);
        }

        [Theory]
        [InlineData("D0000200 0005+80000100 0001")]
        [InlineData("D0000200 0005 80000100 0001")]
        [InlineData("D0000200:0005,80000100-0001")]
        [InlineData("D00002000005+800001000001")]
        [InlineData("D00002000005800001000001")]
        [InlineData("d0000200 5+80000100 1")]
        public void Lines_decode_in_order_whatever_separates_them(string code)
        {
            var writes = N64GameSharkCodec.DecodeWrites(code);

            Assert.Equal(2, writes.Count);
            Assert.Equal((CheatWriteType.IfEqual, 0x200, 0x05u), (writes[0].Type, writes[0].Address, writes[0].Value));
            Assert.Equal((CheatWriteType.Set, 0x100, 0x01u), (writes[1].Type, writes[1].Address, writes[1].Value));
        }

        [Theory]
        [InlineData("8033B21E 0008", true)]
        [InlineData("8033B21E0008", true)]
        [InlineData("88000100 0001", true)]
        [InlineData("8125508A 00??", true)]
        [InlineData("8033B21E", false)]
        [InlineData("8033B21E 00008", false)]
        [InlineData("SXIOPO", false)]
        [InlineData("DD82-64DD", false)]
        [InlineData("7E0DBF09", false)]
        [InlineData("", false)]
        public void Can_decode_answers_on_shape_alone(string code, bool expected)
        {
            Assert.Equal(expected, N64GameSharkCodec.CanDecode(code));
        }

        // Every refusal names its reason, so no code is kept and silently never applied - see Mars_Cheats.md §4.
        [Theory]
        [InlineData("88000100 0001", "GameShark's own button")]
        [InlineData("89000100 0001", "GameShark's own button")]
        [InlineData("D8000100 0001+80000100 0001", "GameShark's own button")]
        [InlineData("F0000100 0001", "boot-time write")]
        [InlineData("F1000100 0001", "boot-time write")]
        [InlineData("EE000000 0000", "Expansion Pak")]
        [InlineData("DE000400 0000", "no effect on memory")]
        [InlineData("CC000000 0000", "either reference emulator applies")]
        [InlineData("FF000000 0000", "either reference emulator applies")]
        [InlineData("82000100 0001", "not a code type Mars decodes")]
        [InlineData("81000101 1234", "odd address")]
        [InlineData("D1000201 1234+80000100 0001", "odd address")]
        [InlineData("50000201 0000+81000100 1234", "odd address")]
        [InlineData("50000002 0000+80000100 0001", "zero times")]
        [InlineData("50000302 0001+D0000100 0001+80000100 0001", "must be followed by")]
        [InlineData("50000302 0001", "no line after it")]
        [InlineData("D0000200 0005", "guards nothing")]
        [InlineData("80000100 0001+D0000200 0005", "guards nothing")]
        [InlineData("8125508A 00??", "placeholder")]
        [InlineData("8033B21E", "isn't an N64 GameShark code")]
        public void A_code_mars_will_not_apply_is_refused_by_name(string code, string reason)
        {
            var ex = Assert.Throws<FormatException>(() => N64GameSharkCodec.DecodeWrites(code));
            Assert.Contains(reason, ex.Message);
        }

        // One odd repetition is none at all, so only a run that moves needs an even step.
        [Fact]
        public void A_single_16_bit_repetition_may_have_an_odd_step()
        {
            Assert.Equal(1, Only("50000101 0000+81000100 1234").RepeatCount);
        }

        [Fact]
        public void Decode_answers_only_for_a_lone_8_bit_write()
        {
            Assert.Equal((0x33B21E, (byte)0x08), N64GameSharkCodec.Decode("8033B21E 0008"));
            Assert.Throws<FormatException>(() => N64GameSharkCodec.Decode("81000100 1234"));
            Assert.Throws<FormatException>(() => N64GameSharkCodec.Decode("50000302 0001+80000100 0010"));
            Assert.Throws<FormatException>(() => N64GameSharkCodec.Decode("D0000200 0005+80000100 0001"));
        }

        // --- On the machine ---

        // RDRAM is big-endian here, so the value's high byte is the lower address - see Mars_Cheats.md §3.
        [Fact]
        public void A_16_bit_write_lands_high_byte_first_as_the_processor_reads_it()
        {
            MarsCore core = Load();

            Apply(core, "81000100 1234");

            Assert.Equal(0x12, core.Bus!.Rdram[0x100]);
            Assert.Equal(0x34, core.Bus.Rdram[0x101]);
            Assert.Equal(0x0000, core.Bus.Rdram[0x102] | core.Bus.Rdram[0x103]);
            Assert.Equal(0x1234_0000u, core.Bus.Read32(0x100));
        }

        [Fact]
        public void An_8_bit_write_lands_on_its_own_byte()
        {
            MarsCore core = Load();

            Apply(core, "80000101 FF42");

            Assert.Equal(new byte[] { 0x00, 0x42, 0x00, 0x00 }, core.Bus!.Rdram[0x100..0x104]);
        }

        [Fact]
        public void An_equal_test_guards_only_the_next_write()
        {
            MarsCore core = Load();
            core.Bus!.Rdram[0x200] = 0x06;

            Apply(core, "D0000200 0005+80000100 0001+80000101 0002");

            Assert.Equal(0x00, core.Bus.Rdram[0x100]);
            Assert.Equal(0x02, core.Bus.Rdram[0x101]);

            core.Bus.Rdram[0x200] = 0x05;
            core.ApplyCheats();

            Assert.Equal(0x01, core.Bus.Rdram[0x100]);
        }

        [Fact]
        public void A_16_bit_test_compares_in_rdram_byte_order()
        {
            MarsCore core = Load();
            core.Bus!.Rdram[0x200] = 0x34;
            core.Bus.Rdram[0x201] = 0x12;

            Apply(core, "D1000200 1234+80000100 0001");
            Assert.Equal(0x00, core.Bus.Rdram[0x100]);

            core.Bus.Rdram[0x200] = 0x12;
            core.Bus.Rdram[0x201] = 0x34;
            core.ApplyCheats();
            Assert.Equal(0x01, core.Bus.Rdram[0x100]);
        }

        [Fact]
        public void The_not_equal_tests_pass_where_the_equal_ones_fail()
        {
            MarsCore core = Load();
            core.Bus!.Rdram[0x200] = 0x05;
            core.Bus.Rdram[0x202] = 0x12;
            core.Bus.Rdram[0x203] = 0x34;

            Apply(core, "D2000200 0005+80000100 0001");
            Apply(core, "D2000200 0006+80000101 0002");
            Apply(core, "D3000202 1234+80000102 0003");
            Apply(core, "D3000202 3412+80000103 0004");

            Assert.Equal(new byte[] { 0x00, 0x02, 0x00, 0x04 }, core.Bus.Rdram[0x100..0x104]);
        }

        // Both references make a run of tests an AND: a failed one is not undone by the next passing - see Mars_Cheats.md §2.2.
        [Theory]
        [InlineData(0x05, 0x07, 0x09)]
        [InlineData(0x04, 0x07, 0x00)]
        [InlineData(0x05, 0x06, 0x00)]
        public void A_run_of_tests_must_all_pass(byte first, byte second, byte expected)
        {
            MarsCore core = Load();
            core.Bus!.Rdram[0x200] = first;
            core.Bus.Rdram[0x201] = second;

            Apply(core, "D0000200 0005+D0000201 0007+80000100 0009");

            Assert.Equal(expected, core.Bus.Rdram[0x100]);
        }

        [Fact]
        public void A_repeater_steps_its_address_and_its_value()
        {
            MarsCore core = Load();

            Apply(core, "50000302 0001+80000100 0010");

            Assert.Equal(new byte[] { 0x10, 0x00, 0x11, 0x00, 0x12, 0x00, 0x00 }, core.Bus!.Rdram[0x100..0x107]);
        }

        [Fact]
        public void A_16_bit_repeater_steps_in_halfwords_high_byte_first()
        {
            MarsCore core = Load();

            Apply(core, "50000204 0101+81000100 1234");

            Assert.Equal(new byte[] { 0x12, 0x34, 0x00, 0x00, 0x13, 0x35, 0x00, 0x00 }, core.Bus!.Rdram[0x100..0x108]);
        }

        // An 8-bit repetition keeps the low byte of a value that carries past it.
        [Fact]
        public void An_8_bit_repeater_wraps_its_value_at_a_byte()
        {
            MarsCore core = Load();

            Apply(core, "50000201 0001+800001FF 00FF");

            Assert.Equal(new byte[] { 0xFF, 0x00 }, core.Bus!.Rdram[0x1FF..0x201]);
        }

        // Project64 guards the whole run, mupen64plus only its first write; Mars follows Project64 - see Mars_Cheats.md §2.3.
        [Theory]
        [InlineData(0x01, 0x07)]
        [InlineData(0x00, 0x00)]
        public void A_test_before_a_repeater_guards_every_repetition(byte flag, byte expected)
        {
            MarsCore core = Load();
            core.Bus!.Rdram[0x200] = flag;

            Apply(core, "D0000200 0001+50000301 0000+80000100 0007");

            Assert.Equal(new[] { expected, expected, expected }, core.Bus.Rdram[0x100..0x103]);
        }

        // A stock console's RDRAM ends at 4MB; past it a write is dropped and a test reads zero - see Mars_Cheats.md §3.1.
        [Fact]
        public void Past_installed_rdram_a_write_is_dropped_and_a_test_reads_zero()
        {
            MarsCore core = Load();

            Apply(core, "80500000 0001");
            Apply(core, "D0500000 0000+80000100 0007");
            Apply(core, "D2500000 0000+80000101 0008");

            Assert.Equal(0x40_0000, core.Bus!.Rdram.Length);
            Assert.Equal(0x07, core.Bus.Rdram[0x100]);
            Assert.Equal(0x00, core.Bus.Rdram[0x101]);
            Assert.Equal(0u, core.Bus.Read32(0x50_0000));
        }

        [Fact]
        public void With_the_expansion_pak_the_same_write_lands()
        {
            MarsCore core = Load(expansionPak: true);

            Apply(core, "80500000 0001");

            Assert.Equal(0x01, core.Bus!.Rdram[0x50_0000]);
        }

        [Fact]
        public void The_other_writable_spaces_take_a_poke_and_an_unknown_one_is_ignored()
        {
            MarsCore core = Load();

            core.Cheats.AddRamPoke("dmem", 0x10, 0x5A, "dmem");
            core.Cheats.AddRamPoke("PIFRAM", 0x3F, 0x5B, "pif");
            core.Cheats.AddRamPoke("CPUBUS", 0x10, 0x5C, "not a space here");
            core.ApplyCheats();

            Assert.Equal(0x5A, core.Bus!.SpDmem[0x10]);
            Assert.Equal(0x5B, core.Bus.PifRam[0x3F]);
            Assert.Equal(0x00, core.Bus.Rdram[0x10]);
        }

        // --- When the frame boundary applies them ---

        // lui/ori/mtc0 set Status.IE with no interrupt unmasked, then spin, as a running game's threads do.
        private static readonly byte[] EnableInterruptsThenSpin =
        {
            0x3C, 0x08, 0x34, 0x00, 0x35, 0x08, 0x00, 0x01, 0x40, 0x88, 0x60, 0x00,
            0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        };

        private MarsCore LoadRunning(byte[] program)
        {
            string path = SyntheticN64Rom.WriteTemp(SyntheticN64Rom.Build(patches: (0, program)));
            _temporaryFiles.Add(path);

            var core = new MarsCore(batteryRamDisabled: true);
            core.LoadRom(path);
            core.Cheats.AddRamPoke(MarsCore.SpaceRdram, 0x100, 0x42, "poke");
            return core;
        }

        // The boot code runs with interrupts off and checksums what a cheat could overwrite, so the frame end waits, as Project64 does - see Mars_Cheats.md §5.1.
        [Fact]
        public void The_frame_end_applies_nothing_while_interrupts_are_disabled()
        {
            MarsCore booting = LoadRunning(SpinForever);
            booting.RunFrame();
            Assert.Equal(0x00, booting.Bus!.Rdram[0x100]);

            MarsCore running = LoadRunning(EnableInterruptsThenSpin);
            running.RunFrame();
            Assert.Equal(0x42, running.Bus!.Rdram[0x100]);
        }

        // An explicit Apply is the user's instruction and is not held back - see Mars_Cheats.md §5.1.
        [Fact]
        public void An_explicit_apply_does_not_wait_for_interrupts()
        {
            MarsCore booting = LoadRunning(SpinForever);
            booting.ApplyCheats();
            Assert.Equal(0x42, booting.Bus!.Rdram[0x100]);
        }

        // Project64's own Infinite Lives for this game, on from power-on: applied mid-checksum it hung the boot - see Mars_Cheats.md §5.1.
        [Fact]
        public void A_cheat_on_from_power_on_leaves_the_boot_checksum_alone()
        {
            string path = Path.Combine(N64TestRomLibrary.Root, "Super Mario 64 (Europe) (En,Fr,De).z64");
            if (!File.Exists(path)) return;

            var core = new MarsCore(batteryRamDisabled: true);
            core.LoadRom(path);
            core.Cheats.AddCheat(CheatKind.RamPoke, N64GameSharkCodec.DecodeWrites("803094DD 0064"), null, "Infinite lives");

            for (int frame = 0; frame < 4; frame++) core.RunFrame();

            Assert.True(core.Bus!.Vi.Fields > 0, $"the game never programmed the VI; the processor is at {(uint)core.Cpu!.CurrentPc:X8}");
            Assert.Equal(0x64, core.Bus.Rdram[0x3094DD]);
        }

        // --- The cartridge ---

        [Fact]
        public void A_rom_patch_reaches_a_processor_load_by_rom_offset()
        {
            MarsCore core = Load();
            uint physical = MemoryMap.CartDomain1Address2 + RomOffset;

            Assert.Equal(Original, core.Bus!.Read8(physical));

            core.Cheats.AddRomPatch(RomOffset, 0xAD, null, "patch");

            Assert.Equal(0xAD, core.Bus.Read8(physical));
            Assert.Equal(0xAD11_2233u, core.Bus.Read32(physical));
            Assert.Equal(Original, core.Rom!.Rom[RomOffset]);
        }

        // Games copy their code out of the cartridge, so the transfer engine has to see the patch too - see Mars_Cheats.md §6.
        [Fact]
        public void A_rom_patch_reaches_a_transfer_into_rdram()
        {
            MarsCore core = Load();
            core.Cheats.AddRomPatch(RomOffset + 1, 0xEE, null, "patch");

            core.Bus!.Write32(MemoryMap.PiBase + PiInterface.DramAddress, 0x1000);
            core.Bus.Write32(MemoryMap.PiBase + PiInterface.CartAddress, MemoryMap.CartDomain1Address2 + RomOffset);
            core.Bus.Write32(MemoryMap.PiBase + PiInterface.WriteLength, 4 - 1);

            Assert.Equal(new byte[] { Original, 0xEE, 0x22, 0x33 }, core.Bus.Rdram[0x1000..0x1004]);
        }

        [Fact]
        public void A_disabled_rom_patch_gives_the_cartridge_byte_back()
        {
            MarsCore core = Load();
            uint physical = MemoryMap.CartDomain1Address2 + RomOffset;

            int id = core.Cheats.AddRomPatch(RomOffset, 0xAD, null, "patch");
            core.Cheats.SetEnabled(id, false);

            Assert.Equal(Original, core.Bus!.Read8(physical));
        }

        // The patcher follows the registry a frontend hands over after the ROM is loaded - see EmuSen_Cheats.md §6.
        [Fact]
        public void A_handed_over_registry_patches_the_already_loaded_cartridge()
        {
            MarsCore core = Load();
            var handed = new CheatRegistry();
            core.Cheats = handed;

            handed.AddRomPatch(RomOffset, 0xAD, null, "patch");

            Assert.Equal(0xAD, core.Bus!.Read8(MemoryMap.CartDomain1Address2 + RomOffset));
        }

        // --- Through the shell ---

        private DianaOSInterpreter Shell(out MarsCore core)
        {
            core = Load();
            var target = new MarsDebugTarget(core);
            return DianaOSInterpreter.CreateDefault(target, null, new N64GameSharkCheatCodec(), null);
        }

        [Fact]
        public void Cheat_add_takes_a_multi_line_code_whole()
        {
            DianaOSInterpreter shell = Shell(out MarsCore core);
            core.Bus!.Rdram[0x200] = 0x05;

            string output = shell.Submit("cheat add D00002000005+800001000001 lives").Output;
            core.ApplyCheats();

            Assert.Contains("GameShark", output);
            Assert.Contains("2 writes", output);
            Assert.Equal(0x01, core.Bus.Rdram[0x100]);
            Assert.Equal(2, core.Cheats.GetCheats().Single().Writes.Count);
        }

        [Fact]
        public void Cheat_add_says_why_it_refuses_a_code()
        {
            DianaOSInterpreter shell = Shell(out MarsCore core);

            string output = shell.Submit("cheat add 880001000001").Output;

            Assert.Contains("GameShark's own button", output);
            Assert.Empty(core.Cheats.GetCheats());
        }

        [Fact]
        public void Cheat_list_shows_a_test_as_a_test()
        {
            DianaOSInterpreter shell = Shell(out _);
            shell.Submit("cheat add D10002001234+810001005678 guarded");

            string output = shell.Submit("cheat list").Output;

            Assert.Contains("if RDRAM 0x200 == 0x1234", output);
            Assert.Contains("RDRAM 0x100 = 0x5678", output);
        }

        // One .cht entry holds one write, so a test and the write it guards cannot both go in it - see EmuSen_Cheats.md §7.
        [Fact]
        public void Export_skips_a_conditional_cheat_and_says_so()
        {
            DianaOSInterpreter shell = Shell(out _);
            shell.Submit("cheat add 8000010000FF plain");
            shell.Submit("cheat add D00002000005+800001000001 guarded");

            string path = Path.Combine(DianaOSSandbox.RootDirectory, "tmp", "mars_export.cht");
            string output = shell.Submit($"cheat export {path}").Output;

            Assert.Contains("Exported 1 cheat(s)", output);
            Assert.Contains("1 conditional cheat(s) skipped", output);
        }

        // The libretro database joins lines with '+', and a repeater spans one - see Mars_Cheats.md §5.
        [Fact]
        public void A_cht_file_imports_each_code_whole_and_skips_what_it_refuses()
        {
            const string text = """
                cheats = 3

                cheat0_desc = "Max items"
                cheat0_code = "50000302 0001+80000100 0010"
                cheat0_enable = false

                cheat1_desc = "Guarded"
                cheat1_code = "D0000200 0005+81000100 1234"
                cheat1_enable = false

                cheat2_desc = "Press GS"
                cheat2_code = "88000100 0001"
                cheat2_enable = false
                """;

            var registry = new CheatRegistry();
            CheatImportResult result = CheatImport.FromChtText(registry, text, new N64GameSharkCheatCodec());

            Assert.Equal(2, result.Loaded);
            Assert.Equal(1, result.Skipped);

            var cheats = registry.GetCheats();
            CheatWrite run = Assert.Single(cheats[0].Writes);
            Assert.Equal((0x100, 3, 2), (run.Address, run.RepeatCount, run.RepeatAddAddress));
            Assert.Equal(new[] { CheatWriteType.IfEqual, CheatWriteType.Set }, cheats[1].Writes.Select(w => w.Type));
            Assert.All(cheats.SelectMany(c => c.Writes), w => Assert.Equal("RDRAM", w.Space));
        }
    }
}
