using EmuSen.Cauldron;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The NES debug target's cases through IDebugTarget alone, run once per engine by the two classes below - see Moon_Native.md §8.4.
    public abstract class MoonDebugTargetContract : IDisposable
    {
        private readonly List<string> _files = new();
        private readonly List<NesDebugRig> _rigs = new();

        protected abstract string Engine { get; }

        protected MoonDebugTargetContract() => CoreOptions.BatteryRamDisabled = true;

        public void Dispose()
        {
            foreach (NesDebugRig rig in _rigs) rig.Dispose();
            foreach (string path in _files) try { File.Delete(path); } catch (IOException) { }
        }

        // The synthetic image, CHR RAM when <chrRam> so a tile can be written through its space.
        private NesDebugRig Load(bool chrRam = false, params (int Offset, byte[] Bytes)[] patches)
        {
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(chrBanks: chrRam ? 0 : 1, patches: patches));
            _files.Add(path);
            var rig = NesDebugRig.Load(Engine, path);
            _rigs.Add(rig);
            return rig;
        }

        private static IDebugMemorySpace Space(IDebugTarget target, string name) => target.GetMemorySpaces().First(s => s.Name == name);

        [Fact]
        public void The_target_names_the_console_and_its_sprite_capacity()
        {
            IDebugTarget target = Load().Target;
            Assert.Equal("NES", target.CoreName);
            Assert.Equal(64, target.MaxSprites);
            Assert.Equal(1, target.TilemapEntryStride);
        }

        [Fact]
        public void Every_advertised_memory_space_is_readable_at_its_own_size()
        {
            var spaces = Load().Target.GetMemorySpaces();
            Assert.NotEmpty(spaces);
            foreach (var space in spaces)
            {
                Assert.True(space.Size > 0, $"{space.Name} advertises no size");
                space.Read(0);
                space.Read(space.Size - 1);
            }
        }

        [Fact]
        public void Writing_through_a_space_reaches_the_real_hardware_behind_it()
        {
            IDebugTarget target = Load().Target;
            Space(target, "RAM").Write(0x0042, 0x99);
            Assert.Equal(0x99, Space(target, "CPUBUS").Read(0x0842));
        }

        // PRG ROM is not writable, and a debug write must not quietly corrupt the loaded image.
        [Fact]
        public void A_read_only_space_refuses_writes()
        {
            var prg = Space(Load().Target, "PRGROM");
            byte before = prg.Read(0x10);
            prg.Write(0x10, (byte)(before ^ 0xFF));
            Assert.False(prg.IsWritable);
            Assert.Equal(before, prg.Read(0x10));
        }

        // Reading $2002 clears vblank and $2007 moves the pointer, so the bus space must declare it.
        [Fact]
        public void Only_the_cpu_bus_space_declares_side_effects()
        {
            foreach (var space in Load().Target.GetMemorySpaces()) Assert.Equal(space.Name == "CPUBUS", space.HasSideEffects);
        }

        // $2006 written twice through the bus, as a program writes it.
        [Fact]
        public void Video_registers_expose_the_loopy_pair_by_name()
        {
            IDebugTarget target = Load().Target;
            var bus = Space(target, "CPUBUS");
            bus.Write(0x2006, 0x21);
            bus.Write(0x2006, 0x08);
            target.RefreshProviders();
            Assert.Equal(0x2108u, target.VideoRegisters.Current.First(r => r.Name == "v").Value);
        }

        [Fact]
        public void The_interrupt_vectors_are_the_three_the_6502_has()
        {
            var vectors = Load().Target.InterruptVectors;
            Assert.Equal(3, vectors.Count);
            Assert.Contains(vectors, v => v.Address == 0xFFFA);
            Assert.Contains(vectors, v => v.Address == 0xFFFC);
            Assert.Contains(vectors, v => v.Address == 0xFFFE);
        }

        [Fact]
        public void Disassembly_decodes_real_instructions_out_of_a_space()
        {
            IDebugTarget target = Load(false, (0, new byte[] { 0xA9, 0x42, 0x8D, 0x00, 0x02, 0x20, 0x10, 0x80 })).Target;
            var listing = target.Disassemble("PRGROM", 0, 3);
            Assert.Equal(("LDA", "#$42", 2), (listing[0].Mnemonic, listing[0].OperandText, listing[0].Length));
            Assert.Equal(("STA", "$0200"), (listing[1].Mnemonic, listing[1].OperandText));
            Assert.Equal(("JSR", "$8010"), (listing[2].Mnemonic, listing[2].OperandText));
        }

        [Fact]
        public void A_relative_branch_resolves_to_its_target_address()
        {
            var listing = Load(false, (0, new byte[] { 0xD0, 0x10 })).Target.Disassemble("PRGROM", 0, 1);
            Assert.Equal(("BNE", "$0012"), (listing[0].Mnemonic, listing[0].OperandText));
        }

        [Fact]
        public void Static_references_classify_calls_reads_and_writes()
        {
            IDebugTarget target = Load(false, (0, new byte[] { 0x20, 0x10, 0x80, 0x8D, 0x00, 0x02, 0xAD, 0x34, 0x12 })).Target;
            var listing = target.Disassemble("PRGROM", 0, 3);
            Assert.Equal((StaticReferenceKind.Call, 0x8010), target.ClassifyStaticReference(listing[0]));
            Assert.Equal((StaticReferenceKind.Write, 0x0200), target.ClassifyStaticReference(listing[1]));
            Assert.Equal((StaticReferenceKind.Read, 0x1234), target.ClassifyStaticReference(listing[2]));
        }

        // An indexed or indirect target is not knowable from the bytes, so it must report nothing.
        [Fact]
        public void A_runtime_dependent_operand_classifies_as_nothing()
        {
            IDebugTarget target = Load(false, (0, new byte[] { 0x9D, 0x00, 0x02, 0xB1, 0x20 })).Target;
            var listing = target.Disassemble("PRGROM", 0, 2);
            Assert.Null(target.ClassifyStaticReference(listing[0]));
            Assert.Null(target.ClassifyStaticReference(listing[1]));
        }

        // Low plane row 0 all set, high plane row 0 alternating: colours 1 and 3.
        [Fact]
        public void A_tile_decodes_to_sixty_four_two_bit_pixels()
        {
            IDebugTarget target = Load(chrRam: true).Target;
            var chr = Space(target, "CHR");
            chr.Write(0, 0xFF);
            chr.Write(8, 0xAA);
            byte[] pixels = target.DecodeTilePixels(chr, 0, 2);
            Assert.Equal(64, pixels.Length);
            Assert.Equal((3, 1), (pixels[0], pixels[1]));
            Assert.All(pixels, p => Assert.InRange(p, 0, 3));
        }

        [Fact]
        public void An_unsupported_bit_depth_is_an_argument_error_not_garbage()
        {
            IDebugTarget target = Load().Target;
            Assert.Throws<ArgumentException>(() => target.DecodeTilePixels(Space(target, "CHR"), 0, 4));
        }

        [Fact]
        public void A_nametable_entry_decodes_with_the_palette_its_attribute_byte_picks()
        {
            IDebugTarget target = Load().Target;
            var ciram = Space(target, "CIRAM");
            ciram.Write(0, 0xA3);
            ciram.Write(0x3C0, 0x02);
            Assert.Equal("$A3 p2", target.DecodeTilemapEntry(ciram, 0));
        }

        [Fact]
        public void The_attribute_table_region_is_labelled_as_such()
        {
            IDebugTarget target = Load().Target;
            var ciram = Space(target, "CIRAM");
            ciram.Write(0x3C0, 0x1B);
            Assert.Equal("attr $1B", target.DecodeTilemapEntry(ciram, 0x3C0));
        }

        [Fact]
        public void The_tile_sheet_and_palette_swatch_render_at_their_advertised_sizes()
        {
            IDebugTarget target = Load().Target;
            var (sheet, sheetWidth, sheetHeight) = target.RenderTileSheet();
            Assert.Equal(128, sheetWidth);
            Assert.Equal(sheetWidth * sheetHeight * 4, sheet.Length);
            var (swatch, swatchWidth, swatchHeight) = target.RenderPaletteSwatch();
            Assert.Equal((256, 32), (swatchWidth, swatchHeight));
            Assert.Equal(swatchWidth * swatchHeight * 4, swatch.Length);
        }

        [Fact]
        public void Eight_palettes_of_four_colours_are_published()
        {
            IDebugTarget target = Load().Target;
            target.RefreshProviders();
            Assert.Equal(8, target.Palettes.Current.Count);
            Assert.All(target.Palettes.Current, p => Assert.Equal(4, p.Colors.Count));
        }

        [Fact]
        public void Sprites_parked_below_the_screen_are_left_out()
        {
            IDebugTarget target = Load().Target;
            var oam = Space(target, "OAM");
            oam.Write(0, 0x20);
            oam.Write(3, 0x40);
            oam.Write(4, 0xF0);
            target.RefreshProviders();
            Assert.Contains(target.Sprites.Current, s => s.Index == 0 && s.X == 0x40);
            Assert.DoesNotContain(target.Sprites.Current, s => s.Index == 1);
        }

        // The write observer is the seam the whole watch mechanism hangs off.
        [Fact]
        public void A_watch_records_a_write_the_cpu_makes_to_ram()
        {
            IDebugTarget target = Load().Target;
            int id = target.Watches.AddWatch("RAM", 0x0042, 1);
            Space(target, "CPUBUS").Write(0x0042, 0x7E);
            var e = Assert.Single(target.Watches.GetEvents(id));
            Assert.Equal((0x42, (byte)0x7E), (e.Address, e.Value));
        }

        [Fact]
        public void A_breakpoint_halts_the_frame_and_resuming_does_not_re_halt_on_it()
        {
            NesDebugRig rig = Load();
            rig.Breakpoints.AddBreakpoint(0x8000);
            rig.Core.RunFrame();
            Assert.True(rig.Halted);
            Assert.Equal(0x8000, rig.HaltedAddress);
            rig.Core.RunFrame();
            Assert.Equal(1, rig.Core.TotalFrames);
        }

        [Fact]
        public void Audio_channels_are_published_as_the_five_the_apu_has()
        {
            IDebugTarget target = Load().Target;
            target.RefreshProviders();
            Assert.Equal(5, target.AudioChannels.Current.Count);
            Assert.Contains(target.AudioChannels.Current, c => c.Name == "Triangle");
        }

        [Fact]
        public void The_summary_names_the_mapper_and_the_current_frame()
        {
            NesDebugRig rig = Load();
            rig.Core.RunFrame();
            rig.Target.RefreshProviders();
            string summary = rig.Target.GetSummaryText();
            Assert.Contains("NES (Moon", summary);
            Assert.Contains("NROM", summary);
            Assert.Contains("CPU", summary);
        }

        [Fact]
        public void Physical_resolution_answers_for_fixed_ranges_and_declines_for_banked_ones()
        {
            IDebugTarget target = Load().Target;
            Assert.Equal("RAM", target.ResolvePhysical(0x0805)?.Space);
            Assert.Equal(0x005, target.ResolvePhysical(0x0805)?.Offset);
            Assert.False(target.ResolvePhysical(0x2002)?.IsAddressable);
            Assert.Null(target.ResolvePhysical(0x8000));
        }

        // The registers the target shows after a frame are the machine's: the same numbers on either engine.
        [Fact]
        public void Cpu_registers_after_a_frame_are_the_machines()
        {
            NesDebugRig rig = Load(false, (0, new byte[] { 0xA9, 0x12, 0xA2, 0x34, 0x4C, 0x04, 0x80 }));
            rig.Core.RunFrame();
            rig.Target.RefreshProviders();
            var registers = rig.Target.CpuRegisters.Current;
            Assert.Equal(0x12u, registers.First(r => r.Name == "A").Value);
            Assert.Equal(0x34u, registers.First(r => r.Name == "X").Value);
        }
    }

    public sealed class MoonDebugTargetContractOnMoon : MoonDebugTargetContract
    {
        protected override string Engine => NesDebugRig.Moon;
    }

    public sealed class MoonDebugTargetContractOnMoonRt : MoonDebugTargetContract
    {
        protected override string Engine => NesDebugRig.MoonRt;
    }
}
