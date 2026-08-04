using EmuSen.Cauldron;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The second IDebugTarget implementation this interface has ever had - see Moon_Debug.md.
    public class MoonDebugTargetTests : IDisposable
    {
        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private (MoonCore Core, MoonDebugTarget Target) Load(params (int Offset, byte[] Bytes)[] patches)
        {
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build(patches: patches));
            _temporaryFiles.Add(path);

            var core = new MoonCore();
            core.LoadRom(path);
            return (core, new MoonDebugTarget(core));
        }

        [Fact]
        public void The_target_names_the_console_and_its_sprite_capacity()
        {
            var (_, target) = Load();

            Assert.Equal("NES", target.CoreName);
            Assert.Equal(64, target.MaxSprites);
            Assert.Equal(1, target.TilemapEntryStride);
        }

        [Fact]
        public void Every_advertised_memory_space_is_readable_at_its_own_size()
        {
            var (_, target) = Load();
            IDebugTarget generic = target;

            var spaces = generic.GetMemorySpaces();
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
            var (core, target) = Load();

            var ram = target.GetMemorySpaces().First(s => s.Name == "RAM");
            ram.Write(0x0042, 0x99);

            Assert.Equal(0x99, core.Bus!.Ram[0x42]);
        }

        // PRG ROM is not writable, and a debug write must not quietly corrupt the loaded image.
        [Fact]
        public void A_read_only_space_refuses_writes()
        {
            var (_, target) = Load();

            var prg = target.GetMemorySpaces().First(s => s.Name == "PRGROM");
            byte before = prg.Read(0x10);
            prg.Write(0x10, (byte)(before ^ 0xFF));

            Assert.False(prg.IsWritable);
            Assert.Equal(before, prg.Read(0x10));
        }

        // Reading $2002 clears vblank and $2007 moves the pointer, so the bus space must declare it.
        [Fact]
        public void Only_the_cpu_bus_space_declares_side_effects()
        {
            var (_, target) = Load();

            foreach (var space in target.GetMemorySpaces())
            {
                Assert.Equal(space.Name == "CPUBUS", space.HasSideEffects);
            }
        }

        [Fact]
        public void Cpu_registers_report_the_live_machine()
        {
            var (core, target) = Load();

            core.Cpu!.A = 0x12;
            core.Cpu.X = 0x34;
            target.RefreshProviders();

            var registers = target.CpuRegisters.Current;
            Assert.Equal(0x12u, registers.First(r => r.Name == "A").Value);
            Assert.Equal(0x34u, registers.First(r => r.Name == "X").Value);
        }

        // The NES had no load bars at all until 2026-08-04 - see Moon_Debug.md §3.2.
        [Fact]
        public void Hardware_load_is_normalized_against_one_native_frame()
        {
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build());
            _temporaryFiles.Add(path);
            var core = new MoonCore();
            core.LoadRom(path);

            // ~16.639ms is one full frame at 60.0985Hz, so half of it reads as ~50%.
            var target = new MoonDebugTarget(core, () => (8.3195, 4.16));
            target.RefreshProviders();

            var load = target.HardwareLoad.Current;
            Assert.Equal(50.0, Assert.Single(load, l => l.Name == "CPU+APU").Percent, 1);
            Assert.Equal(25.0, Assert.Single(load, l => l.Name == "PPU").Percent, 1);

            // Wall-clock emulator timings, not guest load - see EmuSen_Cauldron.md §4.5.
            Assert.All(load, l => Assert.Equal(DebugLoadKind.EmulatorCost, l.Kind));
        }

        [Fact]
        public void Hardware_load_over_a_full_frame_is_clamped()
        {
            string path = SyntheticNesRom.WriteTemp(SyntheticNesRom.Build());
            _temporaryFiles.Add(path);
            var core = new MoonCore();
            core.LoadRom(path);

            var target = new MoonDebugTarget(core, () => (50.0, 50.0));
            target.RefreshProviders();

            Assert.All(target.HardwareLoad.Current, l => Assert.Equal(100.0, l.Percent));
        }

        // Real frames must produce real numbers, not the injected-only path above.
        [Fact]
        public void Running_frames_attributes_time_to_both_phases()
        {
            var (core, target) = Load();

            for (int i = 0; i < 3; i++) core.RunFrame();
            target.RefreshProviders();

            Assert.True(core.LastFrameCpuApuMs > 0, "the CPU+APU phase recorded no time");
            Assert.True(core.LastFramePpuMs > 0, "the PPU phase recorded no time");
            Assert.Equal(2, target.HardwareLoad.Current.Count);
        }

        // Guards the defaulted-no-op regression, not the refresh itself - see Moon_Debug.md §3.
        [Fact]
        public void Refreshing_through_the_interface_publishes_without_a_frame()
        {
            var (core, concrete) = Load();
            IDebugTarget target = concrete;

            core.Cpu!.A = 0x5A;
            target.RefreshProviders();

            Assert.Equal(0x5Au, target.CpuRegisters.Current.First(r => r.Name == "A").Value);
        }

        [Fact]
        public void Video_registers_expose_the_loopy_pair_by_name()
        {
            var (core, target) = Load();

            core.Ppu!.WriteRegister(6, 0x21);
            core.Ppu.WriteRegister(6, 0x08);
            target.RefreshProviders();

            var registers = target.VideoRegisters.Current;
            Assert.Equal(0x2108u, registers.First(r => r.Name == "v").Value);
        }

        [Fact]
        public void The_interrupt_vectors_are_the_three_the_6502_has()
        {
            var (_, target) = Load();
            IDebugTarget generic = target;

            var vectors = generic.InterruptVectors;
            Assert.Equal(3, vectors.Count);
            Assert.Contains(vectors, v => v.Address == 0xFFFA);
            Assert.Contains(vectors, v => v.Address == 0xFFFC);
            Assert.Contains(vectors, v => v.Address == 0xFFFE);
        }

        [Fact]
        public void Disassembly_decodes_real_instructions_out_of_a_space()
        {
            // LDA #$42 ; STA $0200 ; JSR $8010
            var (_, target) = Load((0, new byte[] { 0xA9, 0x42, 0x8D, 0x00, 0x02, 0x20, 0x10, 0x80 }));

            var listing = target.Disassemble("PRGROM", 0, 3);

            Assert.Equal("LDA", listing[0].Mnemonic);
            Assert.Equal("#$42", listing[0].OperandText);
            Assert.Equal(2, listing[0].Length);

            Assert.Equal("STA", listing[1].Mnemonic);
            Assert.Equal("$0200", listing[1].OperandText);

            Assert.Equal("JSR", listing[2].Mnemonic);
            Assert.Equal("$8010", listing[2].OperandText);
        }

        [Fact]
        public void A_relative_branch_resolves_to_its_target_address()
        {
            var (_, target) = Load((0, new byte[] { 0xD0, 0x10 })); // BNE +16

            var listing = target.Disassemble("PRGROM", 0, 1);

            Assert.Equal("BNE", listing[0].Mnemonic);
            Assert.Equal("$0012", listing[0].OperandText);
        }

        [Fact]
        public void Static_references_classify_calls_reads_and_writes()
        {
            var (_, target) = Load((0, new byte[] { 0x20, 0x10, 0x80, 0x8D, 0x00, 0x02, 0xAD, 0x34, 0x12 }));

            var listing = target.Disassemble("PRGROM", 0, 3);

            Assert.Equal((StaticReferenceKind.Call, 0x8010), target.ClassifyStaticReference(listing[0]));
            Assert.Equal((StaticReferenceKind.Write, 0x0200), target.ClassifyStaticReference(listing[1]));
            Assert.Equal((StaticReferenceKind.Read, 0x1234), target.ClassifyStaticReference(listing[2]));
        }

        // An indexed or indirect target is not knowable from the bytes, so it must report nothing.
        [Fact]
        public void A_runtime_dependent_operand_classifies_as_nothing()
        {
            var (_, target) = Load((0, new byte[] { 0x9D, 0x00, 0x02, 0xB1, 0x20 })); // STA $0200,X ; LDA ($20),Y

            var listing = target.Disassemble("PRGROM", 0, 2);

            Assert.Null(target.ClassifyStaticReference(listing[0]));
            Assert.Null(target.ClassifyStaticReference(listing[1]));
        }

        [Fact]
        public void A_tile_decodes_to_sixty_four_two_bit_pixels()
        {
            var (core, target) = Load();

            // Low plane row 0 all set, high plane row 0 alternating: colours 1 and 3.
            core.Cart!.Chr[0] = 0xFF;
            core.Cart.Chr[8] = 0xAA;

            var chr = target.GetMemorySpaces().First(s => s.Name == "CHR");
            byte[] pixels = target.DecodeTilePixels(chr, 0, 2);

            Assert.Equal(64, pixels.Length);
            Assert.Equal(3, pixels[0]);
            Assert.Equal(1, pixels[1]);
            Assert.All(pixels, p => Assert.InRange(p, 0, 3));
        }

        [Fact]
        public void An_unsupported_bit_depth_is_an_argument_error_not_garbage()
        {
            var (_, target) = Load();
            var chr = target.GetMemorySpaces().First(s => s.Name == "CHR");

            Assert.Throws<ArgumentException>(() => target.DecodeTilePixels(chr, 0, 4));
        }

        [Fact]
        public void A_nametable_entry_decodes_with_the_palette_its_attribute_byte_picks()
        {
            var (core, target) = Load();

            core.Ppu!.Ciram[0] = 0xA3;
            core.Ppu.Ciram[0x3C0] = 0x02; // top-left quadrant selects palette 2

            var ciram = target.GetMemorySpaces().First(s => s.Name == "CIRAM");

            Assert.Equal("$A3 p2", target.DecodeTilemapEntry(ciram, 0));
        }

        [Fact]
        public void The_attribute_table_region_is_labelled_as_such()
        {
            var (core, target) = Load();
            core.Ppu!.Ciram[0x3C0] = 0x1B;

            var ciram = target.GetMemorySpaces().First(s => s.Name == "CIRAM");

            Assert.Equal("attr $1B", target.DecodeTilemapEntry(ciram, 0x3C0));
        }

        [Fact]
        public void The_tile_sheet_and_palette_swatch_render_at_their_advertised_sizes()
        {
            var (_, target) = Load();

            var (sheetRgba, sheetWidth, sheetHeight) = target.RenderTileSheet();
            Assert.Equal(128, sheetWidth);
            Assert.Equal(sheetWidth * sheetHeight * 4, sheetRgba.Length);

            var (swatchRgba, swatchWidth, swatchHeight) = target.RenderPaletteSwatch();
            Assert.Equal(256, swatchWidth);
            Assert.Equal(32, swatchHeight);
            Assert.Equal(swatchWidth * swatchHeight * 4, swatchRgba.Length);
        }

        [Fact]
        public void Eight_palettes_of_four_colours_are_published()
        {
            var (_, target) = Load();
            target.RefreshProviders();

            var palettes = target.Palettes.Current;
            Assert.Equal(8, palettes.Count);
            Assert.All(palettes, p => Assert.Equal(4, p.Colors.Count));
        }

        [Fact]
        public void Sprites_parked_below_the_screen_are_left_out()
        {
            var (core, target) = Load();

            core.Ppu!.Oam[0] = 0x20;   // visible
            core.Ppu.Oam[3] = 0x40;
            core.Ppu.Oam[4] = 0xF0;    // parked off the bottom
            target.RefreshProviders();

            var sprites = target.Sprites.Current;
            Assert.Contains(sprites, s => s.Index == 0 && s.X == 0x40);
            Assert.DoesNotContain(sprites, s => s.Index == 1);
        }

        // The write observer is the seam the whole watch mechanism hangs off.
        [Fact]
        public void A_watch_records_a_write_the_cpu_makes_to_ram()
        {
            var (core, target) = Load();

            int id = target.Watches.AddWatch("RAM", 0x0042, 1);
            core.Bus!.Write(0x0042, 0x7E);

            var events = target.Watches.GetEvents(id);
            Assert.Single(events);
            Assert.Equal(0x42, events[0].Address);
            Assert.Equal(0x7E, events[0].Value);
        }

        [Fact]
        public void A_breakpoint_halts_the_frame_and_resuming_does_not_re_halt_on_it()
        {
            var (core, target) = Load();

            target.Breakpoints.AddBreakpoint(0x8000);
            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(0x8000, core.HaltedAddress);

            core.RunFrame();
            Assert.Equal(1, core.TotalFrames);
        }

        [Fact]
        public void Audio_channels_are_published_as_the_five_the_apu_has()
        {
            var (_, target) = Load();
            target.RefreshProviders();

            var channels = target.AudioChannels.Current;
            Assert.Equal(5, channels.Count);
            Assert.Contains(channels, c => c.Name == "Triangle");
        }

        [Fact]
        public void The_summary_names_the_mapper_and_the_current_frame()
        {
            var (core, target) = Load();
            core.RunFrame();

            string summary = target.GetSummaryText();

            Assert.Contains("NES (Moon)", summary);
            Assert.Contains("NROM", summary);
            Assert.Contains("CPU", summary);
        }

        [Fact]
        public void Physical_resolution_answers_for_fixed_ranges_and_declines_for_banked_ones()
        {
            var (_, target) = Load();
            IDebugTarget generic = target;

            Assert.Equal("RAM", generic.ResolvePhysical(0x0805)?.Space);
            Assert.Equal(0x005, generic.ResolvePhysical(0x0805)?.Offset);
            Assert.False(generic.ResolvePhysical(0x2002)?.IsAddressable);

            // $8000+ depends on live mapper state, so guessing would be worse than declining.
            Assert.Null(generic.ResolvePhysical(0x8000));
        }
    }
}
