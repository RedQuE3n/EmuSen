using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Endymion.Input;
using SDL3;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // Enum values are written as names rather than numbers - see
    // EmuSen_Config_Reference.md §2.1. Numbers still read, so no config file
    // written before this change is orphaned.
    [Collection(TestCollections.ProcessGlobals)]
    public class ConfigEnumFormatTests : IDisposable
    {
        // Verbatim gamepadbindings.json as every build before this one wrote it.
        private const string NumericJson = """
            {
              "Up": 11,
              "Down": 12,
              "Left": 13,
              "Right": 14,
              "B": 0,
              "A": 1,
              "Y": 2,
              "X": 3,
              "L": 9,
              "R": 10,
              "Start": 6,
              "Select": 4
            }
            """;

        private readonly string _dir;

        public ConfigEnumFormatTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenEnumFmt_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _dir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private void WriteBindings(string json)
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllText(Path.Combine(_dir, "gamepadbindings.json"), json);
        }

        [Fact]
        public void A_numeric_file_from_an_older_build_still_maps_to_the_same_buttons()
        {
            WriteBindings(NumericJson);

            GamepadBindingMap map = GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Equal(SDL.GamepadButton.DPadUp, map.ButtonToPad[PadButton.Up]);
            Assert.Equal(SDL.GamepadButton.South, map.ButtonToPad[PadButton.B]);
            Assert.Equal(SDL.GamepadButton.East, map.ButtonToPad[PadButton.A]);
            Assert.Equal(SDL.GamepadButton.LeftShoulder, map.ButtonToPad[PadButton.L]);
            Assert.Equal(SDL.GamepadButton.Back, map.ButtonToPad[PadButton.Select]);
        }

        [Fact]
        public void Saving_writes_names_rather_than_numbers()
        {
            new GamepadBindings(new[] { "NES", "SNES" }).Save();

            string json = File.ReadAllText(Path.Combine(_dir, "gamepadbindings.json"));

            Assert.Contains("\"DPadUp\"", json);
            Assert.Contains("\"South\"", json);
            Assert.DoesNotContain(": 11", json);
            Assert.DoesNotContain(": 0,", json);
        }

        // The upgrade path in one test: read the old file, save, read it back.
        [Fact]
        public void A_numeric_file_is_rewritten_as_names_without_changing_meaning()
        {
            WriteBindings(NumericJson);

            GamepadBindings loaded = GamepadBindings.Load(new[] { "NES", "SNES" });
            GamepadBindingMap before = loaded.For("NES");
            loaded.Save();
            GamepadBindingMap after = GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Contains("\"DPadUp\"", File.ReadAllText(Path.Combine(_dir, "gamepadbindings.json")));
            Assert.Equal(before.ButtonToPad, after.ButtonToPad);
        }

        // Someone editing one line of a file the app wrote earlier.
        [Fact]
        public void A_half_edited_file_mixing_names_and_numbers_loads()
        {
            WriteBindings("""{"Up":"DPadUp","Down":12,"B":"South","A":1}""");

            GamepadBindingMap map = GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Equal(SDL.GamepadButton.DPadUp, map.ButtonToPad[PadButton.Up]);
            Assert.Equal(SDL.GamepadButton.DPadDown, map.ButtonToPad[PadButton.Down]);
            Assert.Equal(SDL.GamepadButton.South, map.ButtonToPad[PadButton.B]);
            Assert.Equal(SDL.GamepadButton.East, map.ButtonToPad[PadButton.A]);
        }

        [Fact]
        public void Names_are_matched_without_regard_to_case()
        {
            WriteBindings("""{"Up":"dpadup","B":"SOUTH"}""");

            GamepadBindingMap map = GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Equal(SDL.GamepadButton.DPadUp, map.ButtonToPad[PadButton.Up]);
            Assert.Equal(SDL.GamepadButton.South, map.ButtonToPad[PadButton.B]);
        }

        // The cost of names: a misspelling fails the whole file rather than one
        // entry, and the best-effort contract turns that into defaults. Pinned
        // so the behaviour is a known one - see EmuSen_Config_Reference.md §2.1.
        [Fact]
        public void A_misspelled_name_falls_back_to_defaults_rather_than_throwing()
        {
            WriteBindings("""{"Up":"DPadUpp","B":"South"}""");

            GamepadBindingMap map = GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Equal(new GamepadBindingMap().ButtonToPad, map.ButtonToPad);
        }

        [Fact]
        public void Dictionary_keys_were_already_names_and_still_are()
        {
            new GamepadBindings(new[] { "NES", "SNES" }).Save();

            string json = File.ReadAllText(Path.Combine(_dir, "gamepadbindings.json"));

            Assert.Contains("\"Start\":", json);
            Assert.Contains("\"Select\":", json);
        }

        [Fact]
        public void The_other_two_binding_files_get_the_same_treatment()
        {
            var file = new ConfigFile<Dictionary<PadButton, SDL.GamepadButton>>("sample.json");
            file.Save(new Dictionary<PadButton, SDL.GamepadButton> { [PadButton.Start] = SDL.GamepadButton.Start });

            Assert.Contains("\"Start\": \"Start\"", File.ReadAllText(file.Path));
        }
    }
}
