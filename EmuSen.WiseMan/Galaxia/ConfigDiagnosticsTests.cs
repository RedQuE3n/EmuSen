using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Nehellania.Input;
using SDL3;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Galaxia
{
    // A config file that won't parse still falls back to defaults, but says
    // so and names the value - see EmuSen_Config_Reference.md §6.2.
    public class ConfigDiagnosticsTests : IDisposable
    {
        private readonly string _dir;
        private readonly List<string> _reported = new();

        public ConfigDiagnosticsTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenDiag_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _dir;
            ConfigDiagnostics.Reset();
            ConfigDiagnostics.Sink = _reported.Add;
        }

        public void Dispose()
        {
            ConfigDiagnostics.Sink = null;
            ConfigDiagnostics.Reset();
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private void WriteBindings(string json)
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllText(Path.Combine(_dir, "gamepadbindings.json"), json);
        }

        [Fact]
        public void A_misspelled_pad_button_is_named_and_corrected()
        {
            WriteBindings("""{"Up":"DPadUpp","B":"South"}""");

            GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            string message = Assert.Single(_reported);
            Assert.Contains("'DPadUpp' is not a valid GamepadButton", message);
            Assert.Contains("Did you mean 'DPadUp'?", message);
        }

        // Behaviour is unchanged - the file still can't load. It just isn't
        // silent about it anymore.
        [Fact]
        public void The_fallback_to_defaults_still_happens_and_is_stated()
        {
            WriteBindings("""{"Up":"DPadUpp"}""");

            GamepadBindingMap map = GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Equal(new GamepadBindingMap().ButtonToPad, map.ButtonToPad);
            Assert.Contains("Falling back to defaults", Assert.Single(_reported));
        }

        [Fact]
        public void The_report_names_the_file_it_came_from()
        {
            WriteBindings("""{"Up":"DPadUpp"}""");

            GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Contains(Path.Combine(_dir, "gamepadbindings.json"), Assert.Single(_reported));
        }

        // Dictionary keys go through a different converter path than values.
        [Fact]
        public void A_misspelled_snes_button_key_is_corrected_too()
        {
            WriteBindings("""{"Strat":"Start"}""");

            GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            string message = Assert.Single(_reported);
            Assert.Contains("'Strat' is not a valid PadButton", message);
            Assert.Contains("Did you mean 'Start'?", message);
        }

        [Fact]
        public void A_value_unlike_any_real_one_is_named_without_a_guess()
        {
            WriteBindings("""{"Up":"qwertyuiop"}""");

            GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            string message = Assert.Single(_reported);
            Assert.Contains("'qwertyuiop' is not a valid", message);
            Assert.DoesNotContain("Did you mean", message);
        }

        [Fact]
        public void Malformed_json_is_reported_as_well()
        {
            WriteBindings("{ not json at all");

            GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Single(_reported);
        }

        // A file that simply isn't there yet is the normal first-run case,
        // not something to complain about.
        [Fact]
        public void A_missing_file_reports_nothing()
        {
            GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Empty(_reported);
            Assert.Null(ConfigDiagnostics.LastMessage);
        }

        [Fact]
        public void A_file_that_loads_reports_nothing()
        {
            new GamepadBindings(new[] { "NES", "SNES" }).Save();

            GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.Empty(_reported);
        }

        [Fact]
        public void LastLoadError_is_set_on_failure_and_cleared_on_success()
        {
            var file = new ConfigFile<Dictionary<PadButton, SDL.GamepadButton>>("probe.json");
            Directory.CreateDirectory(_dir);
            File.WriteAllText(file.Path, """{"Up":"DPadUpp"}""");

            Assert.Null(file.Load());
            Assert.NotNull(file.LastLoadError);

            file.Save(new Dictionary<PadButton, SDL.GamepadButton> { [PadButton.Up] = SDL.GamepadButton.DPadUp });

            Assert.NotNull(file.Load());
            Assert.Null(file.LastLoadError);
        }

        // Nothing is required to listen; the default is no sink at all.
        [Fact]
        public void Reporting_with_no_sink_registered_does_not_throw()
        {
            ConfigDiagnostics.Sink = null;
            WriteBindings("""{"Up":"DPadUpp"}""");

            GamepadBindings.Load(new[] { "NES", "SNES" }).For("NES");

            Assert.NotNull(ConfigDiagnostics.LastMessage);
        }
    }
}
