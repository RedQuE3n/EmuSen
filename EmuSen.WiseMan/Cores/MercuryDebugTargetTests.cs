using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Cheats;
using EmuSen.Cores.Nintendo.Mercury.Cpu.Disassembler;
using EmuSen.Cores.Nintendo.Mercury.Debug;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // MercuryDebugTarget and the SM83 disassembler under it - see Mercury_Debug.md.
    public class MercuryDebugTargetTests : IDisposable
    {
        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private string WriteRom(byte cgbFlag = 0x00, params (int Offset, byte[] Bytes)[] patches)
        {
            string path = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(cgbFlag: cgbFlag, patches: patches));
            _temporaryFiles.Add(path);
            return path;
        }

        private (MercuryCore Core, MercuryDebugTarget Target) Load(byte cgbFlag = 0x00, params (int Offset, byte[] Bytes)[] patches)
        {
            var core = new MercuryCore();
            core.LoadRom(WriteRom(cgbFlag, patches));
            return (core, new MercuryDebugTarget(core));
        }

        private static string Text(DisassembledInstruction instruction) =>
            instruction.OperandText.Length == 0
                ? instruction.Mnemonic
                : $"{instruction.Mnemonic} {instruction.OperandText}";

        [Fact]
        public void Every_opcode_has_a_length_between_one_and_three()
        {
            for (int opcode = 0; opcode <= 0xFF; opcode++)
            {
                int length = Sm83Disassembler.LengthOf((byte)opcode);
                Assert.InRange(length, 1, 3);
            }
        }

        // The eleven holes in the table are the ones the CPU refuses to execute - see Mercury_Cpu.md §6.1.
        [Fact]
        public void The_illegal_opcodes_are_exactly_the_ones_the_cpu_throws_on()
        {
            var illegal = new List<int>();

            for (int opcode = 0; opcode <= 0xFF; opcode++)
            {
                if (Sm83Disassembler.MnemonicOf((byte)opcode, 0x00) == Sm83Disassembler.Illegal) illegal.Add(opcode);
            }

            Assert.Equal(
                new[] { 0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD },
                illegal);
        }

        [Theory]
        [InlineData(new byte[] { 0x00 }, "NOP")]
        [InlineData(new byte[] { 0x21, 0x34, 0x12 }, "LD HL,$1234")]
        [InlineData(new byte[] { 0x3E, 0x7F }, "LD A,$7F")]
        [InlineData(new byte[] { 0xC3, 0x50, 0x01 }, "JP $0150")]
        [InlineData(new byte[] { 0xCD, 0x00, 0x40 }, "CALL $4000")]
        [InlineData(new byte[] { 0xE0, 0x44 }, "LDH ($FF44),A")]
        [InlineData(new byte[] { 0xF0, 0x40 }, "LDH A,($FF40)")]
        [InlineData(new byte[] { 0xEA, 0x00, 0xC0 }, "LD ($C000),A")]
        [InlineData(new byte[] { 0x76 }, "HALT")]
        [InlineData(new byte[] { 0xFF }, "RST 38H")]
        public void The_common_forms_print_the_way_a_reader_expects(byte[] bytes, string expected)
        {
            var (_, target) = Load(patches: (0, bytes));

            var instruction = target.Disassemble(MercuryCore.SpaceCpuBus, SyntheticGbRom.EntryPoint, 1)[0];
            Assert.Equal(expected, Text(instruction));
            Assert.Equal(bytes.Length, instruction.Length);
        }

        // A relative branch prints where it lands, not the byte it stores.
        [Fact]
        public void A_relative_jump_resolves_to_its_target_address()
        {
            var (_, target) = Load(patches: (0, new byte[] { 0x18, 0xFE }));

            var instruction = target.Disassemble(MercuryCore.SpaceCpuBus, SyntheticGbRom.EntryPoint, 1)[0];
            Assert.Equal($"JR ${SyntheticGbRom.EntryPoint:X4}", Text(instruction));
        }

        // A displacement is not a target: SP-2 must not read as SP+254.
        [Fact]
        public void A_stack_displacement_prints_signed()
        {
            var (_, target) = Load(patches: (0, new byte[] { 0xE8, 0xFE, 0xF8, 0x02 }));

            var instructions = target.Disassemble(MercuryCore.SpaceCpuBus, SyntheticGbRom.EntryPoint, 2);
            Assert.Equal("ADD SP,-2", Text(instructions[0]));
            Assert.Equal("LD HL,SP+2", Text(instructions[1]));
        }

        [Theory]
        [InlineData(new byte[] { 0xCB, 0x11 }, "RL C")]
        [InlineData(new byte[] { 0xCB, 0x37 }, "SWAP A")]
        [InlineData(new byte[] { 0xCB, 0x7E }, "BIT 7,(HL)")]
        [InlineData(new byte[] { 0xCB, 0x86 }, "RES 0,(HL)")]
        [InlineData(new byte[] { 0xCB, 0xFF }, "SET 7,A")]
        public void The_cb_page_decodes_to_its_operation_and_target(byte[] bytes, string expected)
        {
            var (_, target) = Load(patches: (0, bytes));

            var instruction = target.Disassemble(MercuryCore.SpaceCpuBus, SyntheticGbRom.EntryPoint, 1)[0];
            Assert.Equal(expected, Text(instruction));
            Assert.Equal(2, instruction.Length);
        }

        [Fact]
        public void Disassembly_walks_forward_by_each_instructions_own_length()
        {
            var (_, target) = Load(patches: (0, new byte[] { 0x00, 0x3E, 0x01, 0xC3, 0x50, 0x01 }));

            var instructions = target.Disassemble(MercuryCore.SpaceCpuBus, SyntheticGbRom.EntryPoint, 3);

            Assert.Equal(SyntheticGbRom.EntryPoint, instructions[0].Address);
            Assert.Equal(SyntheticGbRom.EntryPoint + 1, instructions[1].Address);
            Assert.Equal(SyntheticGbRom.EntryPoint + 3, instructions[2].Address);
        }

        [Fact]
        public void A_call_a_store_and_a_load_are_classified_by_what_they_reach()
        {
            var (_, target) = Load(patches: (0, new byte[] { 0xCD, 0x00, 0x40, 0xEA, 0x10, 0xC0, 0xFA, 0x20, 0xC0, 0xE0, 0x47 }));

            var instructions = target.Disassemble(MercuryCore.SpaceCpuBus, SyntheticGbRom.EntryPoint, 4);

            Assert.Equal((StaticReferenceKind.Call, 0x4000), target.ClassifyStaticReference(instructions[0]));
            Assert.Equal((StaticReferenceKind.Write, 0xC010), target.ClassifyStaticReference(instructions[1]));
            Assert.Equal((StaticReferenceKind.Read, 0xC020), target.ClassifyStaticReference(instructions[2]));
            Assert.Equal((StaticReferenceKind.Write, 0xFF47), target.ClassifyStaticReference(instructions[3]));
        }

        // A register-indirect target is not knowable from the bytes, so it is reported as nothing.
        [Fact]
        public void An_indirect_jump_is_not_classified_at_all()
        {
            var (_, target) = Load(patches: (0, new byte[] { 0xE9 }));

            var instruction = target.Disassemble(MercuryCore.SpaceCpuBus, SyntheticGbRom.EntryPoint, 1)[0];
            Assert.Null(target.ClassifyStaticReference(instruction));
        }

        [Fact]
        public void The_memory_spaces_are_the_ones_the_core_already_names()
        {
            var (_, target) = Load();

            string[] names = target.GetMemorySpaces().Select(s => s.Name).ToArray();
            Assert.Equal(new[] { "ROM", "VRAM", "CARTRAM", "WRAM", "OAM", "HRAM", "CPUBUS" }, names);

            var rom = target.GetMemorySpaces().First(s => s.Name == "ROM");
            Assert.False(rom.IsWritable);

            // Only the space routed through the live decode can disturb anything by being read.
            Assert.Single(target.GetMemorySpaces(), s => s.HasSideEffects);
        }

        [Fact]
        public void The_five_interrupt_vectors_are_reported_where_the_hardware_puts_them()
        {
            var (_, target) = Load();

            Assert.Equal(new[] { 0x40, 0x48, 0x50, 0x58, 0x60 }, target.InterruptVectors.Select(v => v.Address));
        }

        [Fact]
        public void Cpu_registers_report_the_live_machine()
        {
            var (core, target) = Load();
            target.RefreshProviders();

            var values = target.CpuRegisters.Current.ToDictionary(v => v.Name, v => v.Value);
            Assert.Equal(core.Cpu!.PC, values["PC"]);
            Assert.Equal(core.Cpu.SP, values["SP"]);
            Assert.Equal(core.Cpu.A, values["A"]);
        }

        [Fact]
        public void Video_registers_only_carry_the_colour_ones_in_colour_mode()
        {
            var (_, monochrome) = Load();
            monochrome.RefreshProviders();
            Assert.DoesNotContain(monochrome.VideoRegisters.Current, v => v.Name == "VBK");

            var (_, colour) = Load(cgbFlag: 0xC0);
            colour.RefreshProviders();
            Assert.Contains(colour.VideoRegisters.Current, v => v.Name == "VBK");
        }

        // Three shade palettes on a DMG, sixteen colour ones on a CGB.
        [Fact]
        public void The_palette_list_matches_the_console()
        {
            var (_, monochrome) = Load();
            monochrome.RefreshProviders();
            Assert.Equal(3, monochrome.Palettes.Current.Count);

            var (_, colour) = Load(cgbFlag: 0xC0);
            colour.RefreshProviders();
            Assert.Equal(16, colour.Palettes.Current.Count);
        }

        [Fact]
        public void A_sprite_parked_off_screen_is_left_out_of_the_sprite_list()
        {
            var (core, target) = Load();

            core.Bus!.Oam[0] = 32;      // Y = 16, on screen
            core.Bus.Oam[1] = 24;
            core.Bus.Oam[4] = 0;        // Y = -16, entirely above the panel
            core.Bus.Oam[8] = 200;      // Y = 184, entirely below it

            target.RefreshProviders();

            Assert.Single(target.Sprites.Current);
            Assert.Equal(16, target.Sprites.Current[0].Y);
            Assert.Equal(16, target.Sprites.Current[0].X);
        }

        [Fact]
        public void A_tile_decodes_row_interleaved_rather_than_plane_split()
        {
            var (core, target) = Load();

            // Row 0: low plane $80, high plane $80, which is colour 3 in the leftmost pixel only.
            core.Bus!.Vram[0] = 0x80;
            core.Bus.Vram[1] = 0x80;

            var space = target.GetMemorySpaces().First(s => s.Name == "VRAM");
            byte[] pixels = target.DecodeTilePixels(space, 0, 2);

            Assert.Equal(3, pixels[0]);
            Assert.Equal(0, pixels[1]);
        }

        [Fact]
        public void Asking_for_a_bit_depth_the_hardware_has_not_got_is_an_error()
        {
            var (_, target) = Load();
            var space = target.GetMemorySpaces().First(s => s.Name == "VRAM");

            Assert.Throws<ArgumentException>(() => target.DecodeTilePixels(space, 0, 4));
        }

        [Fact]
        public void A_bus_address_resolves_to_the_space_that_actually_holds_it()
        {
            var (_, target) = Load();

            Assert.Equal(MercuryCore.SpaceVram, target.ResolvePhysical(0x8010)!.Value.Space);
            Assert.Equal(MercuryCore.SpaceWram, target.ResolvePhysical(0xC005)!.Value.Space);
            Assert.Equal(MercuryCore.SpaceOam, target.ResolvePhysical(0xFE04)!.Value.Space);
            Assert.Equal(MercuryCore.SpaceHram, target.ResolvePhysical(0xFF85)!.Value.Space);

            // A banked ROM window depends on live mapper state, so it is reported as unknowable.
            Assert.Null(target.ResolvePhysical(0x4000));

            // An I/O register is a real place but not one `mem` can be pointed at.
            Assert.False(target.ResolvePhysical(0xFF40)!.Value.IsAddressable);
        }

        [Fact]
        public void The_summary_names_the_console_the_board_and_the_line()
        {
            var (core, target) = Load(cgbFlag: 0xC0);
            core.RunFrame();

            string summary = target.GetSummaryText();
            Assert.Contains("GBC (Mercury)", summary);
            Assert.Contains("CGB", summary);
            Assert.Contains("ROM,", summary);
        }

        [Fact]
        public void The_factory_builds_a_bundle_for_both_game_boy_extensions()
        {
            foreach (string extension in new[] { ".gb", ".gbc" })
            {
                string path = Path.ChangeExtension(WriteRom(), extension);
                File.WriteAllBytes(path, SyntheticGbRom.Build());
                _temporaryFiles.Add(path);

                Assert.True(CoreFactory.IsSupported(path));

                var bundle = CoreFactory.Load(path);
                Assert.IsType<MercuryCore>(bundle.Core);
                Assert.IsType<MercuryDebugTarget>(bundle.DebugTarget);
                Assert.IsType<GbGameSharkCheatCodec>(bundle.CheatAutoDetectCodec);
                Assert.IsType<GbGameGenieCheatCodec>(bundle.CheatExplicitCodec);
            }
        }

        [Fact]
        public void The_catalog_claims_the_two_game_boy_cheat_folders()
        {
            Assert.Contains("Nintendo - Game Boy", CoreCatalog.SupportedCheatSystems);
            Assert.Contains("Nintendo - Game Boy Color", CoreCatalog.SupportedCheatSystems);
            Assert.Equal("GB", CoreCatalog.ConsoleForRom("pocket.gb"));
            Assert.Equal("GB", CoreCatalog.ConsoleForRom("pocket.gbc"));
        }
    }
}
