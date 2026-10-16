using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Mistress.BigPicture;

namespace EmuSen.WiseMan.Mistress
{
    // ES-DE's per-game alternative emulator as the game's engine, with its badge, and ES-DE's cap on the play time one launch records - see EmuSen_BigPicture.md §29.
    [Collection(TestCollections.ProcessGlobals)]
    public class GameEngineAndPlayTimeTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GameEngineAndPlayTimeTests).GetTypeInfo().Assembly);

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        // A game's choices are its console's engines, the SNES's those discovery adds to Venus (C#); a stored engine its console lacks is not used.
        [Fact]
        public void A_game_s_engines_are_its_console_s_and_one_its_console_lacks_is_ignored()
        {
            MetadataField field = GameMetadata.Fields.Single(f => f.Key == GameMetadata.AltEmulator);
            Assert.Equal(new[] { "" }, GameMetadata.ChoicesFor(field, "/r/Game.z64").Select(c => c.Value));
            Assert.Equal(new[] { "" }, GameMetadata.ChoicesFor(field, "/r/Game.nes").Select(c => c.Value));
            Assert.Equal(new[] { "" }, GameMetadata.ChoicesFor(field, "/r/Game.gbc").Select(c => c.Value));
            Assert.Equal(new[] { "" }.Concat(CoreCatalog.EngineFor("SNES")?.Choices ?? Array.Empty<string>()), GameMetadata.ChoicesFor(field, "/r/Game.sfc").Select(c => c.Value));
            Assert.Null(GameMetadata.EngineFor("/r/Game.nes", CoreCatalog.MoonRtEngine));
            Assert.Null(GameMetadata.EngineFor("/r/Game.nes", "Moon (C#)"));
            Assert.Null(GameMetadata.EngineFor("/r/Game.z64", "Mars (C#)"));
            Assert.Null(GameMetadata.EngineFor("/r/Game.gb", "Mercury (C#)"));
            Assert.Null(GameMetadata.EngineFor("/r/Game.gb", CoreCatalog.MercuryRtEngine));
            Assert.Null(GameMetadata.EngineFor("/r/Game.sfc", CoreCatalog.MarsRtEngine));
            Assert.Null(GameMetadata.EngineFor("/r/Game.z64", ""));
        }

        // The editor offers a Game Boy or an NES game no engine, and a Super Nintendo game its engines when discovery found a second; the choice is an edit, and the badge shows it.
        [Fact]
        public Task The_editor_sets_a_game_s_engine_and_the_alternative_emulator_badge_shows_it() => Session.Dispatch(() =>
        {
            const string badges = "<badges name=\"badges\"><pos>0.55 0.8</pos><size>0.3 0.1</size><slots>altemulator</slots><lines>1</lines><itemsPerLine>4</itemsPerLine></badges>";
            using var s = new ThemedSession(extraGamelist: badges, status: ThemedSwitchesTests.Device);
            foreach (string console in new[] { "gb", "nes" })
            {
                ThemedLibraryPadTests.Enter(s, console);
                MetadataEditorWindow single = ThemedGameOptionsTests.OpenEditor(s);
                Assert.False(((Dropdown)single.EditorOf(GameMetadata.AltEmulator)).IsEnabled, $"{console} has one engine");
                single.Close();
                s.Settle();
                s.Pad.B();
            }
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Empty(s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<BadgeStrip>().Single().Entries!);
            MetadataEditorWindow snes = ThemedGameOptionsTests.OpenEditor(s);
            var engine = (Dropdown)snes.EditorOf(GameMetadata.AltEmulator);
            if (CoreCatalog.EngineFor("SNES") is not { } row)
            {
                Assert.False(engine.IsEnabled);
                return;
            }
            Assert.True(engine.IsEnabled);
            Assert.Equal(new[] { "None (the console's engine)" }.Concat(row.Choices!), ((System.Collections.IEnumerable)engine.ItemsSource!).Cast<string>());
            string chosen = row.Choices!.Last();
            engine.SelectedItem = chosen;
            ThemedGameOptionsTests.Reach(s, "MetadataSave");
            s.Pad.A();
            s.Settle();
            Assert.Equal(chosen, ThemedGameOptionsTests.StoredEdits(s, ThemedSession.SnesGames[0] + ".sfc")[GameMetadata.AltEmulator]);
            BadgeEntry badge = Assert.Single(s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<BadgeStrip>().Single().Entries!);
            Assert.Equal(BadgeKind.AltEmulator, badge.Kind);
        }, default);

        // A game or a console whose stored engine is the retired Mars (C#) runs on MarsRT, the N64's one engine - see Mars_Native.md §6.7.
        [Fact]
        public Task An_n64_game_that_names_the_retired_csharp_core_runs_on_MarsRT() => Session.Dispatch(() =>
        {
            string root = Path.Combine(Path.GetTempPath(), "EmuSenGameEngine", Guid.NewGuid().ToString("N"));
            string roms = Path.Combine(root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(root, "Config");
            DataStore.OverrideDirectory = Path.Combine(root, "Home");
            string? logsWere = ErrorLog.DirectoryOverride;
            ErrorLog.DirectoryOverride = Path.Combine(root, "Logs");
            try
            {
                string rom = Path.Combine(roms, "System.z64");
                File.WriteAllBytes(rom, SyntheticN64System.Build(rsp: true));
                new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever, StateDirectory = Path.Combine(root, "States") }.Save();
                GraphicsConfig config = GraphicsConfig.Load();
                config.SetValue("N64", CoreCatalog.EngineKey, "Mars (C#)");
                config.Save();
                using (GameRecords records = GameRecords.Open(GameRecords.DefaultPath))
                    records.SaveEdits(rom, new Dictionary<string, string?> { [GameMetadata.AltEmulator] = "Mars (C#)" }, DateTime.Now);

                string Engine()
                {
                    var window = new MainWindow();
                    window.Show();
                    typeof(MainWindow).GetMethod("LoadRom", Hidden)!.Invoke(window, [rom, "System.z64"]);
                    var session = (EmulatorSession)typeof(MainWindow).GetField("_session", Hidden)!.GetValue(window)!;
                    string engine = session.Engine!;
                    window.Close();
                    return engine;
                }

                Assert.Equal(CoreCatalog.MarsRtEngine, Engine());
                using (GameRecords records = GameRecords.Open(GameRecords.DefaultPath))
                    records.SaveEdits(rom, new Dictionary<string, string?> { [GameMetadata.AltEmulator] = null }, DateTime.Now);
                Assert.Equal(CoreCatalog.MarsRtEngine, Engine());
                using (GameRecords records = GameRecords.Open(GameRecords.DefaultPath))
                    records.SaveEdits(rom, new Dictionary<string, string?> { [GameMetadata.AltEmulator] = CoreCatalog.MercuryRtEngine }, DateTime.Now);
                Assert.Equal(CoreCatalog.MarsRtEngine, Engine());
            }
            finally
            {
                ConfigStore.OverrideDirectory = null;
                DataStore.OverrideDirectory = null;
                ErrorLog.DirectoryOverride = logsWere;
                try { Directory.Delete(root, true); } catch { }
            }
        }, default);

        // ES-DE's rule: 0 records nothing, 24 has no limit, and a launch longer than the limit records nothing at all.
        [Theory]
        [InlineData(8, 7.5, 7.5)]
        [InlineData(8, 8.5, null)]
        [InlineData(1, 0.25, 0.25)]
        [InlineData(0, 0.25, null)]
        [InlineData(24, 30.0, 30.0)]
        public void A_launch_longer_than_the_limit_adds_nothing(int limit, double hours, double? recorded) =>
            Assert.Equal(recorded is { } h ? TimeSpan.FromHours(h) : null, PlayTime.Tracked(TimeSpan.FromHours(hours), limit));

        // At the window: the default of 8 hours keeps a short launch and drops one left running all night; Preferences sets the limit.
        [Fact]
        public Task The_window_records_a_launch_within_the_limit_and_drops_one_past_it() => Session.Dispatch(() =>
        {
            using var s = new ThemedSession(status: ThemedSwitchesTests.Device);
            Assert.Equal(8, s.Settings.MaxPlayTimeTracking);
            string game = Path.Combine(s.RomDirectory, ThemedSession.SnesGames[0] + ".sfc");
            double Seconds()
            {
                using GameRecords records = GameRecords.Open(GameRecords.DefaultPath);
                return records.Find(game)?.PlaySeconds ?? 0;
            }
            void Launch(TimeSpan played)
            {
                typeof(MainWindow).GetField("_currentRomPath", Hidden)!.SetValue(s.Window, game);
                typeof(MainWindow).GetProperty("PlayClockReading", Hidden)!.SetValue(s.Window, (Func<TimeSpan>)(() => played));
                typeof(MainWindow).GetMethod("RecordStart", Hidden)!.Invoke(s.Window, [game]);
                typeof(MainWindow).GetMethod("RecordPlayTime", Hidden)!.Invoke(s.Window, null);
            }

            Launch(TimeSpan.FromMinutes(45));
            Assert.Equal(45 * 60, Seconds(), 3);
            Launch(TimeSpan.FromHours(9));
            Assert.Equal(45 * 60, Seconds(), 3);

            ThemedLibraryFlowTests.Choose(s, "Preferences");
            Window prefs = s.Window.GetControl<EmuSen.LunaP.Windowing.SheetLayer>("Sheets").Current!;
            ThemedSwitchesTests.Named<Dropdown>(prefs, "MaxPlayTimeDropdown").SelectedItem = "No limit";
            Assert.Equal(24, s.Settings.MaxPlayTimeTracking);
            Assert.Equal(24, AppSettings.Load().MaxPlayTimeTracking);
            s.Pad.B();
            Launch(TimeSpan.FromHours(9));
            Assert.Equal(45 * 60 + 9 * 3600, Seconds(), 3);
        }, default);
    }
}
