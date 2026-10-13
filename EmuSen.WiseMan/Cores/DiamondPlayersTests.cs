using System.Diagnostics;
using System.IO.Compression;
using System.Text;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Media;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Psx;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Stage 11's runtime: a disc routed by its content, a sheet's tracks passed beside it, an archive unpacked into Galaxia's folder, an .m3u's discs changed through the lid, and a card kept as a battery file - see Diamond_Players.md.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondPlayersTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamondPlayers_" + Guid.NewGuid().ToString("N"));
        private readonly string _library;

        public DiamondPlayersTests(ITestOutputHelper output)
        {
            _output = output;
            _library = Path.Combine(_root, "Library");
            Directory.CreateDirectory(_library);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
            CoreOptions.BatteryRamDisabled = false;
        }

        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Write(string name, byte[] bytes, string? folder = null)
        {
            string dir = folder is null ? _library : Path.Combine(_library, folder);
            Directory.CreateDirectory(dir);
            string path = Path.Combine(dir, name);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        // What the library holds, by path, size and time, so a test can show nothing in it was written.
        private string[] Listing() => Directory.EnumerateFiles(_library, "*", SearchOption.AllDirectories).Order().Select(f => $"{f} {new FileInfo(f).Length} {File.GetLastWriteTimeUtc(f).Ticks}").ToArray();

        private static string Zip(string path, params (string Name, byte[] Data)[] entries)
        {
            using var zip = ZipFile.Open(path, ZipArchiveMode.Create);
            foreach (var (name, data) in entries)
                using (var s = zip.CreateEntry(name, CompressionLevel.Fastest).Open()) s.Write(data);
            return path;
        }

        [Fact]
        public void A_bin_goes_to_the_console_its_content_is_and_an_iso_and_a_cue_to_the_playstation()
        {
            var disc = SyntheticPsxDisc.Build("Europe");
            string bin = Write("Ruby Lantern (Europe).bin", disc.DataTrack), md = Write("Granite Choir.bin", SyntheticMdRom.Cartridge());
            Assert.Equal(("PlayStation", "Genesis"), (CoreCatalog.ConsoleForRom(bin), CoreCatalog.ConsoleForRom(md)));
            Assert.Equal("PlayStation", CoreCatalog.ConsoleForRom(Write("Ruby Lantern.iso", disc.Iso)));
            // Where development cores are shown, the Sega CD, in development, takes .iso too, and each disc goes to its own console.
            CoreDiscovery.UseDevelopment(true);
            string segacd = Write("Sapphire Coast.iso", SyntheticMdRom.Disc());
            Assert.Equal(("Sega CD", "PlayStation"), (CoreCatalog.Route(segacd).Routed?.System?.Console, CoreCatalog.ConsoleForRom(Write("Ruby Lantern 2.iso", disc.Iso))));
            using (var cd = (CoreEngine)CoreFactory.Load(segacd).Core) Assert.Equal(("nephrite", "mcd"), (cd.Info.Id, cd.Machine.Info.System));
            CoreDiscovery.UseDevelopment(false);
            Assert.Equal("PlayStation", CoreCatalog.ConsoleForRom(disc.WriteSet(_library, "Ruby Set")));
            var routed = CoreCatalog.Route(bin).Routed!;
            Assert.Equal(new Recognition(PsxSystem.Id, "EMUS-00001", "Europe"), routed.Recognition);
            Assert.True(CoreFactory.IsSupported(bin) && CoreFactory.IsSupported(md));
            using (var core = (CoreEngine)CoreFactory.Load(bin).Core) Assert.Equal(("diamond", "pal"), (core.Info.Id, core.Machine.Info.Region));
            var genesis = CoreFactory.Load(md).Core;
            Assert.Equal("Genesis", CoreCatalog.ConsoleForRom(md));
            (genesis as IDisposable)?.Dispose();

            // Neither's content: no console claims a .bin of noise, so it opens nowhere and the words say why; its folder's name decides.
            string noise = Write("Static.bin", Enumerable.Repeat(new byte[] { 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00 }, 2000).SelectMany(b => b).ToArray());
            var none = CoreCatalog.Route(noise);
            Assert.Null(none.Core);
            _output.WriteLine(none.Routed!.Reason);
            Assert.Contains("could be Sega Genesis / Mega Drive or Sony PlayStation", none.Routed.Reason);
            Assert.True(CoreFactory.IsSupported(noise), "its extension is opened, and the load says why it cannot be");
            Assert.Contains("could be", Assert.Throws<NotSupportedException>(() => CoreFactory.Load(noise)).Message);
            Assert.Equal("PlayStation", CoreCatalog.ConsoleForRom(Write("Static.bin", File.ReadAllBytes(noise), folder: "psx")));
        }

        [Fact]
        public void A_sheet_opens_with_its_tracks_and_the_shelf_lists_the_sheet_alone()
        {
            var disc = SyntheticPsxDisc.Build();
            string cue = disc.WriteSet(_library, "Amber Set");
            Assert.Equal(new[] { "Amber Set (Track 1).bin", "Amber Set (Track 2).bin" }, ImageSets.FilesFor(cue).Select(Path.GetFileName));
            using (var core = (CoreEngine)CoreFactory.Load(cue).Core)
            {
                core.RunFrame();
                Assert.Equal(("psx", 1), (core.Machine.Info.System, core.Discs.Count));
            }
            var shelved = ImageSets.Shelved(Directory.EnumerateFiles(_library).ToList());
            Assert.Equal(new[] { "Amber Set.cue" }, shelved.Select(Path.GetFileName));
            File.Delete(Path.Combine(_library, "Amber Set (Track 2).bin"));
            var e = Assert.ThrowsAny<IOException>(() => CoreFactory.Load(cue));
            _output.WriteLine(e.Message);
        }

        [Fact]
        public void A_zipped_sheet_and_a_zipped_track_unpack_beside_galaxia_never_beside_the_archive()
        {
            var disc = SyntheticPsxDisc.Build("Japan");
            string sheet = disc.Cue.Replace("{name}", "Inner");
            string zipped = Zip(Path.Combine(_library, "Coral Gate (Japan).zip"), ("Inner.cue", Encoding.ASCII.GetBytes(sheet)), ("Inner (Track 1).bin", disc.DataTrack), ("Inner (Track 2).bin", disc.AudioTrack), ("readme.txt", "x"u8.ToArray()));
            string track = Zip(Path.Combine(_library, "Coral Track.zip"), ("Coral Track (Track 1).bin", SyntheticPsxDisc.Build().DataTrack));
            string[] before = Listing();
            Assert.Equal((".cue", "PlayStation", ".bin", "PlayStation"), (CoreCatalog.ImageExtension(zipped), CoreCatalog.ConsoleForRom(zipped), CoreCatalog.ImageExtension(track), CoreCatalog.ConsoleForRom(track)));
            Assert.True(CoreFactory.IsSupported(zipped));
            using (var core = (CoreEngine)CoreFactory.Load(zipped).Core)
            {
                core.RunFrame();
                Assert.Equal(("psx", "ntsc", zipped), (core.Machine.Info.System, core.Machine.Info.Region, core.RomPath));
                Assert.StartsWith(DataStore.Unpacked, core.Discs[0]);
                Assert.Equal("Coral Gate (Japan).cue", Path.GetFileName(core.Discs[0]));
                Assert.Equal(SaveLibrary.SramPathFor(zipped, "PlayStation").Replace(".srm", ".mcr"), Path.Combine(DataStore.Saves, "PlayStation", "Coral Gate (Japan).mcr"));
            }
            using (var core = (CoreEngine)CoreFactory.Load(track).Core) Assert.Equal("Coral Track.bin", Path.GetFileName(core.Discs[0]));
            Assert.Equal(before, Listing());
            Assert.Equal(2, Directory.GetDirectories(DataStore.Unpacked).Length);

            // The folder keeps the archives opened last within its size, the one opened now always, and removes a partial unpacking a stopped process left.
            string old = Directory.GetDirectories(DataStore.Unpacked).Single(d => d == ArchiveImages.FolderFor(zipped));
            Directory.SetLastWriteTimeUtc(old, DateTime.UtcNow.AddDays(-2));
            string partial = Directory.CreateDirectory(Path.Combine(DataStore.Unpacked, "abcd.partial-1")).FullName;
            Directory.SetLastWriteTimeUtc(partial, DateTime.UtcNow.AddHours(-2));
            ArchiveImages.Prune(keep: ArchiveImages.FolderFor(track), keptBytes: new FileInfo(ArchiveImages.Unpack(track, CoreCatalog.IsImageExtension)).Length + 1024);
            Assert.Equal(new[] { ArchiveImages.FolderFor(track) }, Directory.GetDirectories(DataStore.Unpacked));
        }

        // The firmware a game is asked for before create is its image's, an archive's unpacked image as for play, never the archive's bytes - see Diamond_Players.md §12.
        [Fact]
        public void A_zipped_disc_is_asked_for_the_firmware_its_image_is()
        {
            var disc = SyntheticPsxDisc.Build();
            string loose = Write("Opal Shore.bin", disc.DataTrack);
            string zipped = Zip(Path.Combine(_library, "Opal Shore Zipped.zip"), ("Opal Shore Zipped/Opal Shore Zipped.bin", disc.DataTrack));
            string[] Names(string path) => CoreFactory.ForFirmwareProbe(path).GetFirmwareRequirements(path).Select(r => r.FileName).ToArray();
            Assert.Equal(new[] { "scph5501.bin" }, Names(loose));
            Assert.Equal(Names(loose), Names(zipped));
        }

        // The question asked before every launch reads no image where the core's info leaves no firmware required: a zip is not unpacked for it, a track gone from its folder not missed - see Diamond_Players.md §12.8.2.
        [Fact]
        public void The_firmware_question_reads_no_image_where_no_entry_can_be_required()
        {
            var disc = SyntheticPsxDisc.Build();
            string zipped = Zip(Path.Combine(_library, "Opal Vault.zip"), ("Opal Vault/Opal Vault.bin", disc.DataTrack));
            Assert.Empty(EmuSen.Common.EmulatorSession.MissingFirmwareFor(zipped));
            Assert.False(Directory.Exists(ArchiveImages.FolderFor(zipped)), "the zip was unpacked for the question");

            string loose = Write("Opal Lock.bin", disc.DataTrack);
            var engine = (CoreEngine)CoreFactory.ForFirmwareProbe(loose);
            File.Move(loose, loose + ".away");
            Assert.ThrowsAny<IOException>(() => engine.GetFirmwareRequirements(loose));
            Assert.Empty(engine.RequiredFirmwareFor(loose));
            File.Move(loose + ".away", loose);
            Assert.Equal(engine.GetFirmwareRequirements(loose).Where(r => r.Required), engine.RequiredFirmwareFor(loose));
            Assert.Equal("scph5501.bin", engine.GetFirmwareRequirements(loose).Single().FileName);
        }

        // A required entry, or a system listing no firmware, sends the question to the image as before; Diamond's and the test core's lists of optional files do not.
        [Fact]
        public void Only_info_listing_every_system_s_firmware_as_optional_answers_without_the_image()
        {
            var diamond = (CoreEngine)CoreFactory.ForFirmwareProbe(Write("Opal Info.bin", SyntheticPsxDisc.Build().DataTrack));
            var test = CoreAdapterTests.Full.Info;
            Assert.False(CoreEngine.CanRequireFirmware(diamond.Info));
            Assert.False(CoreEngine.CanRequireFirmware(test));
            Assert.True(CoreEngine.CanRequireFirmware(test with { Systems = test.Systems.Select(s => s with { Firmware = s.Firmware.Select(f => f with { Required = true }).ToArray() }).ToArray() }));
            Assert.True(CoreEngine.CanRequireFirmware(test with { Systems = test.Systems.Select(s => s with { Firmware = Array.Empty<CoreFirmware>() }).ToArray() }));
            Assert.True(CoreEngine.CanRequireFirmware(test with { Systems = Array.Empty<CoreSystem>() }));
        }

        // A sheet written where names ignore case, or with a Windows separator, names its tracks as the folder or archive holds them - see Diamond_Players.md §12.
        [Fact]
        public void A_sheet_naming_its_tracks_in_another_case_or_with_backslashes_opens_and_is_shelved_once()
        {
            var disc = SyntheticPsxDisc.Build();
            string folder = Path.Combine(_library, "Quartz Bay");
            Directory.CreateDirectory(Path.Combine(folder, "Tracks"));
            File.WriteAllText(Path.Combine(folder, "Quartz Bay.cue"), disc.Cue.Replace("{name}", "TRACKS\\quartz bay"));
            File.WriteAllBytes(Path.Combine(folder, "Tracks", "Quartz Bay (Track 1).bin"), disc.DataTrack);
            File.WriteAllBytes(Path.Combine(folder, "Tracks", "Quartz Bay (Track 2).bin"), disc.AudioTrack);
            string zipped = Zip(Path.Combine(_library, "Quartz Zip.zip"), ("Quartz Zip/Quartz Zip.cue", Encoding.ASCII.GetBytes(disc.Cue.Replace("{name}", "QUARTZ ZIP"))),
                ("Quartz Zip/Quartz Zip (Track 1).bin", disc.DataTrack), ("Quartz Zip/Quartz Zip (Track 2).bin", disc.AudioTrack));
            Assert.Equal(new[] { "Quartz Bay.cue", "Quartz Zip.zip" }, ImageSets.Shelved(Directory.EnumerateFiles(_library, "*", SearchOption.AllDirectories).Order().ToList()).Select(Path.GetFileName));
            foreach (string game in new[] { Path.Combine(folder, "Quartz Bay.cue"), zipped })
            {
                Assert.Equal(new[] { "Quartz Bay (Track 1).bin", "Quartz Bay (Track 2).bin" }.Select(n => n.Replace("Quartz Bay", Path.GetFileNameWithoutExtension(game))), ImageSets.FilesFor(CoreEngine.DiscsOf(game)[0]).Select(Path.GetFileName));
                using var core = (CoreEngine)CoreFactory.Load(game).Core;
                core.RunFrame();
                Assert.Equal(("psx", true), (core.Machine.Info.System, core.TotalFrames > 0));
            }
        }

        // Two discs of one game in an .m3u: the shelf shows the list, the first disc goes in, Change Disc opens the lid, swaps and closes it, and a state made on either disc puts that disc back to load.
        [Fact]
        public void An_m3u_changes_its_discs_through_the_lid_and_a_state_brings_its_own_disc_back()
        {
            var one = SyntheticPsxDisc.Build(exe: SyntheticPsExe.Build(SyntheticPsExe.Words(0x2408_0001, 0x0800_4000, 0)));
            var two = SyntheticPsxDisc.Build(exe: SyntheticPsExe.Build(SyntheticPsExe.Words(0x2408_0002, 0x0800_4000, 0)));
            one.WriteSet(_library, "Twin Peaks (Disc 1)");
            two.WriteSet(_library, "Twin Peaks (Disc 2)");
            string m3u = Path.Combine(_library, "Twin Peaks.m3u");
            File.WriteAllText(m3u, "Twin Peaks (Disc 1).cue\nTwin Peaks (Disc 2).cue\n");
            Assert.Equal(new[] { "Twin Peaks.m3u" }, ImageSets.Shelved(Directory.EnumerateFiles(_library).ToList()).Select(Path.GetFileName));
            Assert.Equal("PlayStation", CoreCatalog.ConsoleForRom(m3u));

            using var core = (CoreEngine)CoreFactory.Load(m3u).Core;
            Assert.True(core.CanChangeDisc);
            Assert.Equal((2, 0, false), (core.Discs.Count, core.DiscIndex, core.LidOpen));
            for (int f = 0; f < 10; f++) core.RunFrame();
            using var first = new MemoryStream();
            core.SaveState(first);

            core.ChangeDisc(1);
            Assert.Equal((1, true), (core.DiscIndex, core.LidOpen));
            for (int f = 0; f < 70; f++) core.RunFrame();
            Assert.False(core.LidOpen, "the lid closes a second of the game's time after the disc goes in");
            using var second = new MemoryStream();
            core.SaveState(second);

            first.Position = 0;
            core.LoadState(first);
            Assert.Equal((0, false), (core.DiscIndex, core.LidOpen));
            second.Position = 0;
            core.LoadState(second);
            Assert.Equal(1, core.DiscIndex);
            Assert.ThrowsAny<InvalidOperationException>(() => core.InsertDisc(0));
        }

        private static uint I(uint op, int rs, int rt, int imm) => op << 26 | (uint)rs << 21 | (uint)rt << 16 | (ushort)imm;

        // A program that writes one 128-byte frame to the card in slot 1 over SIO0 with psx-spx's write command, each byte sent once RXRDY showed the last, the replies kept at 80100000h.
        private static byte[] CardWriting(int frame, byte[] data)
        {
            const int t0 = 8, t1 = 9, t2 = 10, s0 = 16, s1 = 17;
            byte msb = (byte)(frame >> 8), lsb = (byte)frame;
            var bytes = new List<int> { 0x81, 0x57, 0, 0, msb, lsb };
            bytes.AddRange(data.Select(b => (int)b));
            bytes.Add(data.Aggregate((byte)(msb ^ lsb), (c, b) => (byte)(c ^ b)));
            bytes.AddRange(new[] { 0, 0, 0 });
            var w = new List<uint> { I(0x0F, 0, s0, 0x1F80), I(0x0F, 0, s1, 0x8010), I(0x0D, 0, t0, 0x88), I(0x29, s0, t0, 0x104E), I(0x0D, 0, t0, 0x0D), I(0x29, s0, t0, 0x1048) };
            w.AddRange(new[] { I(0x0D, 0, t0, 0x1003), I(0x29, s0, t0, 0x104A) });
            foreach (var (b, i) in bytes.Select((b, i) => (b, i)))
            {
                w.AddRange(new[] { I(0x0D, 0, t1, b), I(0x28, s0, t1, 0x1040) });
                int wait = w.Count;
                w.AddRange(new[] { I(0x23, s0, t2, 0x1044), 0u, I(0x0C, t2, t2, 2) });
                w.Add(I(0x04, t2, 0, wait - (w.Count + 1)));
                w.AddRange(new[] { 0u, I(0x24, s0, t2, 0x1040), 0u, I(0x28, s1, t2, i) });
            }
            w.Add(I(0x29, s0, 0, 0x104A));
            uint loop = SyntheticPsExe.Origin + 4 * (uint)w.Count;
            w.AddRange(new[] { 0x0800_0000u | ((loop >> 2) & 0x03FF_FFFF), 0u });
            return SyntheticPsExe.Build(SyntheticPsExe.Words(w.ToArray()));
        }

        // The card a program writes is saved in the raw 128 KiB form under Saves/PlayStation and read back at the next load - see Diamond_Plan.md §4.4.
        [Fact]
        public void A_card_a_program_writes_round_trips_through_its_battery_file()
        {
            var data = Enumerable.Range(0, 128).Select(i => (byte)(i * 3 + 1)).ToArray();
            string exe = Write("Card Writer.psexe", CardWriting(0x40, data));
            string card = Path.Combine(DataStore.Saves, "PlayStation", "Card Writer.mcr");
            using (var core = (CoreEngine)CoreFactory.Load(exe).Core)
            {
                Assert.Equal(new[] { (0u, ".mcr") }, core.Machine.Info.Battery.Select(b => (b.Which, b.Suffix)));
                for (int f = 0; f < 30; f++) core.RunFrame();
                byte[] ram = new byte[0x100];
                for (int i = 0; i < 0x90; i++) ram[i] = core.ReadSpace("RAM", 0x10_0000 + i);
                _output.WriteLine("replies: " + Convert.ToHexString(ram[..0x90]));
                Assert.Equal(new byte[] { 0x5C, 0x5D, 0x47 }, ram[0x87..0x8A]);
                core.SaveSram();
            }
            byte[] saved = File.ReadAllBytes(card);
            Assert.Equal(131_072, saved.Length);
            Assert.Equal(data, saved[(0x40 * 128)..(0x41 * 128)]);
            saved[0x2000] ^= 0xFF;
            File.WriteAllBytes(card, saved);
            using (var again = (CoreEngine)CoreFactory.Load(exe).Core)
                Assert.Equal(saved, again.Machine.Battery(0).Data);
        }
    }
}
