using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.Galaxia.Media;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Library;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Library Manager framed in each of its states, fed scripted reads and records so every run is the same: the findings, a file's detail page, the plan and a read going - see EmuSen_LibraryTools.md §10.
    public partial class WindowFitAuditTests
    {
        public static readonly string[] LibraryStates = ["Library", "LibraryDetail", "LibraryPlan", "LibraryRunning"];

        private const string LongDump = "Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)";

        // games.db and media.db as a table: what is hidden, and what ScreenScraper answered of each file.
        internal sealed class ScriptedRecords(string library) : ILibraryRecords
        {
            public readonly HashSet<string> HiddenPaths = new(StringComparer.Ordinal);
            public readonly List<(string Path, bool Hide)> Changes = new();

            public IReadOnlySet<string> Hidden() => HiddenPaths.ToHashSet(StringComparer.Ordinal);

            public void SetHidden(string path, bool hidden)
            {
                Changes.Add((path, hidden));
                if (hidden) HiddenPaths.Add(path);
                else HiddenPaths.Remove(path);
            }

            public IReadOnlyList<ScrapedFile> Scraped() =>
            [
                new($"{library}/snes/Amber Saga (USA).sfc", true, 4120, "aa11", 1_048_576, "Amber Saga"),
                new($"{library}/snes/Amber Saga (Europe) (Rev 1).sfc", true, 4120, "bb22", 1_048_576, "Amber Saga"),
                new($"{library}/nes/Granite Choir (Translated En).nes", false, null, "", 262_160),
            ];
        }

        // `rom check -r` over the library: two failed files among four.
        internal static IEnumerable<object> LibraryCheck(bool ends = true)
        {
            object[] files =
            [
                new { @event = "file", index = 1, of = 4, source = $"psx/{LongDump}.cue", system = "psx", state = "failed", failed = new[] { new { check = "sheet", words = $"it lists {LongDump} (Track 2).bin, which is not there" } } },
                new { @event = "file", index = 2, of = 4, source = "snes/Amber Saga (USA).sfc", system = "snes", state = "ok", failed = Array.Empty<object>() },
                new { @event = "file", index = 3, of = 4, source = "snes/Amber Saga (USA) [copier].smc", system = "snes", state = "ok", failed = Array.Empty<object>() },
                new { @event = "file", index = 4, of = 4, source = "snes/Cobalt Harbor (Empty).sfc", system = "snes", state = "failed", failed = new[] { new { check = "size", words = "Cobalt Harbor (Empty).sfc is empty" } } },
            ];
            IEnumerable<object> events = files.Prepend(new { @event = "start", op = "rom-check", files = 4, refused = Array.Empty<string>() });
            return ends ? events.Append(new { @event = "end", op = "rom-check", counts = new Dictionary<string, int> { ["ok"] = 2, ["failed"] = 2 }, seconds = 0.3 }) : events.Take(3);
        }

        // `rom dups`: the headered and plain copies of one dump.
        internal static IEnumerable<object> LibraryDups() =>
        [
            new { @event = "start", op = "rom-dups", files = 4, refused = Array.Empty<string>() },
            new { @event = "group", members = new[] { new { crc32 = "1a2b3c4d", md5 = "aa11", sha1 = "", size = 1_048_576, name = "snes/Amber Saga (USA).sfc" }, new { crc32 = "1a2b3c4d", md5 = "aa11", sha1 = "", size = 1_048_576, name = "snes/Amber Saga (USA) [copier].smc" } } },
            new { @event = "end", op = "rom-dups", counts = new Dictionary<string, int> { ["hashed"] = 4, ["groups"] = 1 }, seconds = 0.4 },
        ];

        // The pane's seams set to scripted reads, put back when the window closes.
        internal static void ScriptLibrary(Window window, bool running = false)
        {
            Func<string, IToolJob> realStart = LibraryPane.StartJob;
            Func<bool> realAvailable = LibraryPane.ToolAvailable;
            Func<string, bool> realExists = LibraryPane.Exists;
            Func<PackageIndex?> realClaims = LibraryPane.Claims;
            window.Closed += (_, _) =>
            {
                LibraryPane.StartJob = realStart;
                LibraryPane.ToolAvailable = realAvailable;
                LibraryPane.Exists = realExists;
                LibraryPane.Claims = realClaims;
            };
            LibraryPane.ToolAvailable = () => true;
            LibraryPane.Exists = _ => true;
            LibraryPane.Claims = () => null;
            LibraryPane.StartJob = request => JsonDocument.Parse(request).RootElement.GetProperty("op").GetString() == "rom-check"
                ? new ScriptedJob(LibraryCheck(!running), ends: !running)
                : new ScriptedJob(LibraryDups(), ends: true);
        }

        // The window opened as the pad menu opens it, its reads taken with scripted records, and for each later state the pad's steps taken to reach it.
        internal static void ShowLibraryManager(MainWindow window, string state)
        {
            ScriptLibrary(window, running: state == "LibraryRunning");
            Call(window, "ShowLibraryManager");
            Settle(window);
            var shown = (LibraryWindow)Sheets(window).Current!;
            LibraryPane pane = shown.Pane;
            Assert.Equal(AdaptiveForm.BigScreen, pane.Form);
            pane.Records = new ScriptedRecords(pane.Library!.TrimEnd('/'));
            for (int i = 0; i < 4 && pane.Busy; i++) pane.Poll();
            if (state == "LibraryRunning")
            {
                Assert.True(pane.Busy);
                Settle(window);
                return;
            }
            pane.Refresh();
            for (int i = 0; i < 4 && pane.Busy; i++) pane.Poll();
            Settle(window);
            Assert.True(pane.Complete);
            Assert.Equal(7, pane.Rows.Count);
            pane.BigScreenTable.Select(pane.Rows.First(r => r.Title == LongDump));
            pane.FocusRole(AdaptiveRole.Content);
            Settle(window);
            // LibraryDetail: the chosen file's detail stands beside the list, so no page is opened; A would ask to hide it.
            if (state == "LibraryDetail") Assert.Equal("Hide…", pane.Hints[0].Label);
            if (state == "LibraryPlan")
            {
                Assert.True(pane.SetMark(pane.Rows.First(r => r.Kind == Finding.SameDump && r.Source.EndsWith(".smc", StringComparison.Ordinal)), LibraryFindings.Hide.Key));
                Assert.True(pane.SetMark(pane.Rows.First(r => r.Title == LongDump), LibraryFindings.Hide.Key));
                Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Menu));
                Assert.True(pane.IsPlanOpen);
            }
            Settle(window);
        }
    }
}
