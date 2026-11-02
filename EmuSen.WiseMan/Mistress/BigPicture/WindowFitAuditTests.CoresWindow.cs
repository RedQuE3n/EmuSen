using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Cores;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Cores window framed in each of its states, fed a scripted listing and a runtime of the test's own so every run is the same: the listing, a core's detail page, Apply's summary, a download running and no catalogue yet - see EmuSen_Cores.md §2.3.
    public partial class WindowFitAuditTests
    {
        internal const string LongCore = "Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)";

        // A runtime of the test's: three of EmuSen's cores and one the player added, waiting for approval; what it was asked is recorded.
        internal sealed class FakeCoresRuntime
        {
            public readonly Dictionary<(string, string), string?> Settings = new();
            public readonly List<string> Approved = new(), Revoked = new();

            public static string Systems => JsonSerializer.Serialize(new object[]
            {
                // PCEE2 first: the PlayStation 2's default while Sapphire is under development (EmuSen_Cores.md §4.8).
                new { id = "ps2", console = "PlayStation 2", engines = new[] { "PCEE2", "Sapphire", "ExtraPS2" } },
                new { id = "psx", console = "PlayStation", engines = new[] { "Diamond" } },
                new { id = "snes", console = "SNES", engines = new[] { "VenusRT" } },
            });

            public static string Cores => JsonSerializer.Serialize(new object[]
            {
                new { library = "/opt/emusen/libsapphire.so", sha256 = new string('1', 64), engine = "Sapphire", directory = "installation", trust = "manifest", name = "Sapphire", authors = new[] { "EmuSen" }, license = "GPL-3.0-or-later", systems = new[] { "ps2" } },
                new { library = "/opt/emusen/libdiamond.so", sha256 = new string('2', 64), engine = "Diamond", directory = "installation", trust = "manifest", name = "Diamond", authors = new[] { "EmuSen" }, license = "GPL-3.0-or-later", systems = new[] { "psx" } },
                new { library = "/opt/emusen/libvenusrt.so", sha256 = new string('3', 64), engine = "VenusRT", directory = "installation", trust = "manifest", name = "VenusRT", authors = new[] { "EmuSen" }, license = "GPL-3.0-or-later", systems = new[] { "snes" } },
                new { library = "/home/player/Cores/libextraps2.so", sha256 = new string('4', 64), engine = "ExtraPS2", directory = "player", trust = "needs_approval", name = "Extra PS2 Core", authors = new[] { "Someone Else" }, license = "MIT", systems = new[] { "ps2" } },
                new { library = "/home/player/Cores/libretro/pcee2/pcee2_libretro.so", sha256 = new string('5', 64), engine = "PCEE2", directory = "player", trust = "approved", name = "PCEE2", authors = new[] { "PCSX2 Team" }, license = "GPLv3", systems = new[] { "ps2" }, libretro = new { sha256 = new string('5', 64) } },
            });

            public CoresRuntime Runtime => new()
            {
                Cores = () => Cores,
                Systems = () => Systems,
                Setting = (console, key) => Settings.TryGetValue((console, key), out string? v) ? v : null,
                SetSetting = (console, key, value) => Settings[(console, key)] = value,
                Approve = Approved.Add,
                Revoke = Revoked.Add,
            };
        }

        // The corename as a real .info gives it, read 2026-11-01 from the smoke test's copies: PCEE2, Beetle PSX, Snes9x; the test's own cores keep their stem.
        internal static string CorenameOf(string stem) => stem switch { "pcee2" => "PCEE2", "mednafen_psx" => "Beetle PSX", "snes9x" => "Snes9x", "glonly" => "Only GL", _ => stem };

        internal static object LibretroCore(string stem, string name, string system, string state, long? size, string? reason = null) =>
            reason is null
                ? new { @event = "core", stem, name, corename = CorenameOf(stem), systems = new[] { system }, source = "libretro", version = "2.3.1", build = "2026-10-31", offered = "2026-10-31", state, license = "GPLv3", authors = new[] { "PCSX2 Team", "libretro" }, size }
                : new { @event = "core", stem, name, corename = CorenameOf(stem), systems = new[] { system }, source = "libretro", version = "1.0", build = "2026-10-30", offered = "2026-10-30", state, license = "MIT", authors = new[] { "Someone" }, size, reason };

        internal static IEnumerable<object> CoresListing(bool catalogue = true)
        {
            if (!catalogue) return [new { @event = "done", op = "core-ls", counts = new Dictionary<string, int>(), catalogue = (string?)null }];
            return
            [
                CoresInfo("pcsx2").First(),
                LibretroCore("pcee2", "Sony - PlayStation 2 (PCEE2)", "ps2", "installed", 41_943_040),
                LibretroCore("mednafen_psx", "Sony - PlayStation (Beetle PSX)", "psx", "update", 8_388_608),
                LibretroCore("snes9x", "Nintendo - SNES / SFC (Snes9x - Current)", "snes", "installed", 3_145_728),
                LibretroCore("glonly", "Only GL Core", "psx", "unsupported", null, "needs OpenGL Core >= 3.3, and EmuSen's host renders through software and Vulkan only"),
                new { @event = "done", op = "core-ls", counts = new Dictionary<string, int> { ["available"] = 1, ["update"] = 1, ["installed"] = 2, ["unsupported"] = 1 }, catalogue = "2026-10-31" },
            ];
        }

        // A row as `core-ls` with detail gives it: core-info's fields beside the listing's.
        internal static IEnumerable<object> CoresInfo(string stem) =>
        [
            new { @event = "core", stem, name = LongCore, corename = "LRPS2", systems = new[] { "ps2" }, source = "libretro", version = "2.3.1", build = "2026-10-31", offered = "2026-10-31", state = "available", license = "GPLv3", authors = new[] { "PCSX2 Team", "libretro" }, size = (long?)null,
                from = $"https://buildbot.libretro.com/nightly/linux/x86_64/latest/{stem}_libretro.so.zip", index_crc32 = "3a224a58",
                firmware = new[] { new { desc = "PlayStation 2 BIOS", path = "pcsx2/bios/ps2.bin", optional = false } }, notes = new[] { "(!) needs a BIOS", "disc images by path" }, permissions = Array.Empty<string>(),
                tested = new { match = "none" } },
            new { @event = "done", op = "core-info", counts = new Dictionary<string, int>(), catalogue = "2026-10-31" },
        ];

        // A job for each request: the listing, a core's detail, and an install that runs and does not end.
        internal static Func<string, IToolJob> CoresJobs(bool catalogue = true) => request =>
        {
            JsonElement r = JsonDocument.Parse(request).RootElement;
            return r.GetProperty("op").GetString() switch
            {
                "core-ls" => new ScriptedJob(CoresListing(catalogue), ends: true),
                _ => new ScriptedJob([new { @event = "plan", stem = "pcsx2", from = "https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip", build = "2026-10-31", crc32 = "3a224a58" },
                    new { @event = "progress", stem = "pcsx2", step = "download", bytes = 18_874_368, total = 43_000_000 }], ends: false),
            };
        };

        // The window opened as Settings opens it, its listing taken, and for each later state the pad's steps taken to reach it.
        internal static void ShowCores(MainWindow window, string state)
        {
            Func<string, IToolJob> realStart = CoresPane.StartJob;
            Func<bool> realAvailable = CoresPane.ToolAvailable;
            Func<string, (long Free, long Total)?> realDisk = CoresPane.Disk;
            Func<CoresRuntime?, CoresRuntime?> realRuntime = CoresPane.RuntimeFor;
            window.Closed += (_, _) =>
            {
                CoresPane.StartJob = realStart;
                CoresPane.ToolAvailable = realAvailable;
                CoresPane.Disk = realDisk;
                CoresPane.RuntimeFor = realRuntime;
            };
            var runtime = new FakeCoresRuntime();
            CoresPane.ToolAvailable = () => true;
            CoresPane.Disk = _ => (410L << 30, 1024L << 30);
            CoresPane.RuntimeFor = _ => runtime.Runtime;
            CoresPane.StartJob = CoresJobs(catalogue: state != "CoresNoCatalogue");
            Call(window, "ShowCores");
            Settle(window);
            var shown = (CoresWindow)Sheets(window).Current!;
            CoresPane pane = shown.Pane;
            pane.Poll();
            Settle(window);
            Assert.Equal(AdaptiveForm.BigScreen, pane.Form);
            Assert.Equal(state == "CoresNoCatalogue" ? 4 : 9, pane.Rows.Count);
            if (state == "CoresNoCatalogue") return;
            CoreRow lrps2 = pane.Rows.First(r => r.Stem == "pcsx2");
            // CoresDetail: the core waiting for approval chosen, its pane beside the list and A asking to approve it.
            pane.BigScreenTable.Select(state == "CoresDetail" ? pane.Rows.First(r => r.AwaitsApproval) : lrps2);
            pane.FocusRole(AdaptiveRole.Content);
            pane.Poll();
            Settle(window);
            if (state == "CoresDetail") Assert.Equal("Approve…", pane.Hints[0].Label);
            if (state is "CoresPlan" or "CoresRunning")
            {
                Assert.True(pane.SetMark(lrps2, CoresPlan.Install.Key));
                Assert.True(pane.SetMark(pane.Rows.First(r => r.Stem == "mednafen_psx"), CoresPlan.Update.Key));
                Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Menu));
                Assert.True(pane.IsPlanOpen);
            }
            if (state == "CoresRunning")
            {
                Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Menu));
                pane.Poll();
                Assert.True(pane.IsRunning);
            }
            Settle(window);
        }
    }
}
