using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The console's FirmwarePreference, auto, replacement or own, as the Rust runtime reads it where a Sega CD disc's BIOS is located, over a firmware folder of the test's own - see EmuSen_Cores.md §3.12.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class FirmwarePreferenceTests : IDisposable
    {
        private const string Console = "Sega CD", Key = "FirmwarePreference";
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenFirmwarePreference_" + Guid.NewGuid().ToString("N"));
        private string Folder => Path.Combine(_root, "MyFirmware");
        private string Bios => Path.Combine(Folder, "bios_CD_U.bin");

        public FirmwarePreferenceTests()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            Directory.CreateDirectory(Folder);
            new AppSettings { FirmwareDirectory = Folder }.Save();
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private string Disc()
        {
            string at = Path.Combine(_root, "game.iso");
            File.WriteAllBytes(at, SyntheticMdRom.Disc("U"));
            return at;
        }

        // The [Firmware] lines the load says, under the preference set for the console first.
        private List<string> Load(string? preference)
        {
            var was = System.Console.Out;
            var text = new StringWriter();
            using var runtime = new CrystalRuntime();
            using var log = new CrystalRuntimeLog(runtime);
            if (preference is not null) runtime.SetSetting(Console, Key, preference);
            System.Console.SetOut(text);
            try
            {
                using (runtime.Open(new CrystalOpen { Path = Disc(), NoBattery = true })) log.File();
            }
            finally
            {
                System.Console.SetOut(was);
            }
            return text.ToString().Split('\n', StringSplitOptions.TrimEntries).Where(l => l.StartsWith("[Firmware]")).ToList();
        }

        private string Players => $"[Firmware] The American Sega CD's BIOS (131,072 bytes): the player's bios_CD_U.bin, in {Folder}";
        private const string Replacement = "[Firmware] The American Sega CD's BIOS (131,072 bytes): Nephrite's own replacement";

        [RustImplementationFact("FirmwarePreference")]
        public void The_console_offers_the_row_with_its_three_choices_and_auto_by_default()
        {
            using var runtime = new CrystalRuntime();
            using JsonDocument schema = JsonDocument.Parse(runtime.SettingsSchema(Console));
            JsonElement row = schema.RootElement.EnumerateArray().Single(r => r.GetProperty("key").GetString() == Key);
            Assert.Equal("choice", row.GetProperty("kind").GetString());
            Assert.Equal("auto", row.GetProperty("default").GetString());
            Assert.Equal(new[] { "auto", "replacement", "own" }, row.GetProperty("choices").EnumerateArray().Select(c => c.GetString()));
            Assert.Equal("auto", runtime.Setting(Console, Key));
            Assert.Equal(CrystalSessionAbi.BadArgument, Assert.Throws<CrystalException>(() => runtime.SetSetting(Console, Key, "mine")).Status);
            // A console with no firmware to choose has no row.
            using JsonDocument nes = JsonDocument.Parse(runtime.SettingsSchema("NES"));
            Assert.DoesNotContain(nes.RootElement.EnumerateArray(), r => r.GetProperty("key").GetString() == Key);
        }

        [RustImplementationFact("FirmwarePreference")]
        public void Auto_uses_the_players_dump_when_found_and_the_replacement_otherwise()
        {
            Assert.Equal(new[] { Replacement }, Load("auto"));
            File.WriteAllBytes(Bios, new byte[0x2_0000]);
            Assert.Equal(new[] { Players }, Load(null));
            Assert.Equal(new[] { Players }, Load("auto"));
        }

        [RustImplementationFact("FirmwarePreference")]
        public void Replacement_uses_the_replacement_even_with_the_players_dump_present()
        {
            File.WriteAllBytes(Bios, new byte[0x2_0000]);
            Assert.Equal(new[] { Replacement }, Load("replacement"));
        }

        [RustImplementationFact("FirmwarePreference")]
        public void Own_uses_the_players_dump_and_without_one_the_launch_fails_in_a_plain_sentence()
        {
            File.WriteAllBytes(Bios, new byte[0x2_0000]);
            Assert.Equal(new[] { Players }, Load("own"));
            File.Delete(Bios);
            CrystalException refused = Assert.Throws<CrystalException>(() => Load("own"));
            Assert.Equal(CrystalAbi.Firmware, refused.Status);
            Assert.Equal("game.iso cannot start: The American Sega CD's BIOS (131,072 bytes) (bios_CD_U.bin) is not in your firmware folders, and the Sega CD is set to use your own firmware, never the open replacement.", refused.Message);
        }
    }
}
