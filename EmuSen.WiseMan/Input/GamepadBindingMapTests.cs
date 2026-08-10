using System.Text.Json;
using EmuSen.Cores;
using EmuSen.Endymion.Input;
using SDL3;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Input
{
    // Pins the on-disk bindings contract across the SDL2 -> SDL3 migration -
    // see EmuSen_Settings_Reference.md §4.6.
    [Collection(TestCollections.ProcessGlobals)]
    public class GamepadBindingMapTests
    {
        // Verbatim gamepadbindings.json written by the Silk.NET.SDL build.
        private const string LegacyJson = """
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

        [Fact]
        public void Sdl2_era_config_still_deserializes_to_the_same_physical_buttons()
        {
            var loaded = JsonSerializer.Deserialize<Dictionary<PadButton, SDL.GamepadButton>>(LegacyJson);

            Assert.NotNull(loaded);
            Assert.Equal(SDL.GamepadButton.DPadUp, loaded[PadButton.Up]);
            Assert.Equal(SDL.GamepadButton.DPadDown, loaded[PadButton.Down]);
            Assert.Equal(SDL.GamepadButton.DPadLeft, loaded[PadButton.Left]);
            Assert.Equal(SDL.GamepadButton.DPadRight, loaded[PadButton.Right]);
            Assert.Equal(SDL.GamepadButton.South, loaded[PadButton.B]);
            Assert.Equal(SDL.GamepadButton.East, loaded[PadButton.A]);
            Assert.Equal(SDL.GamepadButton.West, loaded[PadButton.Y]);
            Assert.Equal(SDL.GamepadButton.North, loaded[PadButton.X]);
            Assert.Equal(SDL.GamepadButton.LeftShoulder, loaded[PadButton.L]);
            Assert.Equal(SDL.GamepadButton.RightShoulder, loaded[PadButton.R]);
            Assert.Equal(SDL.GamepadButton.Start, loaded[PadButton.Start]);
            Assert.Equal(SDL.GamepadButton.Back, loaded[PadButton.Select]);
        }

        [Fact]
        public void Defaults_round_trip_through_the_same_json_shape()
        {
            var defaults = new GamepadBindingMap().ButtonToPad;

            string json = JsonSerializer.Serialize(defaults);
            var reloaded = JsonSerializer.Deserialize<Dictionary<PadButton, SDL.GamepadButton>>(json);

            Assert.Equal(defaults, reloaded);
            Assert.Equal(JsonSerializer.Deserialize<Dictionary<PadButton, SDL.GamepadButton>>(LegacyJson), reloaded);
        }
    }
}
