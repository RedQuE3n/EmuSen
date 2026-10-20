using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
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

        // The runtime's rows as the firmware page words them.
        private sealed record Worded(string Name, string Engine, IReadOnlyList<FirmwareRow> Items);

        private static Worded Words(FirmwareSystem s) => new(s.Name, s.Engine, s.Items.Select(i => new FirmwareRow(i.Label, i.FileName, i.Size, i.Present, i.WrongSize, i.Effect, i.Cost) { FoundIn = i.FoundIn, WrongSizeIn = i.WrongSizeIn }).ToArray());

        private static Worded Snes(ConsolesConfig? config = null) => Words(FirmwareOverview.Build(config ?? new ConsolesConfig()).Single(s => s.Name == "Super Nintendo"));

        private static void Put(string name, int bytes) => File.WriteAllBytes(Path.Combine(FirmwareLibrary.Directory, name), new byte[bytes]);

        [Fact]
        public void The_rows_are_what_the_engine_in_use_declares_and_every_shipped_system_is_listed()
        {
            var systems = FirmwareOverview.Build(new ConsolesConfig());
            Assert.Equal(["Nintendo Entertainment System", "Game Boy and Game Boy Color", "Super Nintendo", "Nintendo 64", "Sony PlayStation", "Sega Master System / Mark III", "Sega SG-1000", "Sega Genesis / Mega Drive", "Sega CD / Mega-CD"], systems.Select(s => s.Name));
            Assert.All(systems.Where(s => s.Name is not ("Super Nintendo" or "Sega Genesis / Mega Drive" or "Sony PlayStation" or "Sega CD / Mega-CD" or "Sega Master System / Mark III")), s => Assert.Empty(s.Items));
            Assert.Equal(2, systems.Single(s => s.Name == "Sega Master System / Mark III").Items.Count);

            Worded snes = Snes();
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
            FirmwareRow boot = Snes().Items.Single(i => i.FileName == "spc700.rom");
            Assert.Equal((false, FirmwareRow.OpenVersion, "Open version"), (boot.Present, boot.InUse, boot.InUseShort));
            Assert.Equal(FirmwareRow.FirstSentence(boot.Cost), boot.Change);
            Assert.Equal("Games may start a fraction of a second later.", boot.Change);

            Put("spc700.rom", 63);
            boot = Snes().Items.Single(i => i.FileName == "spc700.rom");
            Assert.Equal((false, true, FirmwareRow.OpenVersion), (boot.Present, boot.WrongSize, boot.InUse));
            Assert.Equal(("Games may start a fraction of a second later.", "Not used: the file in EmuSen's folder is not 64 bytes."), (boot.Change, boot.Note));

            Put("spc700.rom", 64);
            boot = Snes().Items.Single(i => i.FileName == "spc700.rom");
            Assert.Equal((true, false, FirmwareRow.OwnFile, "Your own file", FirmwareRow.AsTheConsole), (boot.Present, boot.WrongSize, boot.InUse, boot.InUseShort, boot.Change));
            Assert.Null(boot.Note);

            File.Delete(Path.Combine(FirmwareLibrary.Directory, "spc700.rom"));
            Assert.Equal(FirmwareRow.OpenVersion, Snes().Items.Single(i => i.FileName == "spc700.rom").InUse);
        }

        [Fact]
        public void A_chip_with_no_open_version_says_its_games_need_the_file_and_a_split_pair_counts_as_the_file()
        {
            FirmwareRow dsp3 = Snes().Items.Single(i => i.FileName == "dsp3.rom");
            Assert.Equal(("none", FirmwareRow.NoVersion, "None yet", FirmwareRow.NeedsFile), (dsp3.Effect, dsp3.InUse, dsp3.InUseShort, dsp3.Change));

            FirmwareRow dsp1 = Snes().Items.Single(i => i.FileName == "dsp1.rom");
            Assert.Equal(("accuracy", FirmwareRow.OpenVersion), (dsp1.Effect, dsp1.InUse));
            Assert.Equal(FirmwareRow.FirstSentence(dsp1.Cost), dsp1.Change);
            Put("dsp1.program.rom", 6144);
            Put("dsp1.data.rom", 2048);
            Assert.Equal(FirmwareRow.OwnFile, Snes().Items.Single(i => i.FileName == "dsp1.rom").InUse);
            Assert.Equal(FirmwareRow.OpenVersion, Snes().Items.Single(i => i.FileName == "dsp1b.rom").InUse);
        }

        [Fact]
        public void An_exact_open_version_says_so_and_a_cost_is_cut_at_its_first_sentence()
        {
            Assert.Equal(FirmwareRow.Exact, new FirmwareRow("A chip", "chip.rom", 8, false, false, "exact", null).Change);
            Assert.Equal(FirmwareRow.NeedsFile, new FirmwareRow("A chip", "chip.rom", 8, false, false, null, null).Change);
            Assert.Equal(FirmwareRow.NeedsFile, new FirmwareRow("A chip", "chip.rom", 8, false, false, "accuracy", null).Change);
            Assert.Equal("One thing differs.", FirmwareRow.FirstSentence("One thing differs. The rest is detail, e.g. at $FFC0."));
            Assert.Equal("No full stop", FirmwareRow.FirstSentence("No full stop"));
            Assert.Null(FirmwareRow.FirstSentence(" "));
        }

        // A stored choice of the retired Venus (C#) shows VenusRT's rows; with VenusRT's library gone the SNES lists nothing to use - see VenusRT_Native.md §68.5.
        [Fact]
        public void With_venus_stored_the_rows_are_venusrt_s_and_with_its_library_gone_there_are_none()
        {
            var config = new ConsolesConfig();
            config.SetValue("SNES", CoreCatalog.EngineKey, "Venus (C#)");
            Worded snes = Snes(config);
            Assert.Equal(CoreCatalog.VenusRtEngine, snes.Engine);
            Assert.Equal(Snes().Items.Select(i => i.FileName), snes.Items.Select(i => i.FileName));

            CoreDiscovery.UseDirectories([Path.Combine(_root, "NoCores")]);
            try
            {
                Assert.Empty(Snes().Items);
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
                var systems = FirmwareOverview.Build(new ConsolesConfig());
                Assert.DoesNotContain(systems, s => s.Engine == nephrite.EngineName || nephrite.Info.Systems.Any(n => n.Name == s.Name));
            }
            finally
            {
                CoreDiscovery.UseDirectories(null);
            }
        }

        // The Genesis (decided 2026-10-07) and the Sega CD (2026-10-16) are offered, so their rows are listed; the 32X, marked in development in the core's info, is not - see EmuSen_CoreAPI.md §27.4.
        [Fact]
        public void The_genesis_and_the_sega_cd_are_listed_and_the_32x_is_not()
        {
            var systems = FirmwareOverview.Build(new ConsolesConfig());
            FirmwareSystem genesis = systems.Single(s => s.Name == "Sega Genesis / Mega Drive");
            Assert.Equal(CoreDiscovery.Found.Single(c => c.Info.Id == "nephrite").EngineName, genesis.Engine);
            Assert.Equal(["bios_MD.bin"], genesis.Items.Select(i => i.FileName));
            Assert.Contains("bios_CD_U.bin", systems.Single(s => s.Name == "Sega CD / Mega-CD").Items.Select(i => i.FileName));
            Assert.DoesNotContain(systems, s => s.Name == "Sega 32X");
        }
    }
}
