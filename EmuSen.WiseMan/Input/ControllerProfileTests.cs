using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using SDL3;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;
using EmuSen.Endymion.Input;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Input
{
    // Named controller profiles per console beside Default, store 1.1 - see EmuSen_Input.md §10.
    [Collection(TestCollections.ProcessGlobals)]
    public class ControllerProfileTests : IDisposable
    {
        private static readonly string[] Consoles = { "NES", "SNES" };

        private readonly string _dir;

        public ControllerProfileTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenProfiles_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _dir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private void Write(string fileName, string json)
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllText(Path.Combine(_dir, fileName), json);
        }

        private string Read(string fileName) => File.ReadAllText(Path.Combine(_dir, fileName));

        [Fact]
        public void Profiles_are_created_renamed_and_deleted_by_name_and_each_keeps_its_own_maps()
        {
            var bindings = new GamepadBindings(Consoles);
            bindings.For("SNES").Rebind(PadButton.A, SDL.GamepadButton.South);

            Assert.True(bindings.CreateProfile("SNES", "Couch"));
            Assert.Equal(new[] { "Default", "Couch" }, bindings.Profiles("SNES"));
            Assert.Equal(SDL.GamepadButton.South, bindings.For("SNES", 1, "Couch").ButtonToPad[PadButton.A]);

            bindings.Own("SNES", 1, "Couch").Rebind(PadButton.A, SDL.GamepadButton.North);
            Assert.Equal(SDL.GamepadButton.South, bindings.For("SNES").ButtonToPad[PadButton.A]);
            Assert.Equal(SDL.GamepadButton.North, bindings.For("SNES", 1, "Couch").ButtonToPad[PadButton.A]);

            Assert.False(bindings.CreateProfile("SNES", "couch"), "a name is taken whatever its case");
            Assert.False(bindings.CreateProfile("SNES", "  "), "a blank name");
            Assert.True(bindings.CreateProfile("SNES", "Arcade", copyOf: "Couch"));
            Assert.Equal(SDL.GamepadButton.North, bindings.For("SNES", 1, "Arcade").ButtonToPad[PadButton.A]);
            Assert.Equal(new[] { "Default" }, bindings.Profiles("NES"));

            Assert.True(bindings.RenameProfile("SNES", "Couch", "Party"));
            Assert.False(bindings.RenameProfile("SNES", "Couch", "Other"), "no Couch any more");
            Assert.False(bindings.RenameProfile("SNES", "Party", "arcade"), "Arcade is taken");
            Assert.Equal(new[] { "Default", "Party", "Arcade" }, bindings.Profiles("SNES"));
            Assert.Equal(SDL.GamepadButton.North, bindings.For("SNES", 1, "Party").ButtonToPad[PadButton.A]);

            Assert.True(bindings.DeleteProfile("SNES", "Party"));
            Assert.False(bindings.DeleteProfile("SNES", "Party"));
            Assert.Equal(new[] { "Default", "Arcade" }, bindings.Profiles("SNES"));
        }

        [Fact]
        public void Default_is_always_there_and_can_be_neither_deleted_renamed_nor_named_again()
        {
            var bindings = new GamepadBindings(Consoles);
            Assert.Equal(GamepadBindings.DefaultProfile, bindings.Profiles("SNES")[0]);
            Assert.False(bindings.DeleteProfile("SNES", "Default"));
            Assert.False(bindings.DeleteProfile("SNES", "default"));
            Assert.False(bindings.RenameProfile("SNES", "Default", "Mine"));
            Assert.False(bindings.CreateProfile("SNES", "DEFAULT"));
            Assert.True(bindings.CreateProfile("SNES", "Mine"));
            Assert.False(bindings.RenameProfile("SNES", "Mine", "Default"));
            Assert.Equal(new[] { "Default", "Mine" }, bindings.Profiles("SNES"));
            Assert.Equal(new[] { "Default" }, bindings.Profiles("Genesis"));
            Assert.Same(bindings.For("SNES"), bindings.For("SNES", 1, "Default"));
        }

        [Fact]
        public void Bindings_written_before_profiles_load_whole_as_Default_and_save_back_in_the_shape_an_older_build_reads()
        {
            string before = "{\"SNES\":{\"A\":\"West\",\"B\":\"North\"},\"SNES Player 2\":{\"A\":\"South\"},\"NES\":{\"Start\":\"Back\"}}";
            Write("gamepadbindings.json", before);

            GamepadBindings loaded = GamepadBindings.Load(Consoles);

            Assert.Equal(new[] { "Default" }, loaded.Profiles("SNES"));
            Assert.Equal("Default", loaded.ConsoleDefault("SNES"));
            Assert.Equal("Default", loaded.Resolve("SNES"));
            Assert.Equal(SDL.GamepadButton.West, loaded.For("SNES", 1, "Default").ButtonToPad[PadButton.A]);
            Assert.Equal(SDL.GamepadButton.North, loaded.For("SNES", 1, "Default").ButtonToPad[PadButton.B]);
            Assert.True(loaded.HasOwn("SNES", 2, "Default"));
            Assert.Equal(SDL.GamepadButton.South, loaded.For("SNES", 2, "Default").ButtonToPad[PadButton.A]);
            Assert.Equal(SDL.GamepadButton.Back, loaded.For("NES", 1, "Default").ButtonToPad[PadButton.Start]);

            loaded.CreateProfile("SNES", "Couch");
            loaded.Save();
            using var saved = JsonDocument.Parse(Read("gamepadbindings.json"));
            Assert.Equal(new[] { "NES", "SNES", "SNES Player 2" }, saved.RootElement.EnumerateObject().Select(p => p.Name).OrderBy(n => n, StringComparer.Ordinal));
            Assert.Equal("West", saved.RootElement.GetProperty("SNES").GetProperty("A").GetString());
            Assert.Equal("South", saved.RootElement.GetProperty("SNES Player 2").GetProperty("A").GetString());
            Assert.DoesNotContain("Couch", Read("gamepadbindings.json"));
        }

        [Fact]
        public void A_game_starts_with_its_own_profile_else_the_consoles_default_and_the_choice_survives_a_save_rename_and_delete()
        {
            var bindings = new GamepadBindings(Consoles);
            bindings.CreateProfile("SNES", "Couch");
            bindings.Own("SNES", 2, "Couch").Rebind(PadButton.Start, SDL.GamepadButton.Guide);
            Assert.False(bindings.ChooseDefault("SNES", "Missing"));
            Assert.True(bindings.ChooseDefault("SNES", "couch"));

            Assert.Equal("Couch", bindings.ConsoleDefault("SNES"));
            Assert.Equal("Couch", bindings.Resolve("SNES"));
            Assert.Equal("Couch", bindings.Resolve("SNES", "Gone"));
            Assert.Equal("Default", bindings.Resolve("SNES", "default"));
            Assert.Equal("Default", bindings.Resolve("NES", "Couch"));
            bindings.Save();

            GamepadBindings loaded = GamepadBindings.Load(Consoles);
            Assert.Equal(new[] { "Default", "Couch" }, loaded.Profiles("SNES"));
            Assert.Equal("Couch", loaded.Resolve("SNES"));
            Assert.Equal(SDL.GamepadButton.Guide, loaded.For("SNES", 2, "Couch").ButtonToPad[PadButton.Start]);
            Assert.False(loaded.HasOwn("SNES", 2, "Default"));

            Assert.True(loaded.RenameProfile("SNES", "Couch", "Party"));
            Assert.Equal("Party", loaded.ConsoleDefault("SNES"));
            Assert.True(loaded.DeleteProfile("SNES", "Party"));
            Assert.Equal("Default", loaded.Resolve("SNES"));
        }
    }
}
