using System;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Input
{
    // Each player's pad bindings per console, player 1's until changed, beside the keys a file already had - see EmuSen_Input.md §8.4.
    [Collection(TestCollections.ProcessGlobals)]
    public class PlayerBindingsTests : IDisposable
    {
        private static readonly string[] Consoles = { "NES", "SNES" };
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenPlayerBindings_" + Guid.NewGuid().ToString("N"));

        public PlayerBindingsTests() => ConfigStore.OverrideDirectory = _dir;

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private string File => Path.Combine(_dir, "gamepadbindings.json");

        [Fact]
        public void A_player_never_changed_plays_with_player_1s_map_and_follows_it()
        {
            var bindings = new GamepadBindings(Consoles);
            Assert.Same(bindings.For("SNES"), bindings.For("SNES", 2));
            bindings.For("SNES").Rebind(PadButton.A, SDL.GamepadButton.North);
            Assert.Equal(SDL.GamepadButton.North, bindings.For("SNES", 3).ButtonToPad[PadButton.A]);
            Assert.False(bindings.HasOwn("SNES", 2));
        }

        // Its own map starts as a copy of player 1's and leaves player 1's alone.
        [Fact]
        public void A_players_own_map_starts_from_player_1s_and_is_its_own()
        {
            var bindings = new GamepadBindings(Consoles);
            bindings.For("SNES").Rebind(PadButton.Y, SDL.GamepadButton.LeftShoulder);
            GamepadBindingMap two = bindings.Own("SNES", 2);
            Assert.Equal(SDL.GamepadButton.LeftShoulder, two.ButtonToPad[PadButton.Y]);

            two.Rebind(PadButton.B, SDL.GamepadButton.North);
            Assert.Equal(SDL.GamepadButton.South, bindings.For("SNES").ButtonToPad[PadButton.B]);
            Assert.Same(two, bindings.For("SNES", 2));
            Assert.Same(bindings.For("NES"), bindings.For("NES", 2));

            bindings.Forget("SNES", 2);
            Assert.Same(bindings.For("SNES"), bindings.For("SNES", 2));
        }

        [Fact]
        public void A_players_map_survives_a_save_and_load_under_its_own_key()
        {
            var bindings = new GamepadBindings(Consoles);
            bindings.Own("SNES", 2).Rebind(PadButton.B, SDL.GamepadButton.North);
            bindings.Save();

            using (JsonDocument json = JsonDocument.Parse(System.IO.File.ReadAllText(File)))
                Assert.Equal(new[] { "NES", "SNES", "SNES Player 2" }, json.RootElement.EnumerateObject().Select(p => p.Name).Order());

            GamepadBindings loaded = GamepadBindings.Load(Consoles);
            Assert.True(loaded.HasOwn("SNES", 2));
            Assert.Equal(SDL.GamepadButton.North, loaded.For("SNES", 2).ButtonToPad[PadButton.B]);
            Assert.Equal(SDL.GamepadButton.South, loaded.For("SNES").ButtonToPad[PadButton.B]);
            Assert.Equal(new[] { "NES", "SNES" }, loaded.ByConsole.Keys.Order());
        }

        // A file from before players: every player plays with player 1's map, and nothing is renamed.
        [Fact]
        public void A_file_written_before_players_gives_every_player_its_consoles_map()
        {
            Directory.CreateDirectory(_dir);
            System.IO.File.WriteAllText(File, """{"SNES":{"A":"North","B":"South"},"NES":{"A":"West"}}""");

            GamepadBindings loaded = GamepadBindings.Load(Consoles);
            Assert.Equal(SDL.GamepadButton.North, loaded.For("SNES", 2).ButtonToPad[PadButton.A]);
            Assert.Equal(SDL.GamepadButton.West, loaded.For("NES", 4).ButtonToPad[PadButton.A]);
            Assert.False(loaded.HasOwn("SNES", 2));

            System.IO.File.WriteAllText(File, """{"A":"North"}""");
            Assert.Equal(SDL.GamepadButton.North, GamepadBindings.Load(Consoles).For("NES", 2).ButtonToPad[PadButton.A]);
        }

        // A build from before players reads a player's key as a console it does not know, and writes it back untouched.
        [Fact]
        public void A_players_key_is_kept_by_a_reader_that_knows_only_consoles()
        {
            var bindings = new GamepadBindings(Consoles);
            bindings.Own("NES", 2).Rebind(PadButton.A, SDL.GamepadButton.West);
            bindings.Save();

            var older = new ConfigFile<System.Collections.Generic.Dictionary<string, System.Collections.Generic.Dictionary<PadButton, SDL.GamepadButton>>>("gamepadbindings.json");
            var read = older.Load()!;
            Assert.True(older.Save(read));
            Assert.Equal(SDL.GamepadButton.West, GamepadBindings.Load(Consoles).For("NES", 2).ButtonToPad[PadButton.A]);
        }

        [Fact]
        public void Reset_to_defaults_returns_every_player_to_player_1s()
        {
            var bindings = new GamepadBindings(Consoles);
            bindings.Own("SNES", 3).Rebind(PadButton.A, SDL.GamepadButton.West);
            bindings.ResetToDefaults();
            Assert.False(bindings.HasOwn("SNES", 3));
            Assert.Equal(SDL.GamepadButton.East, bindings.For("SNES", 3).ButtonToPad[PadButton.A]);
        }

        [Theory]
        [InlineData("SNES Player 2", true)]
        [InlineData("N64 Player 4", true)]
        [InlineData("SNES Player 1", false)]
        [InlineData("SNES Player x", false)]
        public void Only_a_console_and_a_player_past_the_first_is_a_players_key(string key, bool player)
        {
            Directory.CreateDirectory(_dir);
            System.IO.File.WriteAllText(File, """{"SNES":{"A":"East"},""" + "\"" + key + "\"" + """:{"A":"West"}}""");
            GamepadBindings loaded = GamepadBindings.Load(Consoles);
            Assert.Equal(player, !loaded.ByConsole.ContainsKey(key));
        }
    }
}
