using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Packages;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Packages window framed in each of its states, fed a scripted listing so every run is the same: the listing, a game's detail page, Apply's summary and a plan running - see EmuSen_Packages.md §7.
    public partial class WindowFitAuditTests
    {
        // PackagesPlan and PackagesRunning, scripted below, join the audited states when the frame's plan sheet and apply bar fit the big screen (EmuSen_Packages.md §7.4).
        public static readonly string[] PackagesStates = ["Packages", "PackagesDetail"];

        private const string LongPackaged = "Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)";

        // A listing of every kind of row: linked and self-contained packages, a problem, and games not packaged, cartridges and discs.
        internal static IEnumerable<object> PackagesListing()
        {
            object[] rows =
            [
                new { @event = "package", index = 1, of = 3, package = $"SNES/{LongPackaged}.sfc", state = "ok", mode = "linked", system = "snes", console = "SNES", where = "library", game = $"snes/{LongPackaged}.sfc", size = 2_048, game_size = 1_048_576 },
                new { @event = "package", index = 2, of = 3, package = "NES/Cobalt Harbor (Synthetic).nes", state = "ok", mode = "self-contained", system = "nes", console = "NES", where = "package", game = "Cobalt Harbor (Synthetic).nes", size = 264_000, game_size = 262_160, patches = new[] { "Cobalt Harbor (Synthetic).ips" }, identity = "original" },
                new { @event = "package", index = 3, of = 3, package = "PlayStation/Ruby Lantern (Synthetic).cue", state = "changed", words = "psx/Ruby Lantern (Synthetic).cue is not the dump recorded (SHA-1 3f786850e387550fdab836ed7e6dc881de23001b)", mode = "linked", system = "psx", console = "PlayStation", where = "library", game = "psx/Ruby Lantern (Synthetic).cue", size = 3_100, game_size = 690_000_000 },
                new { @event = "game", index = 1, of = 3, source = "psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue", system = "PlayStation", state = "unpackaged", size = 690_000_000, convertible = true },
                new { @event = "game", index = 2, of = 3, source = "SCD/Ember Choir (USA).cue", system = "Sega CD", state = "unpackaged", size = 560_000_000, convertible = true },
                new { @event = "game", index = 3, of = 3, source = "snes/Amber Saga (USA).sfc", system = "SNES", state = "unpackaged", size = 2_097_152 },
            ];
            return rows.Prepend(new { @event = "start", op = "pkg-ls", packages = 3, games = 3, refused = Array.Empty<string>() })
                .Append(new { @event = "end", op = "pkg-ls", counts = new Dictionary<string, int> { ["ok"] = 2, ["changed"] = 1, ["unpackaged"] = 3 }, seconds = 0.2 });
        }

        // The window opened as Settings opens it, its listing taken, and for each later state the pad's steps taken to reach it.
        internal static void ShowPackages(MainWindow window, string state)
        {
            Func<string, IToolJob> realStart = PackagesPane.StartJob;
            Func<bool> realAvailable = PackagesPane.ToolAvailable;
            Func<string, (long Free, long Total)?> realDisk = PackagesPane.Disk;
            window.Closed += (_, _) =>
            {
                PackagesPane.StartJob = realStart;
                PackagesPane.ToolAvailable = realAvailable;
                PackagesPane.Disk = realDisk;
            };
            PackagesPane.ToolAvailable = () => true;
            PackagesPane.Disk = _ => (412L << 30, 1024L << 30);
            PackagesPane.StartJob = request => JsonDocument.Parse(request).RootElement.GetProperty("op").GetString() == "pkg-ls"
                ? new ScriptedJob(PackagesListing(), ends: true)
                : new ScriptedJob([new { @event = "start", op = "pkg-make", packages = 2, mode = "linked", zip = false, refused = Array.Empty<string>() }, new { @event = "package", index = 1, of = 2, source = "snes/Amber Saga (USA).sfc", system = "SNES", state = "made", package = "SNES/Amber Saga (USA).sfc", mode = "linked" }], ends: false);
            Call(window, "ShowPackages");
            Settle(window);
            var shown = (PackagesWindow)Sheets(window).Current!;
            PackagesPane pane = shown.Pane;
            pane.Poll();
            Settle(window);
            Assert.Equal(AdaptiveForm.BigScreen, pane.Form);
            Assert.Equal(6, pane.Rows.Count);
            pane.BigScreenTable.Select(pane.Rows.First(r => r.Title.StartsWith("Basalt", StringComparison.Ordinal)));
            pane.FocusRole(AdaptiveRole.Content);
            Settle(window);
            if (state == "PackagesDetail") Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Accept));
            if (state is "PackagesPlan" or "PackagesRunning")
            {
                Assert.True(pane.SetMark(pane.Rows.First(r => r.Title == "Amber Saga (USA)"), PackagesPlan.Make.Key));
                Assert.True(pane.SetMark(pane.Rows.First(r => r.Title.StartsWith("Basalt", StringComparison.Ordinal)), PackagesPlan.Convert.Key));
                Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Menu));
                Assert.True(pane.IsPlanOpen);
            }
            if (state == "PackagesRunning")
            {
                Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Menu));
                pane.Poll();
                Assert.True(pane.IsRunning);
            }
            Settle(window);
        }
    }
}
