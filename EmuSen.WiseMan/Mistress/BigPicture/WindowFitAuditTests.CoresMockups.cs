using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Cores pictured in the four states of its approved mockups, from a listing holding the mockups' own cores, beside which the comparison is made - see EmuSen_Cores.md §3.8.
    public partial class WindowFitAuditTests
    {
        public static TheoryData<string, int, int> CoresMockupCases()
        {
            var data = new TheoryData<string, int, int>();
            foreach (string state in new[] { "list", "waiting", "approve", "installing" })
                foreach ((int w, int h) in new[] { (1280, 800), (1920, 1080) }) data.Add(state, w, h);
            return data;
        }

        // The mockups' consoles, with the systems table's names, makers and years.
        private static readonly (string Id, string Console, string Shown, string Maker, int Year, string? Firmware, string[] Engines)[] MockupSystems =
        [
            ("nes", "NES", "NES", "Nintendo", 1983, "Famicom Disk System BIOS", ["MoonRT"]),
            ("snes", "SNES", "Super Nintendo", "Nintendo", 1990, null, ["VenusRT"]),
            ("md", "Genesis", "Genesis", "Sega", 1988, null, ["Nephrite"]),
            ("psx", "PlayStation", "PlayStation", "Sony", 1994, "PlayStation BIOS", ["Diamond"]),
            ("n64", "N64", "Nintendo 64", "Nintendo", 1996, null, ["MarsRT"]),
            ("ps2", "PlayStation 2", "PlayStation 2", "Sony", 2000, "PlayStation 2 BIOS", ["PCEE2", "Sapphire"]),
        ];

        private static readonly (string Name, string System)[] MockupProjectCores = [("MoonRT", "nes"), ("VenusRT", "snes"), ("Nephrite", "md"), ("Diamond", "psx"), ("MarsRT", "n64"), ("Sapphire", "ps2")];

        // The libretro cores: stem, corename, the name libretro lists, system, state.
        private static readonly (string Stem, string Core, string Listed, string System, string State)[] MockupLibretro =
        [
            ("pcee2", "PCEE2", "Sony - PlayStation 2 (LRPS2)", "ps2", "available"),
            ("mednafen_psx_hw", "Beetle PSX HW", "Sony - PlayStation (Beetle PSX HW)", "psx", "update"),
            ("pcsx_rearmed", "PCSX ReARMed", "Sony - PlayStation (PCSX ReARMed)", "psx", "available"),
            ("snes9x", "Snes9x", "Nintendo - SNES / SFC (Snes9x - Current)", "snes", "installed"),
            ("bsnes", "bsnes", "Nintendo - SNES / SFC (bsnes)", "snes", "available"),
            ("nestopia", "Nestopia UE", "Nintendo - NES / Famicom (Nestopia UE)", "nes", "available"),
            ("fceumm", "FCEUmm", "Nintendo - NES / Famicom (FCEUmm)", "nes", "available"),
            ("genesis_plus_gx", "Genesis Plus GX", "Sega - MS/GG/MD/CD (Genesis Plus GX)", "md", "available"),
            ("picodrive", "PicoDrive", "Sega - MS/MD/CD/32X (PicoDrive)", "md", "available"),
            ("mupen64plus_next", "Mupen64Plus-Next", "Nintendo - Nintendo 64 (Mupen64Plus-Next)", "n64", "available"),
            ("parallel_n64", "ParaLLEl N64", "Nintendo - Nintendo 64 (ParaLLEl N64)", "n64", "available"),
        ];

        private const string Pcee2Hash = "9f2c3a110b7e44d28c1f0e9a44d2e01a5b3c7d19e8f00a2173c4ab901d2e3f4a";

        // The runtime's side: the project's cores, and PCEE2 once the install has run, waiting for approval.
        private sealed class MockupRuntime
        {
            public bool Pcee2Installed;
            public string Pcee2Trust = "needs_approval";
            public readonly Dictionary<(string, string), string?> Settings = new();

            public CoresRuntime Runtime => new()
            {
                Systems = () => JsonSerializer.Serialize(MockupSystems.Select(s => new { id = s.Id, console = s.Console, display_name = s.Shown, manufacturer = s.Maker, release_year = s.Year, firmware_name = s.Firmware, engines = s.Engines })),
                Cores = () => JsonSerializer.Serialize(MockupProjectCores.Select(c => (object)new { library = $"/opt/emusen/lib{c.Name.ToLowerInvariant()}.so", sha256 = new string('1', 64), engine = c.Name, directory = "installation", trust = "manifest", name = c.Name, authors = new[] { "the EmuSen project" }, license = "GPL-3.0-or-later", systems = new[] { c.System }, development = c.Name == "Sapphire", firmware = c.System switch { "ps2" => "required", "psx" => "optional", _ => (string?)null } })
                    .Concat(Pcee2Installed ? [new { library = "/home/player/Cores/libretro/pcee2/pcee2_libretro.so", sha256 = Pcee2Hash, engine = "PCEE2", directory = "player", trust = Pcee2Trust, name = "PCEE2", authors = new[] { "the PCSX2 Team" }, license = "GPLv3", systems = new[] { "ps2" }, libretro = new { sha256 = Pcee2Hash } }] : Array.Empty<object>())),
                Setting = (console, key) => Settings.TryGetValue((console, key), out string? v) ? v : null,
                SetSetting = (console, key, value) => Settings[(console, key)] = value,
                Approve = _ => { },
                Revoke = _ => { },
                // The list's mockup says "✓ You have one." under Sapphire's need: the player's PlayStation 2 BIOS is found.
                Firmware = system => JsonSerializer.Serialize(new { system, firmware = system == "ps2" ? new[] { new { label = "PlayStation 2 BIOS (Americas)", file = "ps2-0230a-20080220.bin", required = true, status = "present" } } : [] }),
            };
        }

        private static IEnumerable<object> MockupListing(MockupRuntime runtime)
        {
            foreach (var c in MockupLibretro)
            {
                string state = c.Stem == "pcee2" && runtime.Pcee2Installed ? "installed" : c.State;
                yield return new { @event = "core", stem = c.Stem, name = c.Listed, corename = c.Core, systems = new[] { c.System }, source = "libretro", version = "", build = "2026-10-31", offered = "2026-10-31", state, license = "GPLv3", authors = new[] { "the PCSX2 Team" }, size = 42_991_616L,
                    firmware = c.Stem == "pcee2" ? new[] { new { desc = "PlayStation 2 BIOS", path = "pcsx2/bios/ps2.bin", optional = false } } : [], sha256 = c.Stem == "pcee2" && runtime.Pcee2Installed ? Pcee2Hash : null, fetched = c.Stem == "pcee2" && runtime.Pcee2Installed ? "today" : null, tested = new { match = "none" } };
            }
            yield return new { @event = "done", op = "core-ls", counts = new Dictionary<string, int>(), catalogue = "2026-10-31" };
        }

        // The listing, and an install that stops at 18 MB of 41 or runs to its end.
        private static Func<string, IToolJob> MockupJobs(MockupRuntime runtime, bool installEnds) => request =>
        {
            string op = JsonDocument.Parse(request).RootElement.GetProperty("op").GetString()!;
            if (op == "core-ls") return new ScriptedJob(MockupListing(runtime).ToList(), ends: true);
            var events = new List<object>
            {
                new { @event = "plan", stem = "pcee2", from = "https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcee2_libretro.so.zip", build = "2026-10-31", crc32 = "3a224a58" },
                new { @event = "checked", stem = "pcee2", check = "connection" },
                new { @event = "progress", stem = "pcee2", step = "download", bytes = 18_874_368, total = 42_991_616 },
            };
            if (installEnds)
            {
                runtime.Pcee2Installed = true;
                events.Add(new { @event = "installed", stem = "pcee2", revoke = Array.Empty<string>() });
                events.Add(new { @event = "done", op = "core-install", counts = new Dictionary<string, int>(), catalogue = "2026-10-31" });
            }
            return new ScriptedJob(events, ends: installEnds);
        };

        [WindowLookPngTheory]
        [MemberData(nameof(CoresMockupCases))]
        public Task Cores_is_pictured_in_its_mockups_states(string state, int width, int height) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver _) = InGame(width, height);
            Func<string, IToolJob> realStart = CoresPane.StartJob;
            Func<bool> realAvailable = CoresPane.ToolAvailable;
            Func<string, (long Free, long Total)?> realDisk = CoresPane.Disk;
            Func<CoresRuntime?, CoresRuntime?> realRuntime = CoresPane.RuntimeFor;
            try
            {
                // The list's mockup has PCEE2 installed and approved, the PlayStation 2's default; the others start before its install.
                var runtime = new MockupRuntime { Pcee2Installed = state == "list", Pcee2Trust = state == "list" ? "approved" : "needs_approval" };
                CoresPane.ToolAvailable = () => true;
                CoresPane.Disk = _ => (410L << 30, 1024L << 30);
                CoresPane.RuntimeFor = _ => runtime.Runtime;
                CoresPane.StartJob = MockupJobs(runtime, installEnds: state != "installing");
                Call(window, "ShowCores");
                Settle(window);
                var shown = (CoresWindow)Sheets(window).Current!;
                CoresPane pane = shown.Pane;
                Drain(window, pane);
                if (state == "list")
                {
                    pane.BigScreenTable.Select(pane.Rows.First(r => r.Name == "Sapphire"));
                    pane.FocusRole(AdaptiveRole.Content);
                }
                else
                {
                    for (int i = 0; i < 8 && pane.SelectedSection != "console:PlayStation 2"; i++)
                    {
                        Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.PageDown));
                        Settle(window);
                    }
                    Assert.Equal("console:PlayStation 2", pane.SelectedSection);
                    pane.BigScreenTable.Select(pane.Rows.First(r => r.Stem == "pcee2"));
                    pane.FocusRole(AdaptiveRole.Content);
                    Settle(window);
                    Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Accept));
                    Settle(window);
                    Assert.True(pane.IsPlanOpen);
                    Assert.True(pane.Confirm());
                    Drain(window, pane);
                    if (state == "approve")
                    {
                        Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Accept));
                        Settle(window);
                        Assert.True(pane.IsPlanOpen);
                    }
                }
                Settle(window);
                string dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "tmp", "cores-followup", "built");
                Directory.CreateDirectory(dir);
                UiTest.Capture(window).SavePng(Path.Combine(dir, $"cores-{state}-{width}x{height}.png"));
                Assert.Empty(Audit(window, $"CoresMockup-{state}-{width}x{height}", _out));
            }
            finally
            {
                CoresPane.StartJob = realStart;
                CoresPane.ToolAvailable = realAvailable;
                CoresPane.Disk = realDisk;
                CoresPane.RuntimeFor = realRuntime;
                Stop(window);
                window.Close();
            }
        }, default);

        // Every queued event taken, until the jobs run out or one waits for ever.
        private static void Drain(MainWindow window, CoresPane pane)
        {
            for (int i = 0; i < 8 && (i == 0 || pane.Busy); i++)
            {
                pane.Poll();
                Settle(window);
            }
        }
    }
}
