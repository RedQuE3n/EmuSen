using System;
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
    // The runtime's firmware_status over the build's engines and a firmware folder of the test's own: present, missing, not needed - see EmuSen_Cores.md §3.11.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed class FirmwareStatusTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenFirmwareStatus_" + Guid.NewGuid().ToString("N"));
        private string Folder => Path.Combine(_root, "MyFirmware");

        public FirmwareStatusTests()
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

        private static CrystalRuntime Runtime() => new(new CrystalRuntimeOptions { Cores = new[] { AppContext.BaseDirectory } });

        // Each file of the answer as "file required status".
        private static string[] Items(string answer, string system)
        {
            using JsonDocument doc = JsonDocument.Parse(answer);
            Assert.Equal(system, doc.RootElement.GetProperty("system").GetString());
            return doc.RootElement.GetProperty("firmware").EnumerateArray()
                .Select(f => $"{f.GetProperty("file").GetString()} {f.GetProperty("required").GetBoolean()} {f.GetProperty("status").GetString()}").ToArray();
        }

        [Fact]
        public void A_console_s_firmware_is_present_missing_or_not_needed_as_the_folders_hold_it()
        {
            using CrystalRuntime runtime = Runtime();
            // Sapphire's BIOS has no open replacement, so each region's is needed, and none is in either folder yet.
            string[] ps2 = Items(runtime.FirmwareStatus("ps2"), "ps2");
            Assert.Equal(3, ps2.Length);
            Assert.All(ps2, f => Assert.EndsWith(" True missing", f));
            // Diamond's BIOS has an open replacement: not needed while absent.
            string[] psx = Items(runtime.FirmwareStatus("psx"), "psx");
            Assert.NotEmpty(psx);
            Assert.All(psx, f => Assert.EndsWith(" False not_needed", f));
            // The NES declares no firmware at all.
            Assert.Empty(Items(runtime.FirmwareStatus("nes"), "nes"));

            // The player's own folder holds the American BIOS by its name and size; the store holds the American PlayStation's.
            File.WriteAllBytes(Path.Combine(Folder, "ps2-0230a-20080220.bin"), new byte[4 << 20]);
            Directory.CreateDirectory(DataStore.Firmware);
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, "scph5501.bin"), new byte[512 << 10]);
            ps2 = Items(runtime.FirmwareStatus("ps2"), "ps2");
            Assert.Equal(new[] { "ps2-0230j-20080220.bin True missing", "ps2-0230a-20080220.bin True present", "ps2-0230e-20080220.bin True missing" }, ps2);
            Assert.Contains("scph5501.bin False present", Items(runtime.FirmwareStatus("psx"), "psx"));
            // A file of the right name and the wrong size is not the firmware.
            File.WriteAllBytes(Path.Combine(Folder, "ps2-0230j-20080220.bin"), new byte[1024]);
            Assert.Contains("ps2-0230j-20080220.bin True missing", Items(runtime.FirmwareStatus("ps2"), "ps2"));
        }

        // Each file's need, by file name.
        private static System.Collections.Generic.Dictionary<string, string> Needs(string answer)
        {
            using JsonDocument doc = JsonDocument.Parse(answer);
            return doc.RootElement.GetProperty("firmware").EnumerateArray().ToDictionary(f => f.GetProperty("file").GetString()!, f => f.GetProperty("need").GetString()!);
        }

        [Fact]
        public void Files_an_engine_declares_alternatives_share_its_need_and_a_cartridges_chips_do_not()
        {
            using CrystalRuntime runtime = Runtime();
            Assert.True(CrystalAbi.Minor >= 11);
            // Diamond declares its three market BIOSes one need, each in a slot of its own.
            var psx = Needs(runtime.FirmwareStatus("psx"));
            Assert.Equal(3, psx.Count);
            Assert.Single(psx.Values.Distinct());
            // Sapphire declares the PlayStation 2's likewise.
            Assert.Single(Needs(runtime.FirmwareStatus("ps2")).Values.Distinct());
            // VenusRT declares each coprocessor's program a need of its own, though they share a slot and many a size.
            var snes = Needs(runtime.FirmwareStatus("snes"));
            Assert.True(snes.Count > 1, string.Join(", ", snes.Keys));
            Assert.Equal(snes.Count, snes.Values.Distinct().Count());
        }

        [Fact]
        public void A_system_no_table_or_engine_names_is_absent_and_none_is_refused()
        {
            using CrystalRuntime runtime = Runtime();
            Assert.Equal(CrystalSessionAbi.Absent, Assert.Throws<CrystalException>(() => runtime.FirmwareStatus("no-such-console")).Status);
            Assert.Equal(CrystalSessionAbi.BadArgument, Assert.Throws<CrystalException>(() => runtime.FirmwareStatus("")).Status);
        }
    }
}
