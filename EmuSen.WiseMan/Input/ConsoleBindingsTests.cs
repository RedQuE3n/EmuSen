using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Input;
using SDL3;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Input;
using EmuSen.Mistress.Input;
using EmuSen.Nehellania.Input;

namespace EmuSen.WiseMan.Input
{
    // Bindings are per console, and a file written before they were - see EmuSen_Input.md §5.1.
    public class ConsoleBindingsTests : IDisposable
    {
        private static readonly string[] Consoles = { "NES", "SNES" };

        private readonly string _dir;

        public ConsoleBindingsTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenBindings_" + Guid.NewGuid().ToString("N"));
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

        // --- Keyboard ---

        [Fact]
        public void Each_console_keeps_its_own_keys_through_a_save_and_load()
        {
            var bindings = new ControllerKeyBindings(Consoles);
            bindings.For("NES").Rebind(PadButton.A, Key.N);
            bindings.For("SNES").Rebind(PadButton.A, Key.S);
            bindings.Save();

            ControllerKeyBindings loaded = ControllerKeyBindings.Load(Consoles);

            Assert.Equal(Key.N, loaded.For("NES").ButtonToKey[PadButton.A]);
            Assert.Equal(Key.S, loaded.For("SNES").ButtonToKey[PadButton.A]);
        }

        [Fact]
        public void Rebinding_one_console_does_not_disturb_the_other()
        {
            var bindings = new ControllerKeyBindings(Consoles);
            Key before = bindings.For("SNES").ButtonToKey[PadButton.B];

            bindings.For("NES").Rebind(PadButton.B, Key.J);

            Assert.Equal(Key.J, bindings.For("NES").ButtonToKey[PadButton.B]);
            Assert.Equal(before, bindings.For("SNES").ButtonToKey[PadButton.B]);
        }

        // The two consoles are free to share a key, which one map could never express.
        [Fact]
        public void Two_consoles_may_bind_the_same_key()
        {
            var bindings = new ControllerKeyBindings(Consoles);

            bindings.For("NES").Rebind(PadButton.A, Key.K);
            bindings.For("SNES").Rebind(PadButton.Start, Key.K);

            Assert.Equal(Key.K, bindings.For("NES").ButtonToKey[PadButton.A]);
            Assert.Equal(Key.K, bindings.For("SNES").ButtonToKey[PadButton.Start]);
        }

        // The flat file used to mean "these keys, on whatever is loaded".
        [Fact]
        public void A_flat_legacy_keyboard_file_is_inherited_by_every_console()
        {
            Write("keybindings.json", """{"A":"K","B":"L"}""");

            ControllerKeyBindings loaded = ControllerKeyBindings.Load(Consoles);

            foreach (string console in Consoles)
            {
                Assert.Equal(Key.K, loaded.For(console).ButtonToKey[PadButton.A]);
                Assert.Equal(Key.L, loaded.For(console).ButtonToKey[PadButton.B]);
            }
        }

        [Fact]
        public void A_migrated_flat_file_is_rewritten_per_console()
        {
            Write("keybindings.json", """{"A":"K"}""");

            ControllerKeyBindings.Load(Consoles).Save();

            string json = Read("keybindings.json");
            Assert.Contains("\"NES\"", json);
            Assert.Contains("\"SNES\"", json);
            Assert.Equal(Key.K, ControllerKeyBindings.Load(Consoles).For("NES").ButtonToKey[PadButton.A]);
        }

        // Migration must copy, not alias - editing one console afterwards must not move the other.
        [Fact]
        public void Consoles_that_inherited_a_flat_file_do_not_share_one_dictionary()
        {
            Write("keybindings.json", """{"A":"K"}""");

            ControllerKeyBindings loaded = ControllerKeyBindings.Load(Consoles);
            loaded.For("NES").Rebind(PadButton.A, Key.Z);

            Assert.Equal(Key.Z, loaded.For("NES").ButtonToKey[PadButton.A]);
            Assert.Equal(Key.K, loaded.For("SNES").ButtonToKey[PadButton.A]);
        }

        [Fact]
        public void A_console_missing_from_the_file_falls_back_to_defaults()
        {
            Write("keybindings.json", """{"SNES":{"A":"K"}}""");

            ControllerKeyBindings loaded = ControllerKeyBindings.Load(Consoles);

            Assert.Equal(Key.K, loaded.For("SNES").ButtonToKey[PadButton.A]);
            Assert.Equal(new ControllerKeyMap().ButtonToKey, loaded.For("NES").ButtonToKey);
        }

        // --- Gamepad ---

        [Fact]
        public void Each_console_keeps_its_own_pad_through_a_save_and_load()
        {
            var bindings = new GamepadBindings(Consoles);
            bindings.For("NES").Rebind(PadButton.A, SDL.GamepadButton.North);
            bindings.For("SNES").Rebind(PadButton.A, SDL.GamepadButton.South);
            bindings.Save();

            GamepadBindings loaded = GamepadBindings.Load(Consoles);

            Assert.Equal(SDL.GamepadButton.North, loaded.For("NES").ButtonToPad[PadButton.A]);
            Assert.Equal(SDL.GamepadButton.South, loaded.For("SNES").ButtonToPad[PadButton.A]);
        }

        [Fact]
        public void A_flat_legacy_gamepad_file_is_inherited_by_every_console()
        {
            Write("gamepadbindings.json", """{"A":"North"}""");

            GamepadBindings loaded = GamepadBindings.Load(Consoles);

            foreach (string console in Consoles)
            {
                Assert.Equal(SDL.GamepadButton.North, loaded.For(console).ButtonToPad[PadButton.A]);
            }
        }

        // --- The identifier the file is keyed on ---

        // CoreCatalog names the tab and the config key; ICore.CoreName names the
        // running core. MainWindow assigns one to the other, so they must agree.
        [Fact]
        public void Every_cores_console_name_matches_what_the_catalog_calls_it()
        {
            var catalogNames = CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToHashSet();

            Assert.Contains(new EmuSen.Cores.Nintendo.Moon.MoonCore().CoreName, catalogNames);
            Assert.Contains(new EmuSen.Cores.Nintendo.Venus.VenusCore().CoreName, catalogNames);
        }

        [Fact]
        public void The_catalog_lists_consoles_by_manufacturer_then_release()
        {
            string[] order = CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console).ToArray();

            Assert.Equal(new[] { "NES", "SNES" }, order);
        }

        // The rebind window lists a console's pad with no ROM loaded, so this must
        // come off the core rather than being restated - see EmuSen_Input.md §5.1.
        [Fact]
        public void The_catalogs_button_list_is_the_cores_own()
        {
            Assert.Equal(new EmuSen.Cores.Nintendo.Moon.MoonCore().SupportedButtons, CoreCatalog.ButtonsFor("NES"));
            Assert.Equal(new EmuSen.Cores.Nintendo.Venus.VenusCore().SupportedButtons, CoreCatalog.ButtonsFor("SNES"));
        }

        [Fact]
        public void An_unknown_console_gets_the_whole_button_union()
        {
            Assert.Equal(Enum.GetValues<PadButton>(), CoreCatalog.ButtonsFor("Dreamcast"));
        }
    }
}
