using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // What a frontend needs of MarsRT beyond the machine: the factory's choice, settings, cheats, the debugger's view, and the frame it hands out - see Mars_Native.md §5.5.
    [Collection("MarsStatics")]
    public class MarsRtFrontendTests(Xunit.Abstractions.ITestOutputHelper output) : IDisposable
    {
        private readonly List<string> _temporary = new();

        // Status.IE set with nothing unmasked, then a spin, as CoreFactoryCheatWiringTests' cartridge.
        private static readonly byte[] InterruptsOnSpin =
        {
            0x3C, 0x08, 0x34, 0x00, 0x35, 0x08, 0x00, 0x01, 0x40, 0x88, 0x60, 0x00,
            0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00,
        };

        // The boot's Status left alone, so interrupts stay off, then a spin.
        private static readonly byte[] InterruptsOffSpin = { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 };

        public void Dispose()
        {
            foreach (string path in _temporary)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private string Rom(byte[] image, string extension = ".z64")
        {
            string path = SyntheticN64Rom.WriteTemp(image, extension);
            _temporary.Add(path);
            return path;
        }

        private string Cht(string code)
        {
            string path = Path.Combine(Path.GetTempPath(), $"wiseman_{Guid.NewGuid():N}.cht");
            File.WriteAllText(path, $"cheats = 1\n\ncheat0_desc = \"Poke\"\ncheat0_code = \"{code}\"\ncheat0_enable = false\n");
            _temporary.Add(path);
            return path;
        }

        [Theory]
        [InlineData(".z64")]
        [InlineData(".n64")]
        [InlineData(".v64")]
        public void The_factory_builds_MarsRT_only_when_its_engine_is_asked_for(string extension)
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Rom(SyntheticN64Rom.Build(patches: (0, InterruptsOffSpin)), extension);

            CoreBundle mars = CoreFactory.Load(rom);
            Assert.IsType<MarsCore>(mars.Core);
            Assert.Null(mars.Notice);

            CoreBundle marsRt = CoreFactory.Load(rom, engine: CoreCatalog.MarsRtEngine);
            Assert.IsType<MarsRtCore>(marsRt.Core);
            Assert.IsType<MarsRtDebugTarget>(marsRt.DebugTarget);
            Assert.IsType<EmuSen.Cores.Nintendo.Mars.Cheats.N64GameSharkCheatCodec>(marsRt.CheatAutoDetectCodec);
            Assert.Null(marsRt.Notice);
            Assert.True(((MarsRtCore)marsRt.Core).ExpansionPak);

            CoreBundle unknown = CoreFactory.Load(rom, engine: "MarsJS");
            Assert.IsType<MarsCore>(unknown.Core);
            Assert.Contains("MarsJS", unknown.Notice);
            Assert.Null(CoreFactory.Load(rom, engine: CoreCatalog.MarsEngine).Notice);
        }

        [Fact]
        public void Only_the_N64_has_an_engine_to_choose_and_its_default_is_the_csharp_Mars()
        {
            CoreSetting engine = CoreCatalog.EngineFor("N64")!;
            Assert.Equal(CoreCatalog.EngineKey, engine.Key);
            Assert.Equal(CoreCatalog.MarsEngine, engine.Default);
            Assert.Equal(new[] { CoreCatalog.MarsEngine, CoreCatalog.MarsRtEngine }, engine.Choices);
            foreach (string console in new[] { "SNES", "NES", "GB" }) Assert.Null(CoreCatalog.EngineFor(console));
            Assert.DoesNotContain(CoreCatalog.SettingsFor("N64"), s => s.Key == CoreCatalog.EngineKey);
        }

        [Fact]
        public void MarsRT_offers_Mars_s_keys_and_each_hint_says_whether_it_is_honoured()
        {
            using var core = new MarsRtCore();
            ICoreSettings settings = core;

            Assert.Equal(MarsCore.VideoSettings.Select(s => s.Key).Append("Recompiler"), settings.Settings.Select(s => s.Key));
            foreach (CoreSetting setting in settings.Settings)
            {
                if (setting.Key is "ExpansionPak" or "SkipRepeatedScans" or "RenderScale" or "Antialiasing" or "Gpu") Assert.Equal(MarsCore.VideoSettings.Single(s => s.Key == setting.Key).Hint, setting.Hint);
                else Assert.DoesNotContain("ignores it", setting.Hint);
            }
        }

        [Fact]
        public void Every_setting_is_checked_and_read_back_and_an_unknown_key_is_refused()
        {
            using var core = new MarsRtCore();
            ICoreSettings settings = core;

            foreach (CoreSetting setting in settings.Settings.Where(s => s.Key != "ExpansionPak")) Assert.Equal(setting.Default, settings.Get(setting.Key));
            settings.Set("RdpWorkers", "3");
            settings.Set("RenderScale", "4");
            settings.Set("Antialiasing", "2x");
            settings.Set("ThreadedRdp", "False");
            Assert.Equal("3", settings.Get("RdpWorkers"));
            Assert.Equal("4", settings.Get("RenderScale"));
            Assert.Equal("2x", settings.Get("Antialiasing"));
            Assert.Equal("false", settings.Get("ThreadedRdp"));
            settings.Set("RdpWorkers", "99");
            Assert.Equal("8", settings.Get("RdpWorkers"));
            Assert.Equal(4, core.RenderScale);
            Assert.Equal(2, core.Antialiasing);
            Assert.Equal(1, core.EffectiveAntialiasing);
            settings.Set("RenderScale", "2");
            Assert.Equal(2, core.EffectiveAntialiasing);
            settings.Set("Gpu", "True");
            Assert.Equal("true", settings.Get("Gpu"));
            Assert.True(core.Gpu);

            Assert.Throws<ArgumentException>(() => settings.Set("RdpWorkers", "many"));
            Assert.Throws<ArgumentException>(() => settings.Set("Antialiasing", "5x"));
            Assert.Throws<ArgumentException>(() => settings.Set("Gpu", "sometimes"));
            Assert.Throws<ArgumentException>(() => settings.Set("Engine", CoreCatalog.MarsRtEngine));
            Assert.Throws<ArgumentException>(() => settings.Get("Nothing"));
        }

        // The game reads the memory's size as it boots, so only a change before the first frame rebuilds - see EmuSen_Settings_Reference.md §4.26.
        [Fact]
        public void The_expansion_pak_rebuilds_MarsRT_before_its_first_frame_and_waits_after_it()
        {
            string rom = Rom(SyntheticN64Rom.Build(patches: (0, InterruptsOffSpin)));
            var core = (MarsRtCore)CoreFactory.Load(rom, engine: CoreCatalog.MarsRtEngine).Core;
            ICoreSettings settings = core;
            Assert.Equal(8 << 20, core.SpaceSize(MarsRtSpace.Rdram));

            settings.Set("ExpansionPak", "false");
            Assert.Equal(4 << 20, core.SpaceSize(MarsRtSpace.Rdram));
            Assert.Equal("false", settings.Get("ExpansionPak"));

            core.RunFrame();
            settings.Set("ExpansionPak", "true");
            Assert.Equal("true", settings.Get("ExpansionPak"));
            Assert.Equal(4 << 20, core.SpaceSize(MarsRtSpace.Rdram));
        }

        // A state of the other size rebuilds the machine, and the Pak follows the state, as MarsCore's does - see Mars_SaveStates.md §1.
        [Fact]
        public void A_state_of_the_other_memory_size_carries_the_pak_with_it()
        {
            string rom = Rom(SyntheticN64Rom.Build(patches: (0, InterruptsOffSpin)));
            using var small = new MarsRtCore(expansionPak: false, batteryRamDisabled: true);
            small.LoadRom(rom);
            small.RunFrame();
            byte[] state = small.Save(snapshot: false);

            using var large = new MarsRtCore(expansionPak: true, batteryRamDisabled: true);
            large.LoadRom(rom);
            large.LoadState(state);
            Assert.False(large.ExpansionPak);
            Assert.Equal(4 << 20, large.SpaceSize(MarsRtSpace.Rdram));
        }

        [Fact]
        public void A_cheat_imported_from_a_cht_file_lands_at_the_frame_s_end()
        {
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(Rom(SyntheticN64Rom.Build(patches: (0, InterruptsOnSpin))), cheats: cheats, engine: CoreCatalog.MarsRtEngine);
            var core = (MarsRtCore)bundle.Core;

            CheatImportResult imported = CheatImport.FromChtFile(cheats, Cht("80000100 0042+81000200 1234"), bundle.CheatAutoDetectCodec, replace: true);
            Assert.Equal(1, imported.Loaded);
            cheats.SetEnabled(cheats.GetCheats().Single().Id, true);
            Assert.Equal(0, core.Peek(MarsRtSpace.Rdram, 0x100));

            core.RunFrame();

            Assert.Equal(0x42, core.Peek(MarsRtSpace.Rdram, 0x100));
            Assert.Equal(new byte[] { 0x12, 0x34 }, core.ReadSpace(MarsRtSpace.Rdram, 0x200, 2));
            Assert.Same(cheats, bundle.DebugTarget.Cheats);
        }

        // Held while interrupts are off, or a cheat lands in the boot code's checksum; Apply from a paused frontend is not held - see Mars_Cheats.md §5.1.
        [Fact]
        public void While_interrupts_are_off_MarsRT_holds_a_cheat_as_Mars_does()
        {
            string rom = Rom(SyntheticN64Rom.Build(patches: (0, InterruptsOffSpin)));
            var cheats = new CheatRegistry();
            cheats.AddRamPoke(MarsCore.SpaceRdram, 0x100, 0x42, "poke");
            CoreBundle marsRt = CoreFactory.Load(rom, cheats: cheats, engine: CoreCatalog.MarsRtEngine);
            CoreBundle mars = CoreFactory.Load(rom, cheats: cheats);
            var core = (MarsRtCore)marsRt.Core;

            for (int frame = 0; frame < 3; frame++)
            {
                mars.Core.RunFrame();
                core.RunFrame();
            }

            Assert.Equal(0UL, core.Cop0(12) & 1);
            Assert.Equal(0, ((MarsCore)mars.Core).Bus!.Rdram[0x100]);
            Assert.Equal(0, core.Peek(MarsRtSpace.Rdram, 0x100));

            marsRt.DebugTarget.ApplyCheats();
            Assert.Equal(0x42, core.Peek(MarsRtSpace.Rdram, 0x100));
        }

        // Past the end of a memory reads zero and a write there is dropped, and every writable space the codec can name is reached - see Mars_Cheats.md §3.1.
        [Fact]
        public void A_cheat_reaches_every_writable_space_and_nothing_past_one_s_end()
        {
            var cheats = new CheatRegistry();
            var core = (MarsRtCore)CoreFactory.Load(Rom(SyntheticN64Rom.Build(patches: (0, InterruptsOnSpin))), cheats: cheats, engine: CoreCatalog.MarsRtEngine).Core;
            cheats.AddRamPoke("DMEM", 0xFFF, 0x11, "dmem");
            cheats.AddRamPoke("imem", 0x10, 0x22, "imem");
            cheats.AddRamPoke("PIFRAM", 0x3F, 0x33, "pif");
            cheats.AddRamPoke("RDRAM", 8 << 20, 0x44, "past the end");
            cheats.AddRamPoke("ROM", 0x40, 0x55, "no such space for a poke");

            core.ApplyCheats();

            Assert.Equal(0x11, core.Peek(MarsRtSpace.Dmem, 0xFFF));
            Assert.Equal(0x22, core.Peek(MarsRtSpace.Imem, 0x10));
            Assert.Equal(0x33, core.Peek(MarsRtSpace.PifRam, 0x3F));
            Assert.Equal(0, core.Peek(MarsRtSpace.Rdram, 8 << 20));
            Assert.NotEqual(0x55, core.Peek(MarsRtSpace.Rom, 0x40));
        }

        // The cheat writes into the frame buffer, so where it lands against the scan is seen in the picture as well as the state - see Mars_Native.md §5.5.
        [Fact]
        public void With_a_cheat_on_MarsRT_leaves_the_state_and_the_picture_Mars_leaves()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            string rom = Rom(SyntheticN64System.Build(rsp: true));
            var cheats = new CheatRegistry();
            cheats.AddCheat(CheatKind.RamPoke, new EmuSen.Cores.Nintendo.Mars.Cheats.N64GameSharkCheatCodec().DecodeWrites("8110FB40 FFFF+8110FB42 F801+80001003 0000")!, null, "stripe");

            MarsCore oracle = MarsRtTests.Oracle();
            using MarsRtCore twin = MarsRtTests.Twin();
            oracle.SkipRendering = twin.SkipRendering = false;
            oracle.Cheats = twin.Cheats = cheats;
            oracle.LoadRom(rom);
            twin.LoadRom(rom);

            bool drew = false;
            for (int frame = 1; frame <= 60; frame++)
            {
                oracle.RunFrame();
                twin.RunFrame();
                Assert.True(State(oracle).AsSpan().SequenceEqual(twin.Save(false)), $"frame {frame}: the states differ");
                byte[] want = oracle.GetFrameBufferRgba(), got = twin.GetFrameBufferRgba();
                Assert.True(want.AsSpan().SequenceEqual(got), $"frame {frame}: the pictures differ from byte {want.AsSpan().CommonPrefixLength(got)}");
                drew |= want.Where((b, i) => i % 4 != 3).Any(b => b != 0);
            }

            Assert.Equal(0, twin.Peek(MarsRtSpace.Rdram, 0x1003));
            Assert.Equal(0xFF, twin.Peek(MarsRtSpace.Rdram, 0x10FB40));
            Assert.True(drew, "no frame showed anything, so the picture compared nothing");
        }

        // ROM patches on the program the boot copies out, the data a transfer reads each field and the bytes the processor loads, changed between frames - see Mars_Native.md §6.6.1.
        [Fact]
        public void With_rom_patches_MarsRT_leaves_the_state_and_the_picture_Mars_leaves_frame_by_frame()
        {
            Assert.True(MarsRtCore.Available, MarsNative.Report);
            byte[] image = SyntheticN64System.Build(rsp: true);
            int origin = OnlyWord(image, 0x3C09_0010, SyntheticN64System.ImageAt, SyntheticN64System.ImageLength);
            const int Data = SyntheticN64System.DataAt;
            string rom = Rom(image);

            var cheats = new CheatRegistry();
            cheats.AddRomPatch(origin + 3, 0x00, null, "the picture's origin to zero");
            int run = cheats.AddCheat(CheatKind.RomPatch, new[] { new CheatWrite { Address = Data + 0x10, Value = 0xF00D, Width = 2, BigEndian = true, RepeatCount = 16, RepeatAddAddress = 0x100, RepeatAddValue = 0x11 } }, null, "a run through every transfer");
            cheats.AddRomPatch(Data + 3, 0x5C, image[Data + 3], "the byte load, compared true");
            cheats.AddRomPatch(Data + 6, 0x5D, (byte)(image[Data + 6] ^ 0xFF), "the halfword load, compared false");
            cheats.AddRomPatch(0x1005, 0xA5, null, "the word load past the latch");

            MarsCore oracle = MarsRtTests.Oracle();
            using MarsRtCore twin = MarsRtTests.Twin(), plain = MarsRtTests.Twin();
            oracle.SkipRendering = twin.SkipRendering = plain.SkipRendering = false;
            oracle.Cheats = twin.Cheats = cheats;
            oracle.LoadRom(rom);
            twin.LoadRom(rom);
            plain.LoadRom(rom);

            int statesApart = 0, picturesApart = 0;
            for (int frame = 1; frame <= 80; frame++)
            {
                if (frame == 25) cheats.SetEnabled(run, false);
                if (frame == 40) cheats.MasterEnabled = false;
                if (frame == 50)
                {
                    cheats.MasterEnabled = true;
                    cheats.AddRomPatch(Data + 0x210, 0x77, null, "added while running");
                }
                oracle.RunFrame();
                twin.RunFrame();
                plain.RunFrame();
                byte[] state = twin.Save(false);
                Assert.True(State(oracle).AsSpan().SequenceEqual(state), $"frame {frame}: the states differ");
                byte[] want = oracle.GetFrameBufferRgba(), got = twin.GetFrameBufferRgba();
                Assert.True(want.AsSpan().SequenceEqual(got), $"frame {frame}: the pictures differ from byte {want.AsSpan().CommonPrefixLength(got)}");
                statesApart += state.AsSpan().SequenceEqual(plain.Save(false)) ? 0 : 1;
                picturesApart += got.AsSpan().SequenceEqual(plain.GetFrameBufferRgba()) ? 0 : 1;
            }

            output.WriteLine($"80 frames identical to Mars; {statesApart} states and {picturesApart} pictures differ from MarsRT unpatched");
            Assert.Equal(image[origin + 3], twin.Peek(MarsRtSpace.Rom, (uint)origin + 3));
            Assert.True(statesApart > 60 && picturesApart > 60, $"the patches changed {statesApart} states and {picturesApart} pictures of 80, so the comparison tested little");
        }

        // The one offset in a stretch of the image where a big-endian word stands.
        private static int OnlyWord(byte[] image, uint word, int from, int length)
        {
            var at = new List<int>();
            for (int i = from; i + 4 <= from + length; i += 4)
                if ((uint)(image[i] << 24 | image[i + 1] << 16 | image[i + 2] << 8 | image[i + 3]) == word) at.Add(i);
            return Assert.Single(at);
        }

        // Since §6.13 the claim is the lending's: a buffer held is never written or lent again until it is returned, and a returned one is reused.
        [Fact]
        public void The_frame_handed_out_is_a_copy_the_next_frame_does_not_touch()
        {
            string rom = Rom(SyntheticN64System.Build(rsp: false));
            // Presented at once: the synthetic picture changes on alternate frames, and a deferred one shows the same alternation a frame late.
            using var core = new MarsRtCore(batteryRamDisabled: true) { DeferredPresentation = false };
            core.LoadRom(rom);
            for (int frame = 0; frame < 5; frame++) core.RunFrame();

            byte[] shown = core.GetFrameBufferRgba();
            byte[] kept = (byte[])shown.Clone();
            long serial = core.FrameSerial;
            core.RunFrame();

            Assert.NotEqual(serial, core.FrameSerial);
            Assert.False(core.GetFrameBufferRgba().AsSpan().SequenceEqual(kept), "the picture did not change, so the copy was not tested");
            Assert.Equal(kept, shown);
            Assert.NotSame(core.GetFrameBufferRgba(), core.GetFrameBufferRgba());

            // Others lent and returned while this one is held are reused, and it is not among them however many frames pass.
            long made = core.FrameBuffers.Made;
            for (int frame = 0; frame < 12; frame++)
            {
                core.RunFrame();
                byte[] other = core.GetFrameBufferRgba();
                Assert.NotSame(shown, other);
                core.ReturnFrameBuffer(other);
            }
            Assert.Equal(kept, shown);
            Assert.True(core.FrameBuffers.Made - made <= 1, $"{core.FrameBuffers.Made - made} arrays made for 12 frames returned each time");

            // Returned, it is lent again, holding the new picture rather than the old.
            core.ReturnFrameBuffer(shown);
            core.RunFrame();
            byte[] reused = core.GetFrameBufferRgba();
            byte[] fresh = core.GetFrameBufferRgba();
            Assert.True(ReferenceEquals(reused, shown) || ReferenceEquals(fresh, shown), "the returned array was not lent again");
            Assert.Equal(fresh, reused);
        }

        [Fact]
        public void Mars_s_debugger_commands_read_MarsRT_halt_it_and_step_it()
        {
            string rom = Rom(SyntheticN64Rom.Build(patches: (0, InterruptsOnSpin)));
            CoreBundle bundle = CoreFactory.Load(rom, engine: CoreCatalog.MarsRtEngine);
            IDebugTarget target = bundle.DebugTarget;
            bundle.Core.RunFrame();
            target.RefreshProviders();

            Assert.Contains("lui", new DisasmCommand().Execute(target, new[] { "disasm", "cpu", "A4000040", "4" }, null).Output);
            Assert.Contains("mtc0", new DisasmCommand().Execute(target, new[] { "disasm", "cpu", "A4000040", "4" }, null).Output);
            Assert.Contains("PC", new RegsCommand().Execute(target, new[] { "regs" }, null).Output);
            Assert.Equal(0x34000001UL, target.CpuRegisters.Current.Single(r => r.Name == "t0").Value);

            IDebugMemorySpace rdram = target.GetMemorySpaces().Single(s => s.Name == "RDRAM");
            rdram.Write(0x300, 0x5A);
            Assert.Equal(0x5A, target.GetMemorySpaces().Single(s => s.Name == "CPU").Read(unchecked((int)0x8000_0300)));
            Assert.Equal(8 << 20, rdram.Size);
            IDebugMemorySpace romSpace = target.GetMemorySpaces().Single(s => s.Name == "ROM");
            Assert.False(romSpace.IsWritable);
            Assert.Equal(0x80, romSpace.Read(0));

            Assert.Contains("5A", new MemCommand().Execute(target, new[] { "mem", "RDRAM", "300", "4" }, null).Output);
            // Since Mars_Native.md §6.5 the processor halts at a breakpoint and steps; the RSP still cannot, as on Mars.
            Assert.Contains("added", new BreakCommand().Execute(target, new[] { "bp", "add", "A400004C" }, null).Output);
            bundle.Core.RunFrame();
            Assert.True(bundle.Core.IsHaltedAtBreakpoint);
            Assert.Equal(unchecked((int)0xA400_004C), bundle.Core.HaltedAddress);
            target.Breakpoints.RemoveBreakpoint(target.Breakpoints.GetBreakpoints()[0].Id);
            Assert.Equal(0, new StepCommand().Execute(target, new[] { "step", "cpu" }, null).ExitCode);
            bundle.Core.RunFrame();
            Assert.Equal(unchecked((int)0xA400_0050), bundle.Core.HaltedAddress);
            Assert.Contains("cannot be halted", new StepCommand().Execute(target, new[] { "step", "rsp" }, null).Output);
            Assert.Contains("MarsRT", target.GetSummaryText());
        }

        // The synthetic system maps user space's first page pair onto physical 1MB and stores its field count there; Mars's own target is the oracle.
        [Fact]
        public void The_processor_s_view_follows_the_TLB_the_game_set_as_Mars_s_does()
        {
            string rom = Rom(SyntheticN64System.Build(rsp: false));
            MarsCore oracle = MarsRtTests.Oracle();
            using MarsRtCore twin = MarsRtTests.Twin();
            oracle.LoadRom(rom);
            twin.LoadRom(rom);
            for (int frame = 0; frame < 5; frame++)
            {
                oracle.RunFrame();
                twin.RunFrame();
            }

            IDebugMemorySpace want = new EmuSen.Cores.Nintendo.Mars.Debug.MarsDebugTarget(oracle).GetMemorySpaces().Single(s => s.Name == "CPU");
            IDebugMemorySpace got = new MarsRtDebugTarget(twin).GetMemorySpaces().Single(s => s.Name == "CPU");
            foreach (uint address in new uint[] { 0x0000_0000, 0x0000_0010, 0x0000_1FFC, 0x0000_2000, 0x8000_0400, 0xA400_0040, 0xBFC0_07C0 })
                for (int i = 0; i < 4; i++) Assert.Equal(want.Read(unchecked((int)address) + i), got.Read(unchecked((int)address) + i));

            Assert.NotEqual(0, twin.Peek(MarsRtSpace.Cpu, 0x13));
            Assert.Equal(twin.Peek(MarsRtSpace.Rdram, 0x10_0013), twin.Peek(MarsRtSpace.Cpu, 0x13));
        }

        private static byte[] State(MarsCore core)
        {
            using var stream = new MemoryStream();
            core.SaveState(stream);
            return stream.ToArray();
        }
    }

    // MarsRT's battery saves through the same files and switch as MarsCore's - see Mars_Save.md §7 and Mars_Native.md §5.5.
    [Collection(TestCollections.ProcessGlobals)]
    public class MarsRtSaveFileTests : IDisposable
    {
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenMarsRt_" + Guid.NewGuid().ToString("N"));

        // Block 5 of a 4 Kbit EEPROM, which the synthetic system never writes; it writes block 2.
        private const int Marked = 5 * 8, Written = 2 * 8;
        private static readonly byte[] Marker = "MARSRT!!"u8.ToArray();

        public MarsRtSaveFileTests()
        {
            Directory.CreateDirectory(_dir);
            DataStore.OverrideDirectory = Path.Combine(_dir, "Home");
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            if (Directory.Exists(_dir)) Directory.Delete(_dir, recursive: true);
        }

        private string WriteRom(string name)
        {
            string path = Path.Combine(_dir, name + ".z64");
            File.WriteAllBytes(path, SyntheticN64System.Build(rsp: false));
            return path;
        }

        private static byte[] MarkedSave()
        {
            byte[] save = Enumerable.Repeat((byte)0xFF, EmuSen.Cores.Nintendo.Mars.Memory.Eeprom.Size).ToArray();
            Marker.CopyTo(save, Marked);
            return save;
        }

        private static void Place(string rom, byte[] save)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(SaveLibrary.SramPathFor(rom))!);
            File.WriteAllBytes(SaveLibrary.SramPathFor(rom), save);
        }

        [Fact]
        public void A_save_is_read_at_load_and_what_the_game_wrote_is_written_back_for_either_engine_to_read()
        {
            string rom = WriteRom("Eeprom");
            Place(rom, MarkedSave());

            using var first = new MarsRtCore(batteryRamDisabled: false);
            first.LoadRom(rom);
            Assert.Equal(Marker, first.BatteryContents![Marked..(Marked + 8)]);
            for (int frame = 0; frame < 40; frame++) first.RunFrame();
            first.SaveSram();

            byte[] file = File.ReadAllBytes(SaveLibrary.SramPathFor(rom));
            Assert.Equal(EmuSen.Cores.Nintendo.Mars.Memory.Eeprom.Size, file.Length);
            Assert.Equal(Marker, file[Marked..(Marked + 8)]);
            Assert.NotEqual(Enumerable.Repeat((byte)0xFF, 8), file[Written..(Written + 8)]);

            using var second = new MarsRtCore(batteryRamDisabled: false);
            second.LoadRom(rom);
            Assert.Equal(file, second.BatteryContents);

            var mars = new MarsCore(batteryRamDisabled: false);
            mars.LoadRom(rom);
            Assert.Equal(file, mars.Bus!.Save.Contents);
        }

        [Fact]
        public void Nothing_is_written_when_the_game_changed_nothing()
        {
            string rom = Path.Combine(_dir, "Untouched.z64");
            File.WriteAllBytes(rom, SyntheticN64Rom.Build(patches: (0, new byte[] { 0x10, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00 })));

            using var core = new MarsRtCore(batteryRamDisabled: false);
            core.LoadRom(rom);
            core.RunFrame();
            core.SaveSram();

            Assert.False(File.Exists(SaveLibrary.SramPathFor(rom)));
            Assert.False(File.Exists(Path.ChangeExtension(SaveLibrary.SramPathFor(rom), MarsCore.PakExtension)));
        }

        [Fact]
        public void A_saved_chip_is_written_again_only_after_it_changes_again()
        {
            string rom = WriteRom("Twice");
            string save = SaveLibrary.SramPathFor(rom);

            using var core = new MarsRtCore(batteryRamDisabled: false);
            core.LoadRom(rom);
            for (int frame = 0; frame < 40; frame++) core.RunFrame();
            core.SaveSram();
            Assert.True(File.Exists(save));
            File.Delete(save);

            core.SaveSram();
            Assert.False(File.Exists(save));

            for (int frame = 0; frame < 40; frame++) core.RunFrame();
            core.SaveSram();
            Assert.True(File.Exists(save));
        }

        // Three hundred frames, as MarsCore's own autosave counts them - see Mars_Save.md §7.
        [Fact]
        public void A_changed_chip_is_written_on_the_three_hundredth_frame_without_being_asked()
        {
            string rom = WriteRom("Autosave");
            string save = SaveLibrary.SramPathFor(rom);

            using var core = new MarsRtCore(batteryRamDisabled: false);
            core.LoadRom(rom);
            for (int frame = 1; frame < MarsCore.SaveEveryNFrames; frame++) core.RunFrame();
            Assert.False(File.Exists(save));

            core.RunFrame();
            Assert.Equal(MarsCore.SaveEveryNFrames, core.TotalFrames);
            Assert.True(File.Exists(save));
        }

        [Fact]
        public void With_battery_ram_disabled_a_save_is_neither_read_nor_written()
        {
            string rom = WriteRom("NoBattery");
            byte[] original = MarkedSave();
            Place(rom, original);

            using var core = new MarsRtCore(batteryRamDisabled: true);
            core.LoadRom(rom);
            Assert.Equal(Enumerable.Repeat((byte)0xFF, 8), core.BatteryContents![Marked..(Marked + 8)]);
            for (int frame = 0; frame < 40; frame++) core.RunFrame();
            core.SaveSram();

            Assert.Equal(original, File.ReadAllBytes(SaveLibrary.SramPathFor(rom)));
        }

        // A scratch copy of Super Mario 64, the libretro database's own code for it, and the byte it names - see Mars_Native.md §5.5.
        [Fact]
        public void Super_Mario_64_s_unlimited_lives_hold_the_lives_at_one_hundred()
        {
            string? folder = Environment.GetEnvironmentVariable(MarsRtTests.StatesVariable);
            string cht = Environment.GetEnvironmentVariable("EMUSEN_MARSRT_SM64_CHT") ?? "";
            if (folder is null || !File.Exists(Path.Combine(folder, "sm64.z64")) || !File.Exists(cht)) return;

            string rom = Path.Combine(_dir, "sm64.z64");
            File.Copy(Path.Combine(folder, "sm64.z64"), rom);
            var cheats = new CheatRegistry();
            CoreBundle bundle = CoreFactory.Load(rom, cheats: cheats, engine: CoreCatalog.MarsRtEngine);
            var core = (MarsRtCore)bundle.Core;
            CheatImport.FromChtFile(cheats, cht, bundle.CheatAutoDetectCodec, replace: true);
            CheatInfo lives = cheats.GetCheats().Single(c => c.Description == "Unlimited Lives");
            cheats.SetEnabled(lives.Id, true);
            MarsCore oracle = MarsRtTests.Oracle(expansionPak: true);
            oracle.Cheats = cheats;
            oracle.LoadRom(rom);
            core.SkipRendering = true;
            foreach (EmuSen.Cores.ICore each in new EmuSen.Cores.ICore[] { core, oracle }) each.LoadState(Path.Combine(folder, "sm64.state"));
            Assert.NotEqual(0x64, core.Peek(MarsRtSpace.Rdram, 0x3094DD));

            for (int frame = 0; frame < 120; frame++)
            {
                core.RunFrame();
                oracle.RunFrame();
            }

            Assert.Equal(0x64, core.Peek(MarsRtSpace.Rdram, 0x3094DD));
            using var state = new MemoryStream();
            oracle.SaveState(state);
            Assert.True(state.ToArray().AsSpan().SequenceEqual(core.Save(false)), "with the cheat on, MarsRT and Mars left different states");
        }
    }
}
