using System;
using System.IO;
using System.Linq;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // The firmware page's rows: read from each engine's own declarations, worded by what is in the folder, and silent about a core in development - see EmuSen_Settings_Reference.md §4.89.
    [Collection(TestCollections.ProcessGlobals)]
    public class FirmwareOverviewTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenFirmwareOverview_" + Guid.NewGuid().ToString("N"));

        public FirmwareOverviewTests()
        {
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            FirmwareLibrary.ResetDirectory();
            Directory.CreateDirectory(FirmwareLibrary.Directory);
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static DiscoveredCore VenusRt => CoreDiscovery.Found.Single(c => c.Info.Id == "venusrt");

        private static FirmwareSystem Snes(GraphicsConfig? config = null) => FirmwareOverview.Build(config ?? new GraphicsConfig()).Single(s => s.Name == "Super Nintendo");

        private static void Put(string name, int bytes) => File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, name), new byte[bytes]);

        [Fact]
        public void The_rows_are_what_the_engine_in_use_declares_and_every_shipped_system_is_listed()
        {
            var systems = FirmwareOverview.Build(new GraphicsConfig());
            Assert.Equal(["Nintendo Entertainment System", "Game Boy and Game Boy Color", "Super Nintendo", "Nintendo 64", "Sega Genesis / Mega Drive"], systems.Select(s => s.Name));
            Assert.All(systems.Where(s => s.Name is not ("Super Nintendo" or "Sega Genesis / Mega Drive")), s => Assert.Empty(s.Items));

            FirmwareSystem snes = Snes();
            CoreSystem declared = VenusRt.Info.Systems.Single(s => s.Id == "snes");
            Assert.Equal(CoreCatalog.VenusRtEngine, snes.Engine);
            Assert.Equal(declared.Firmware.Select(f => (f.Label, f.Name, (int)f.Size, f.Replacement?.Effect, f.Replacement?.Cost)), snes.Items.Select(i => (i.Label, i.FileName, i.Size, i.Effect, i.Cost)));
            Assert.Equal(["spc700.rom", "dsp1.rom", "dsp1b.rom", "dsp2.rom", "dsp3.rom", "dsp4.rom", "st010.rom", "st011.rom"], snes.Items.Select(i => i.FileName));
            Assert.All(snes.Items, i => Assert.Equal($"{i.Label} ({i.FileName})", i.Title));
            Assert.Equal("SNES sound chip start-up (spc700.rom)", snes.Items[0].Title);
        }

        [Fact]
        public void A_file_in_the_folder_changes_the_words_and_one_of_the_wrong_size_is_said_to_be_unused()
        {
            FirmwareItem boot = Snes().Items.Single(i => i.FileName == "spc700.rom");
            Assert.Equal((false, FirmwareItem.OpenVersion, "Open version"), (boot.Present, boot.InUse, boot.InUseShort));
            Assert.Equal(FirmwareItem.FirstSentence(boot.Cost), boot.Change);
            Assert.Equal("Games may start a fraction of a second later.", boot.Change);

            Put("spc700.rom", 63);
            boot = Snes().Items.Single(i => i.FileName == "spc700.rom");
            Assert.Equal((false, true, FirmwareItem.OpenVersion), (boot.Present, boot.WrongSize, boot.InUse));
            Assert.Equal(("Games may start a fraction of a second later.", "The file here is not 64 bytes, so it is not used."), (boot.Change, boot.Note));

            Put("spc700.rom", 64);
            boot = Snes().Items.Single(i => i.FileName == "spc700.rom");
            Assert.Equal((true, false, FirmwareItem.OwnFile, "Your own file", FirmwareItem.AsTheConsole), (boot.Present, boot.WrongSize, boot.InUse, boot.InUseShort, boot.Change));
            Assert.Null(boot.Note);

            File.Delete(Path.Combine(FirmwareLibrary.Directory, "spc700.rom"));
            Assert.Equal(FirmwareItem.OpenVersion, Snes().Items.Single(i => i.FileName == "spc700.rom").InUse);
        }

        [Fact]
        public void A_chip_with_no_open_version_says_its_games_need_the_file_and_a_split_pair_counts_as_the_file()
        {
            FirmwareItem dsp3 = Snes().Items.Single(i => i.FileName == "dsp3.rom");
            Assert.Equal(("none", FirmwareItem.NoVersion, "None yet", FirmwareItem.NeedsFile), (dsp3.Effect, dsp3.InUse, dsp3.InUseShort, dsp3.Change));

            FirmwareItem dsp1 = Snes().Items.Single(i => i.FileName == "dsp1.rom");
            Assert.Equal(("accuracy", FirmwareItem.OpenVersion), (dsp1.Effect, dsp1.InUse));
            Assert.Equal(FirmwareItem.FirstSentence(dsp1.Cost), dsp1.Change);
            Put("dsp1.program.rom", 6144);
            Put("dsp1.data.rom", 2048);
            Assert.Equal(FirmwareItem.OwnFile, Snes().Items.Single(i => i.FileName == "dsp1.rom").InUse);
            Assert.Equal(FirmwareItem.OpenVersion, Snes().Items.Single(i => i.FileName == "dsp1b.rom").InUse);
        }

        [Fact]
        public void An_exact_open_version_says_so_and_a_cost_is_cut_at_its_first_sentence()
        {
            Assert.Equal(FirmwareItem.Exact, new FirmwareItem("A chip", "chip.rom", 8, false, false, "exact", null).Change);
            Assert.Equal(FirmwareItem.NeedsFile, new FirmwareItem("A chip", "chip.rom", 8, false, false, null, null).Change);
            Assert.Equal(FirmwareItem.NeedsFile, new FirmwareItem("A chip", "chip.rom", 8, false, false, "accuracy", null).Change);
            Assert.Equal("One thing differs.", FirmwareItem.FirstSentence("One thing differs. The rest is detail, e.g. at $FFC0."));
            Assert.Equal("No full stop", FirmwareItem.FirstSentence("No full stop"));
            Assert.Null(FirmwareItem.FirstSentence(" "));
        }

        [Fact]
        public void With_venus_chosen_or_venusrt_s_library_gone_the_rows_are_the_reference_core_s_own_requests()
        {
            var config = new GraphicsConfig();
            config.SetValue("SNES", CoreCatalog.EngineKey, CoreCatalog.VenusEngine);
            FirmwareSystem snes = Snes(config);
            Assert.Equal(CoreCatalog.VenusEngine, snes.Engine);
            Assert.Equal(Enum.GetValues<NecDspVariant>().Select(v => NecDspFirmware.RequestFor(v).FileName).Distinct(), snes.Items.Select(i => i.FileName));
            Assert.All(snes.Items, i => Assert.Equal((FirmwareItem.NoVersion, FirmwareItem.NeedsFile), (i.InUse, i.Change)));

            CoreDiscovery.UseDirectories([Path.Combine(_root, "NoCores")]);
            try
            {
                Assert.Equal(CoreCatalog.VenusEngine, Snes().Engine);
            }
            finally
            {
                CoreDiscovery.UseDirectories(null);
            }
        }

        // A core in development, here Nephrite's library with its sidecar written as it was before 2026-10-07, is left out even when discovery lists it.
        [Fact]
        public void A_core_in_development_is_left_out_even_when_discovery_lists_it()
        {
            string built = Path.Combine(_root, "development");
            Directory.CreateDirectory(built);
            string copy = Path.Combine(built, Path.GetFileName(NephriteTests.LibraryPath));
            File.Copy(NephriteTests.LibraryPath, copy);
            Assert.True(CoreSidecar.Write(copy, development: true).Development);
            CoreDiscovery.UseDirectories(new[] { built });
            try
            {
                CoreDiscovery.UseDevelopment(true);
                DiscoveredCore nephrite = CoreDiscovery.Found.Single(c => c.Info.Id == "nephrite");
                Assert.True(nephrite.Sidecar.Development);
                Assert.Contains(nephrite.Info.Systems, s => s.Firmware.Count > 0);
                var systems = FirmwareOverview.Build(new GraphicsConfig());
                Assert.DoesNotContain(systems, s => s.Engine == nephrite.EngineName || nephrite.Info.Systems.Any(n => n.Name == s.Name));
            }
            finally
            {
                CoreDiscovery.UseDirectories(null);
            }
        }

        // Decided 2026-10-07: the Genesis is offered, so its row is listed; the Sega CD and the 32X, marked in development in the core's info, are not - see EmuSen_CoreAPI.md §27.4.
        [Fact]
        public void The_genesis_is_listed_and_its_attachments_are_not()
        {
            var systems = FirmwareOverview.Build(new GraphicsConfig());
            FirmwareSystem genesis = systems.Single(s => s.Name == "Sega Genesis / Mega Drive");
            Assert.Equal(CoreDiscovery.Found.Single(c => c.Info.Id == "nephrite").EngineName, genesis.Engine);
            Assert.Equal(["bios_MD.bin"], genesis.Items.Select(i => i.FileName));
            Assert.DoesNotContain(systems, s => s.Name is "Sega CD / Mega-CD" or "Sega 32X");
        }
    }
}
