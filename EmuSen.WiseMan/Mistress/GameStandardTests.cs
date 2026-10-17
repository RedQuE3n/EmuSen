using System;
using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // The CRT's television standard follows the running game's, the player's choice for the game over the console's - see EmuSen_CRT.md §26.
    [Collection(TestCollections.ProcessGlobals)]
    public class GameStandardTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(GameStandardTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenGameStandard", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public GameStandardTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            new AppSettings { RomDirectory = roms, LogDirectory = Path.Combine(_root, "Logs"), StateDirectory = Path.Combine(_root, "States"), ResumeOnLaunch = AppSettings.ResumeNever }.Save();
            _rom = Path.Combine(roms, "Game.smc");
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // A blank cartridge whose header names a country: 0 the Japanese, 2 a European, which VenusRT runs as PAL.
        private void Cartridge(byte country) => File.WriteAllBytes(_rom, SyntheticRom.Build((0x7FD9, new[] { country })));

        // The CRT chosen for the SNES, and any value stored for its standard.
        private static void Configure(float? standard)
        {
            var graphics = new GraphicsConfig();
            graphics.SetValue("SNES", GraphicsSettingsWindow.ScreenFilterKey, CrtFilter.Name);
            if (standard is { } value) graphics.SetParameter("SNES", CrtFilter.Name, TelevisionStandards.Parameter, value.ToString(System.Globalization.CultureInfo.InvariantCulture));
            graphics.Save();
        }

        private void GameChoice(string? standard)
        {
            using GameRecords records = GameRecords.Open(GameRecords.DefaultPath);
            records.SetStandardChoice(_rom, standard, DateTime.Now);
        }

        // The standard the frame control was given for the game's first frame.
        private static float Drawn() => Launched(frame => frame.ShaderParameters![TelevisionStandards.Parameter]);

        private static T Launched<T>(Func<GameFrameControl, T> read)
        {
            var window = new MainWindow();
            window.Show();
            try
            {
                window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
                typeof(MainWindow).GetMethod("LaunchSelectedLibraryEntry", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
                return read(window.GetControl<GameFrameControl>("GameFrame"));
            }
            finally
            {
                window.Close();
            }
        }

        [Fact]
        public Task With_nothing_chosen_the_picture_is_the_game_s_standard() => Session.Dispatch(() =>
        {
            Configure(null);
            Cartridge(0x00);
            Assert.Equal(0f, Drawn());
            Cartridge(0x02);
            Assert.Equal(1f, Drawn());
        }, default);

        [Fact]
        public Task A_console_s_stored_standard_is_kept_over_the_game_s() => Session.Dispatch(() =>
        {
            Configure(0f);
            Cartridge(0x02);
            Assert.Equal(0f, Drawn());
            Configure(TelevisionStandards.FollowsGame);
            Assert.Equal(1f, Drawn());
        }, default);

        [Fact]
        public Task The_game_s_own_choice_wins_from_its_first_frame() => Session.Dispatch(() =>
        {
            Configure(1f);
            Cartridge(0x02);
            GameChoice(TelevisionStandards.Ntsc);
            Assert.Equal(0f, Drawn());
            GameChoice(null);
            Assert.Equal(1f, Drawn());
        }, default);

        // The same order of a load draws a game's own crop from its first frame, which the console's crop stood in for until the game's options were opened.
        [Fact]
        public Task A_game_s_own_crop_is_drawn_from_its_first_frame() => Session.Dispatch(() =>
        {
            Configure(null);
            Cartridge(0x00);
            using (GameRecords records = GameRecords.Open(GameRecords.DefaultPath)) records.SetCrop(_rom, new GameCrop(1, 2, 3, 4));
            Assert.Equal(new GameCrop(1, 2, 3, 4).Picture, Launched(frame => frame.Crop));
        }, default);
    }
}
