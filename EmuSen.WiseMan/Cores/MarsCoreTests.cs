using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Cheats;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.WiseMan.Cores
{
    // Mars behind ICore: the frame boundary, the size contract and the stubs - see Mars_Core.md §9.
    public class MarsCoreTests : IDisposable
    {
        // beq zero, zero, -1 and its empty delay slot: the first instruction the handoff runs, forever.
        private static readonly byte[] SpinForever = { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 };

        private const uint NtscSync = 525, NtscLine = 3093, PalSync = 625, PalLine = 3177;

        private const uint Framebuffer = 0x0020_0000;

        private const string WaveRace = "Wave Race 64 (USA) (Rev A).z64";

        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch { }
            }
        }

        private string WriteRom(byte[] image, string extension = ".z64")
        {
            string path = SyntheticN64Rom.WriteTemp(image, extension);
            _temporaryFiles.Add(path);
            return path;
        }

        private MarsCore Load()
        {
            var core = new MarsCore();
            core.LoadRom(WriteRom(SyntheticN64Rom.Build(patches: (0, SpinForever))));
            return core;
        }

        // What a game's boot does to the VI, done from outside so the ROM itself can stay a two-instruction loop.
        private static void ProgramVi(MemoryBus bus, uint sync, uint line, bool serrate = false)
        {
            bus.Write32(MemoryMap.ViBase + VideoInterface.Control, serrate ? 1u << 6 : 0);
            bus.Write32(MemoryMap.ViBase + VideoInterface.VerticalSync, sync);
            bus.Write32(MemoryMap.ViBase + VideoInterface.HorizontalSync, line);
        }

        [Theory]
        [InlineData(".z64")]
        [InlineData(".n64")]
        [InlineData(".v64")]
        public void The_factory_routes_every_n64_extension_to_mars(string extension)
        {
            string path = WriteRom(SyntheticN64Rom.Build(), extension);

            Assert.True(CoreFactory.IsSupported(path));
            Assert.Equal("N64", CoreCatalog.ConsoleForRom(path));

            CoreBundle bundle = CoreFactory.Load(path);
            Assert.IsType<MarsCore>(bundle.Core);
            Assert.IsType<MarsDebugTarget>(bundle.DebugTarget);
            Assert.IsType<N64GameSharkCheatCodec>(bundle.CheatAutoDetectCodec);
            Assert.Null(bundle.CheatExplicitCodec);
            Assert.True(bundle.Core.IsRomLoaded);
        }

        // The magic word decides the container, so a dump named for the wrong order still loads - see Mars_Rom.md §1.1.
        [Fact]
        public void The_extension_does_not_decide_the_byte_order()
        {
            byte[] bigEndian = SyntheticN64Rom.Build();
            string path = WriteRom(SyntheticN64Rom.ToLittleEndian(bigEndian), ".v64");

            var core = (MarsCore)CoreFactory.Load(path).Core;

            Assert.Equal(RomByteOrder.LittleEndian, core.Rom!.SourceByteOrder);
            Assert.Equal("WISEMAN", core.Rom.Title);
        }

        [Fact]
        public void The_catalog_reaches_mars_by_console_name_and_codename()
        {
            Assert.Same(CoreCatalog.Registry["n64"], CoreCatalog.Registry["mars"]);
            Assert.Equal("N64", CoreCatalog.Registry["mars"].Console);
            Assert.Equal(new MarsCore().CoreName, CoreCatalog.Registry["mars"].Console);
            Assert.Contains("Nintendo - Nintendo 64", CoreCatalog.SupportedCheatSystems);
            Assert.Equal(MarsCore.PadButtons, CoreCatalog.ButtonsFor("N64"));

            var codecs = CoreFactory.CheatCodecsFor("N64");
            Assert.IsType<N64GameSharkCheatCodec>(codecs.AutoDetect);
            Assert.Null(codecs.Explicit);
        }

        // A game that never programs the VI still has to give the frame back - see Mars_Core.md §3.
        [Fact]
        public void A_frame_ends_on_the_cycle_cap_when_the_vi_is_never_programmed()
        {
            MarsCore core = Load();
            long before = core.Bus!.Cycles;

            core.RunFrame();

            long spent = core.Bus.Cycles - before;
            Assert.InRange(spent, MarsCore.CycleCap, MarsCore.CycleCap + 8);
            Assert.Equal(0, core.Bus.Vi.Fields);
            Assert.Equal(1, core.TotalFrames);
            Assert.Equal(MarsCore.ProcessorClockHz / (double)spent, core.FrameRateHz);
        }

        // Once the VI counts, a frame is exactly one of its fields - see Mars_Core.md §3.
        [Fact]
        public void A_frame_ends_on_the_vis_own_field_once_it_counts()
        {
            MarsCore core = Load();
            MemoryBus bus = core.Bus!;
            ProgramVi(bus, NtscSync, NtscLine);

            core.RunFrame();
            Assert.Equal(1, bus.Vi.Fields);

            long before = bus.Cycles;
            core.RunFrame();
            long spent = bus.Cycles - before;

            // 525 half lines of 3093 interface clocks at 48.681818 MHz, in 93.75 MHz processor cycles.
            double field = NtscSync * NtscLine * 93_750_000.0 / (2 * 48_681_818.0);

            Assert.Equal(2, bus.Vi.Fields);
            Assert.InRange(spent, (long)field - 2, (long)field + 2);
            Assert.InRange(core.FrameRateHz, 59.95, 59.97);
        }

        // The longest field the registers can describe still ends on the VI, not the cap - see Mars_Core.md §3.
        [Fact]
        public void The_cap_never_ends_a_frame_the_vi_would_have_ended()
        {
            MarsCore core = Load();
            MemoryBus bus = core.Bus!;
            ProgramVi(bus, 0x3FF, 0xFFF);

            long before = bus.Cycles;
            core.RunFrame();

            Assert.Equal(1, bus.Vi.Fields);
            Assert.True(bus.Cycles - before < MarsCore.CycleCap,
                $"a {bus.Cycles - before}-cycle field reached the {MarsCore.CycleCap}-cycle cap");
        }

        // Length and dimensions are set together, whatever the signal - see Mars_Core.md §2.
        [Fact]
        public void The_frame_buffer_is_always_as_long_as_the_screen_it_claims()
        {
            var core = new MarsCore();
            AssertConsistent(core, 480);

            core.LoadRom(WriteRom(SyntheticN64Rom.Build(patches: (0, SpinForever))));
            AssertConsistent(core, 480);

            core.RunFrame();
            AssertConsistent(core, 480);

            ProgramVi(core.Bus!, PalSync, PalLine);
            core.RunFrame();
            AssertConsistent(core, 576);

            ProgramVi(core.Bus!, NtscSync - 1, NtscLine, serrate: true);
            core.RunFrame();
            AssertConsistent(core, 480);

            static void AssertConsistent(MarsCore core, int height)
            {
                Assert.Equal(640, core.ScreenWidth);
                Assert.Equal(height, core.ScreenHeight);
                Assert.Equal(core.ScreenWidth * core.ScreenHeight * 4, core.GetFrameBufferRgba().Length);
            }
        }

        // The raster's fourth byte is coverage; a frontend drawing it as alpha shows a near-transparent picture - see Mars_Core.md §2.1.
        [Fact]
        public void Every_pixel_is_opaque_whatever_coverage_the_raster_carries()
        {
            MarsCore core = Load();
            MemoryBus bus = core.Bus!;

            for (uint i = 0; i < 256 * 120 * 2; i += 2) bus.Write16(Framebuffer + i, (ushort)(0x1235 + i * 0x0421));

            ProgramVi(bus, NtscSync, NtscLine);
            bus.Write32(MemoryMap.ViBase + VideoInterface.Control, 2 | (3 << 8));
            bus.Write32(MemoryMap.ViBase + VideoInterface.Origin, Framebuffer);
            bus.Write32(MemoryMap.ViBase + VideoInterface.Width, 256);
            bus.Write32(MemoryMap.ViBase + VideoInterface.HorizontalStart, (108 << 16) | (108 + 256));
            bus.Write32(MemoryMap.ViBase + VideoInterface.VerticalStart, (34 << 16) | (34 + 240));
            bus.Write32(MemoryMap.ViBase + VideoInterface.ScaleX, 0x400);
            bus.Write32(MemoryMap.ViBase + VideoInterface.ScaleY, 0x400);

            core.RunFrame();

            byte[] raster = bus.Vi.Frame.ToArray();
            byte[] frame = core.GetFrameBufferRgba();

            // The test is only worth something if the raster really carries a coverage, and here it is 7.
            Assert.Contains(raster.Where((_, i) => i % 4 == 3), a => a == 7);
            Assert.Contains(raster.Where((_, i) => i % 4 != 3), c => c != 0);

            // A progressive field is every other line, so each raster row lands twice, its colour untouched and its alpha whole.
            int rowBytes = 640 * 4;
            var expected = new byte[raster.Length * 2];
            for (int row = 0; row < bus.Vi.FrameHeight; row++)
            {
                for (int copy = 0; copy < 2; copy++)
                {
                    int at = (row * 2 + copy) * rowBytes;
                    Array.Copy(raster, row * rowBytes, expected, at, rowBytes);
                    for (int alpha = at + 3; alpha < at + rowBytes; alpha += 4) expected[alpha] = 0xFF;
                }
            }

            Assert.Equal(expected, frame);
        }

        [Fact]
        public void A_skipped_frame_leaves_the_last_picture_alone()
        {
            MarsCore core = Load();
            byte[] before = core.GetFrameBufferRgba();

            core.SkipRendering = true;
            core.RunFrame();

            Assert.Same(before, core.GetFrameBufferRgba());
            Assert.Equal(1, core.TotalFrames);
        }

        // The joybus answers a state command with what SetButton left - see Mars_Serial.md §3.1.
        [Fact]
        public void A_pressed_button_reaches_the_state_the_joybus_reports()
        {
            MarsCore core = Load();

            core.SetButton(0, PadButton.A, true);
            core.SetButton(0, PadButton.Start, true);
            core.SetButton(0, PadButton.Right, true);
            core.SetButton(0, PadButton.L, true);
            core.SetButton(0, PadButton.R, true);

            Assert.Equal(new byte[] { 0x91, 0x30 }, StateReply(core.Bus!));

            core.SetButton(0, PadButton.A, false);
            core.SetButton(0, PadButton.Up, true);
            core.SetButton(0, PadButton.B, true);

            Assert.Equal(new byte[] { 0x59, 0x30 }, StateReply(core.Bus!));
        }

        // X, Y and Select have no N64 twin, and are dropped rather than moved onto Z or a C button - see Mars_Core.md §5.
        [Fact]
        public void A_button_the_pad_lacks_changes_nothing()
        {
            MarsCore core = Load();

            foreach (PadButton button in new[] { PadButton.X, PadButton.Y, PadButton.Select })
            {
                core.SetButton(0, button, true);
            }

            core.SetButton(7, PadButton.A, true);
            core.SetButton(-1, PadButton.A, true);

            Assert.All(core.Bus!.Si.Controllers, c => Assert.Equal(0, c.Buttons));
            Assert.Equal(new byte[] { 0x00, 0x00 }, StateReply(core.Bus!));
            Assert.DoesNotContain(PadButton.Select, core.SupportedButtons);
        }

        // A counter the processor keeps in RDRAM, so every frame changes the machine - see Mars_SaveStates.md §4.
        private static readonly byte[] CountForever =
        {
            0x3C, 0x04, 0xA0, 0x10, // lui   a0, 0xA010
            0x8C, 0x88, 0x00, 0x00, // lw    t0, 0(a0)
            0x25, 0x08, 0x00, 0x01, // addiu t0, t0, 1
            0xAC, 0x88, 0x00, 0x00, // sw    t0, 0(a0)
            0x10, 0x00, 0xFF, 0xFC, // b     the lw
            0x00, 0x00, 0x00, 0x00, // nop
        };

        private MarsCore Counting()
        {
            var core = new MarsCore(batteryRamDisabled: true);
            core.LoadRom(WriteRom(SyntheticN64Rom.Build(patches: (0, CountForever))));
            ProgramVi(core.Bus!, 0x20, 0x40);
            return core;
        }

        // A snapshot is version 2 and loads like a state; a state keeps version 1 and its bytes - see Mars_SaveStates.md §1.
        [Fact]
        public void A_snapshot_loads_like_a_state_and_a_state_keeps_its_version()
        {
            MarsCore saver = Counting();
            for (int i = 0; i < 20; i++) saver.RunFrame();

            using var state = new MemoryStream();
            saver.SaveState(state);
            using var snapshot = new MemoryStream();
            saver.SaveSnapshot(snapshot);

            Assert.Equal(1, BitConverter.ToInt32(state.GetBuffer(), 4));
            Assert.Equal(2, BitConverter.ToInt32(snapshot.GetBuffer(), 4));
            Assert.Equal(state.Length + 4 + 8 * EmuSen.Cores.Nintendo.Mars.Memory.DpInterface.SnapshotWords, snapshot.Length);

            snapshot.Position = 0;
            MarsCore loader = Counting();
            loader.LoadState(snapshot);

            for (int i = 0; i < 20; i++)
            {
                saver.RunFrame();
                loader.RunFrame();
                Assert.Equal(saver.Bus!.Cycles, loader.Bus!.Cycles);
                Assert.Equal(saver.Cpu!.Pc, loader.Cpu!.Pc);
            }
        }

        // A fresh core that loads a state keeps step with the one that saved it - see Mars_SaveStates.md §4.
        [Fact]
        public void A_fresh_core_loaded_from_a_state_keeps_step_with_the_one_that_saved_it()
        {
            MarsCore saver = Counting();
            for (int i = 0; i < 20; i++) saver.RunFrame();

            using var state = new MemoryStream();
            saver.SaveState(state);
            state.Position = 0;

            MarsCore loader = Counting();
            loader.RunFrame();
            loader.LoadState(state);

            for (int i = 0; i < 20; i++)
            {
                saver.RunFrame();
                loader.RunFrame();

                Assert.Equal(saver.Bus!.Cycles, loader.Bus!.Cycles);
                Assert.Equal(saver.Cpu!.Pc, loader.Cpu!.Pc);
                Assert.Equal(saver.Bus.Read32(0x0010_0000), loader.Bus.Read32(0x0010_0000));
            }

            Assert.Equal(saver.TotalFrames, loader.TotalFrames);
            Assert.True(saver.Bus!.Read32(0x0010_0000) > 1000);
        }

        // The same count, with the timer set first to a value it reaches between the save and the end - see Mars_Performance.md §10.
        private static readonly byte[] CountWithTimer =
        {
            0x24, 0x09, 0x70, 0x00, // addiu t1, zero, 0x7000
            0x40, 0x89, 0x58, 0x00, // mtc0  t1, Compare
            0x3C, 0x04, 0xA0, 0x10, // lui   a0, 0xA010
            0x8C, 0x88, 0x00, 0x00, // lw    t0, 0(a0)
            0x25, 0x08, 0x00, 0x01, // addiu t0, t0, 1
            0xAC, 0x88, 0x00, 0x00, // sw    t0, 0(a0)
            0x10, 0x00, 0xFF, 0xFC, // b     the lw
            0x00, 0x00, 0x00, 0x00, // nop
        };

        // A fresh core's timer was scheduled against the Compare its own boot set; the state's is another - see Mars_Performance.md §10.
        [Fact]
        public void A_fresh_core_loaded_from_a_state_raises_the_timer_where_the_saver_did()
        {
            MarsCore Timed()
            {
                var core = new MarsCore(batteryRamDisabled: true);
                core.LoadRom(WriteRom(SyntheticN64Rom.Build(patches: (0, CountWithTimer))));
                ProgramVi(core.Bus!, 0x20, 0x40);
                return core;
            }

            static ulong TimerLine(MarsCore core) => core.Cpu!.Cop0[EmuSen.Cores.Nintendo.Mars.Cpu.Core.Cpu.CauseRegister] & EmuSen.Cores.Nintendo.Mars.Cpu.Core.Cpu.CauseInterruptTimer;

            MarsCore saver = Timed();
            for (int i = 0; i < 20; i++) saver.RunFrame();
            Assert.Equal(0UL, TimerLine(saver));

            using var state = new MemoryStream();
            saver.SaveState(state);
            state.Position = 0;

            MarsCore loader = Timed();
            loader.LoadState(state);

            for (int i = 0; i < 20; i++)
            {
                saver.RunFrame();
                loader.RunFrame();
                Assert.Equal(TimerLine(saver), TimerLine(loader));
            }

            Assert.Equal(EmuSen.Cores.Nintendo.Mars.Cpu.Core.Cpu.CauseInterruptTimer, TimerLine(saver));
        }

        // A file that begins "MARS", and a load that puts back what came after - see Mars_SaveStates.md §1.
        [Fact]
        public void A_state_file_round_trips_the_machine()
        {
            MarsCore core = Counting();
            for (int i = 0; i < 5; i++) core.RunFrame();
            uint counted = core.Bus!.Read32(0x0010_0000);

            string path = Path.Combine(Path.GetTempPath(), $"wiseman_{Guid.NewGuid():N}.state");
            _temporaryFiles.Add(path);
            core.SaveState(path);

            for (int i = 0; i < 5; i++) core.RunFrame();
            core.LoadState(path);

            Assert.Equal("MARS"u8.ToArray(), File.ReadAllBytes(path)[..4]);
            Assert.Equal(counted, core.Bus.Read32(0x0010_0000));
            Assert.Equal(5, core.TotalFrames);
        }

        // A state is the machine it was made on: one with the other amount of memory rebuilds this machine to it - see Mars_SaveStates.md §1.
        [Fact]
        public void A_state_made_with_the_pak_rebuilds_a_stock_machine_to_it_and_the_reverse()
        {
            var expanded = new MarsCore(expansionPak: true, batteryRamDisabled: true);
            expanded.LoadRom(WriteRom(SyntheticN64Rom.Build(patches: (0, CountForever))));
            expanded.Bus!.Write32(0x0060_0000, 0xCAFE_F00D);
            using var withPak = new MemoryStream();
            expanded.SaveState(withPak);

            MarsCore stock = Counting();
            stock.Bus!.Write32(0x0010_0000, 0x1234_5678);
            using var withoutPak = new MemoryStream();
            stock.SaveState(withoutPak);

            withPak.Position = 0;
            stock.LoadState(withPak);
            Assert.Equal(MemoryBus.RdramSizeExpanded, stock.Bus!.Rdram.Length);
            Assert.True(stock.ExpansionPak);
            Assert.Equal(0xCAFE_F00Du, stock.Bus.Read32(0x0060_0000));

            withoutPak.Position = 0;
            expanded.LoadState(withoutPak);
            Assert.Equal(MemoryBus.RdramSize, expanded.Bus!.Rdram.Length);
            Assert.Equal(0x1234_5678u, expanded.Bus.Read32(0x0010_0000));
        }

        // Refused before anything is read into this machine, which is left as it was - see Mars_SaveStates.md §1.
        [Fact]
        public void A_state_that_is_not_one_is_refused_before_anything_is_read()
        {
            MarsCore stock = Counting();
            stock.Bus!.Write32(0x0010_0000, 0x1234_5678);

            Assert.Throws<InvalidDataException>(() => stock.LoadState(new MemoryStream(new byte[] { 0x53, 0x45, 0x4E, 0x53, 1, 0, 0, 0 })));

            using var state = new MemoryStream();
            stock.SaveState(state);
            byte[] bytes = state.ToArray();
            BitConverter.GetBytes(0x0030_0000).CopyTo(bytes, 8);
            Assert.Throws<InvalidDataException>(() => stock.LoadState(new MemoryStream(bytes)));
            Assert.Equal(MemoryBus.RdramSize, stock.Bus.Rdram.Length);
            Assert.Equal(0x1234_5678u, stock.Bus.Read32(0x0010_0000));
        }

        // Before the first frame nothing has read the memory's size, so the machine is rebuilt; after it the change waits for the next load - see Mars_Core.md §7.
        [Fact]
        public void The_pak_set_before_the_first_frame_rebuilds_the_machine_and_after_it_waits_for_a_load()
        {
            string rom = WriteRom(SyntheticN64Rom.Build(patches: (0, CountForever)));
            var core = new MarsCore(batteryRamDisabled: true);
            core.LoadRom(rom);
            core.ExpansionPak = true;
            Assert.Equal(MemoryBus.RdramSizeExpanded, core.Bus!.Rdram.Length);

            core.RunFrame();
            core.ExpansionPak = false;
            Assert.Equal(MemoryBus.RdramSizeExpanded, core.Bus.Rdram.Length);
            Assert.False(core.ExpansionPak);

            core.LoadRom(rom);
            Assert.Equal(MemoryBus.RdramSize, core.Bus!.Rdram.Length);
        }

        // The warm boot path skips the memory sizing, so the hand-off leaves what it would have: the size where libultra reads it, or where the 6105's boot code copies it from - see Mars_Boot.md §6.5.
        [Theory]
        [InlineData(false, EmuSen.Cores.Nintendo.Mars.Rom.CicChip.Nus6102, 0x318u, 0x0040_0000u)]
        [InlineData(true, EmuSen.Cores.Nintendo.Mars.Rom.CicChip.Nus6102, 0x318u, 0x0080_0000u)]
        [InlineData(true, EmuSen.Cores.Nintendo.Mars.Rom.CicChip.Unknown, 0x318u, 0x0080_0000u)]
        [InlineData(false, EmuSen.Cores.Nintendo.Mars.Rom.CicChip.Nus6105, 0x3F0u, 0x0040_0000u)]
        [InlineData(true, EmuSen.Cores.Nintendo.Mars.Rom.CicChip.Nus6105, 0x3F0u, 0x0080_0000u)]
        public void The_hand_off_leaves_the_memory_size_a_cold_boot_would_have_left(bool pak, EmuSen.Cores.Nintendo.Mars.Rom.CicChip chip, uint at, uint size)
        {
            var rom = EmuSen.Cores.Nintendo.Mars.Rom.RomImage.Load(WriteRom(SyntheticN64Rom.Build()));
            rom.CicChip = chip;
            var bus = new MemoryBus(pak);

            Boot.HandOff(bus, new EmuSen.Cores.Nintendo.Mars.Cpu.Core.Cpu(bus), rom);

            Assert.Equal(size, bus.Read32(at));
            Assert.Equal(0u, bus.Read32(at == 0x318u ? 0x3F0u : 0x318u));
        }

        // A frontend's machine has the Pak, because the factory asks for it - see Mars_Core.md §7.
        [Fact]
        public void The_factorys_machine_has_the_pak()
        {
            var core = (MarsCore)global::EmuSen.Cores.CoreFactory.Create("game.z64");
            Assert.True(core.ExpansionPak);
        }

        // Reflection fills only what exists, so the chip is rebuilt before its bytes are read - see Mars_SaveStates.md §3.
        [Fact]
        public void A_state_carries_the_save_chip_the_pak_and_the_unmodelled_registers()
        {
            MarsCore saver = Counting();
            MemoryBus bus = saver.Bus!;
            bus.Save = new SaveChip(N64SaveType.Eeprom16k);
            bus.Save.Eeprom!.Data[9] = 0x5A;
            bus.Si.Controllers[0].Pak = new ControllerPak(null);
            bus.Si.Controllers[0].Pak!.Data[0x4000] = 0x77;
            bus.Write32(MemoryMap.RiBase + 4, 0xCAFE_F00D);

            using var state = new MemoryStream();
            saver.SaveState(state);
            state.Position = 0;

            MarsCore loader = Counting();
            loader.Bus!.Si.Controllers[0].Pak = null;
            loader.LoadState(state);

            Assert.Equal(N64SaveType.Eeprom16k, loader.Bus.Save.Type);
            Assert.Equal(0x5A, loader.Bus.Save.Eeprom!.Data[9]);
            Assert.True(loader.Bus.Save.Dirty);
            Assert.Equal(0x77, loader.Bus.Si.Controllers[0].Pak!.Data[0x4000]);
            Assert.True(loader.Bus.Si.Controllers[0].Pak!.Dirty);
            Assert.Equal(0xCAFE_F00Du, loader.Bus.Read32(MemoryMap.RiBase + 4));
        }

        // Samples a frontend has not drained belong to the moment before the load - see Mars_SaveStates.md §2.
        [Fact]
        public void Loading_a_state_drops_the_audio_a_frontend_has_not_drained()
        {
            MarsCore core = Counting();
            using var state = new MemoryStream();
            core.SaveState(state);

            core.Bus!.Write32(MemoryMap.AiBase + AiInterface.DacRate, 0x1000);
            core.Bus.Write32(MemoryMap.AiBase + AiInterface.Control, 1);
            core.Bus.Write32(MemoryMap.AiBase + AiInterface.DramAddress, 0x0020_0000);
            core.Bus.Write32(MemoryMap.AiBase + AiInterface.Length, 0x2000);
            for (int i = 0; i < 200; i++) core.RunFrame();
            Assert.True(core.Bus.Ai.BufferedSamples > 0);

            state.Position = 0;
            core.LoadState(state);

            Assert.Equal(0, core.Bus.Ai.BufferedSamples);
        }

        // Real snapshots now, so holding rewind steps back to an earlier machine - see EmuSen_Rewind_And_FastForward.md §1.7.
        [Fact]
        public void Mars_gives_the_rewind_buffer_history_it_can_step_back_through()
        {
            MarsCore core = Counting();
            var rewind = new RewindBuffer { Enabled = true, IntervalFrames = 1 };

            core.RunFrame();
            rewind.CaptureNow(core);
            uint earlier = core.Bus!.Read32(0x0010_0000);

            for (int i = 0; i < 3; i++)
            {
                core.RunFrame();
                rewind.CaptureNow(core);
            }

            Assert.Equal(3, rewind.Depth);
            while (rewind.Rewind(core)) { }

            Assert.Equal(earlier, core.Bus.Read32(0x0010_0000));
        }

        [Fact]
        public void A_game_that_never_starts_the_audio_interface_queues_nothing_at_the_default_rate()
        {
            MarsCore core = Load();
            core.RunFrame();

            Assert.Equal(AiInterface.DefaultSampleRate, core.AudioSampleRate);
            Assert.Empty(core.DequeueAudioSamples(int.MaxValue));
        }

        // What the audio interface plays reaches a frontend through ICore, at the rate the game set - see Mars_Core.md §4.
        [Fact]
        public void What_the_audio_interface_plays_reaches_the_speaker_at_the_games_rate()
        {
            MarsCore core = Load();
            MemoryBus bus = core.Bus!;
            bus.Write32(0x0010_0000, 0x1234_5678);
            bus.Write32(MemoryMap.AiBase + AiInterface.DacRate, 1520);
            bus.Write32(MemoryMap.AiBase + AiInterface.Control, 1);
            bus.Write32(MemoryMap.AiBase + AiInterface.DramAddress, 0x0010_0000);
            bus.Write32(MemoryMap.AiBase + AiInterface.Length, 0x1000);

            core.RunFrame();
            short[] samples = core.DequeueAudioSamples(int.MaxValue);

            Assert.Equal((int)System.Math.Round(bus.Vi.VideoClock / 1521.0), core.AudioSampleRate);
            Assert.NotEmpty(samples);
            Assert.Equal(new short[] { 0x1234, 0x5678 }, samples[..2]);
        }

        // Nothing changed, so flushing writes nothing, beside the ROM or anywhere - see Mars_Save.md §7.
        [Fact]
        public void Flushing_save_data_writes_nothing()
        {
            var core = new MarsCore();
            string path = WriteRom(SyntheticN64Rom.Build(patches: (0, SpinForever)));
            core.LoadRom(path);

            string directory = Path.GetDirectoryName(path)!;
            string stem = Path.GetFileNameWithoutExtension(path);
            core.SaveSram();

            Assert.Equal(new[] { path }, Directory.GetFiles(directory, stem + "*"));
        }

        [Fact]
        public void The_default_machine_is_a_stock_console()
        {
            MarsCore stock = Load();
            Assert.Equal(MemoryBus.RdramSize, stock.Bus!.Rdram.Length);

            var expanded = new MarsCore(expansionPak: true);
            expanded.LoadRom(WriteRom(SyntheticN64Rom.Build()));
            Assert.Equal(MemoryBus.RdramSizeExpanded, expanded.Bus!.Rdram.Length);
        }

        [Fact]
        public void Every_member_that_needs_a_rom_says_so_before_one_is_loaded()
        {
            var core = new MarsCore();

            Assert.Throws<InvalidOperationException>(() => core.RunFrame());
            Assert.Throws<InvalidOperationException>(() => core.SaveState(new MemoryStream()));
            Assert.Throws<InvalidOperationException>(() => core.LoadState(new MemoryStream()));
            Assert.Throws<InvalidOperationException>(() => core.SaveState("never.state"));

            core.SetButton(0, PadButton.A, true);
            core.SaveSram();
            Assert.False(core.IsRomLoaded);
        }

        // The debug target is what lets CoreFactory hand Mars out; its memory spaces are the machine's own arrays - see Mars_Core.md §8.
        [Fact]
        public void The_debug_target_reads_the_machine_it_was_built_on()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(WriteRom(SyntheticN64Rom.Build(patches: (0, SpinForever))), cheats: cheats);
            var core = (MarsCore)bundle.Core;
            var target = bundle.DebugTarget;

            Assert.Same(cheats, target.Cheats);
            Assert.Equal("N64", target.CoreName);

            var rom = target.GetMemorySpaces().Single(s => s.Name == "ROM");
            Assert.False(rom.IsWritable);
            Assert.Equal(0x10, rom.Read(RomImage.HeaderLength));

            var rdram = target.GetMemorySpaces().Single(s => s.Name == "RDRAM");
            rdram.Write(0x100, 0xAB);
            Assert.Equal(0xAB, core.Bus!.Rdram[0x100]);

            target.RefreshProviders();
            Assert.Contains(target.CpuRegisters.Current, r => r.Name == "PC" && r.Value == core.Cpu!.Pc);
            Assert.Equal(4, target.Disassemble("RDRAM", 0, 4).Count);
            Assert.Contains("N64 (Mars)", target.GetSummaryText());
        }

        // A commercial game through ICore alone: fields at the NTSC rate and a picture a frontend can draw - see Mars_Core.md §9.
        [Fact]
        public void Wave_Race_reaches_a_picture_through_the_interface_alone()
        {
            string path = Path.Combine(N64TestRomLibrary.Root, WaveRace);
            if (!File.Exists(path)) return;

            ICore core = CoreFactory.Load(path).Core;

            int frames = 0;
            byte[] rgba = core.GetFrameBufferRgba();
            while (frames < 60 && !rgba.Where((_, i) => i % 4 != 3).Any(c => c != 0))
            {
                core.RunFrame();
                frames++;
                rgba = core.GetFrameBufferRgba();
                Assert.Equal(core.ScreenWidth * core.ScreenHeight * 4, rgba.Length);
            }

            Assert.True(frames < 60, "no frame of the first sixty carried a lit pixel");
            Assert.Equal(480, core.ScreenHeight);
            Assert.InRange(core.FrameRateHz, 59.95, 59.97);
            Assert.All(rgba.Where((_, i) => i % 4 == 3), a => Assert.Equal(0xFF, a));
        }

        // L2 is Z, the left stick is the stick with its Y turned over, and the right stick is the C buttons - see Mars_Core.md §5.
        [Fact]
        public void The_generic_pad_reaches_every_input_the_n64_controller_has()
        {
            MarsCore core = Load();

            core.SetButton(0, PadButton.L2, true);
            core.SetAxis(0, PadAxis.LeftX, 1.0);
            core.SetAxis(0, PadAxis.LeftY, 1.0);
            core.SetAxis(0, PadAxis.RightY, -0.6);
            core.SetAxis(0, PadAxis.RightX, -0.6);

            Assert.Equal(new byte[] { 0x20, 0x0A, 0x7F, 0x81 }, StateReply(core.Bus!, 4));

            core.SetAxis(0, PadAxis.LeftX, 0.5);
            core.SetAxis(0, PadAxis.LeftY, 0);
            core.SetAxis(0, PadAxis.RightY, -0.4);
            core.SetAxis(0, PadAxis.RightX, 0.6);

            Assert.Equal(new byte[] { 0x20, 0x01, 0x40, 0x00 }, StateReply(core.Bus!, 4));
        }

        [Fact]
        public void The_n64_reads_both_sticks_and_neither_trigger()
        {
            Assert.Equal(new[] { PadAxis.LeftX, PadAxis.LeftY, PadAxis.RightX, PadAxis.RightY }, MarsCore.PadAxes);
            Assert.Contains(PadButton.L2, MarsCore.PadButtons);
        }

        // A state command for the first port, run through the serial interface the way a game runs it - see Mars_Serial.md §2.
        private static byte[] StateReply(MemoryBus bus, int length = 2)
        {
            const uint Dram = 0x0010_0000;
            byte[] block = { 0x01, 0x04, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE };

            for (uint i = 0; i < 64; i++) bus.Rdram[Dram + i] = i < block.Length ? block[i] : (byte)0;
            bus.Rdram[Dram + 63] = 1;

            bus.Write32(MemoryMap.SiBase + SiInterface.DramAddress, Dram);
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressWrite, 0);
            bus.Write32(MemoryMap.SiBase + SiInterface.PifAddressRead, 0);

            // What the game reads back, since the PIF answers as its RAM goes out - see Mars_Serial.md §2.
            return bus.Rdram.AsSpan((int)Dram + 3, length).ToArray();
        }
    }
}
