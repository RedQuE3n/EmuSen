using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.Tools.Scraper;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Scraper window framed in each of its states, fed a scripted command so every run is the same: the listing, a game's detail page, the plan and a run going - see EmuSen_LibraryTools.md §9.
    public partial class WindowFitAuditTests
    {
        public static readonly string[] ScraperStates = ["Scraper", "ScraperDetail", "ScraperPlan", "ScraperRunning"];

        // The scrape command answered from tables: every request kept, a run going once one is started.
        internal sealed class ScriptedScrape
        {
            public readonly List<string> Asked = new();
            public bool Running;
            public Func<string, string?>? Refuse;

            public const string LongTitle = "Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)";

            public static readonly (string Path, string System, string?[] Sources)[] Games =
            [
                ($"/r/psx/{LongTitle}.cue", "psx", ["screenscraper", null, null, "package"]),
                ("/r/snes/Amber Saga (USA).sfc", "snes", ["hand", "esde", "screenscraper", "screenscraper"]),
                ("/r/nes/Cobalt Harbor (Synthetic).nes", "nes", [null, null, null, null]),
            ];

            public string Ask(string request)
            {
                Asked.Add(request);
                if (Refuse?.Invoke(request) is { } words) throw new ScrapeCommandException(ScrapeCommand.StoreInUse, words);
                JsonElement r = JsonDocument.Parse(request).RootElement;
                string sub = r.GetProperty("args")[0].GetString()!;
                // The module's default since 2026-11-02: every picture a theme can draw (EmuSen_LibraryTools.md §9.7).
                string[] kinds = ["cover", "screenshot", "marquee", "titlescreen", "miximage", "backcover", "3dbox", "physicalmedia", "fanart"];
                object answer = sub switch
                {
                    "ls" => new
                    {
                        games = Games.Where(g => !r.GetProperty("args").EnumerateArray().Any(a => a.GetString() == "--missing") || g.Sources.Contains(null)).Select(g => new { path = g.Path, system = g.System, package = (string?)null, media = kinds.Select((k, i) => (k, i < g.Sources.Length ? g.Sources[i] : null)).ToDictionary(x => x.k, x => x.Item2) }),
                        kinds = Games.ToDictionary(g => g.System, _ => kinds),
                        notes = new[] { "1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it" },
                    },
                    "collections" => Array.Empty<object>(),
                    "status" => new
                    {
                        quota = new { requests = 120, max_per_day = 20000, threads = 1, per_minute = 60, gate = "open" },
                        account = new { signed_in = false },
                        developer = new { origin = "embedded" },
                        progress = new { state = "idle", serial = 0 },
                    },
                    "plan" => new
                    {
                        plan = new { games = 2 },
                        kinds,
                        said = "2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min",
                        notes = Array.Empty<string>(),
                    },
                    "run" => Start(),
                    "progress" => Running
                        ? new { progress = (object)new { state = "running", serial = 3, total = 2, done = 1, current = new { path = Games[0].Path }, results = new[] { new { serial = 3, path = Games[1].Path, outcome = "found", skipped = false } } }, said = (string?)null }
                        : new { progress = (object)new { state = "done", serial = 4, total = 2, done = 2, results = new[] { new { serial = 4, path = Games[0].Path, outcome = "unknown", skipped = false } } }, said = (string?)"2 of 2 game(s) done: 1 found, 1 unknown, 0 failed, 0 skipped, 0 unchanged; 6 request(s)" },
                    "cancel" => new { cancelled = Running },
                    _ => throw new ScrapeCommandException(-1163, $"not scripted: {sub}"),
                };
                return JsonSerializer.Serialize(answer);
            }

            private object Start()
            {
                Running = true;
                return new { queued = 2, since = 2, kinds = new[] { "cover" }, notes = Array.Empty<string>() };
            }

            // The scripted command put in the pane's place, and the real one put back when the window closes.
            public static ScriptedScrape Install(Avalonia.Controls.Window owner)
            {
                var script = new ScriptedScrape();
                Func<string, string> ask = ScraperPane.Ask;
                Func<bool> available = ScraperPane.CommandAvailable;
                Func<Func<string>, Task<string>> background = ScraperPane.Background;
                owner.Closed += (_, _) => Restore(ask, available, background);
                ScraperPane.Ask = script.Ask;
                ScraperPane.CommandAvailable = () => true;
                ScraperPane.Background = work => Task.FromResult(work());
                return script;
            }

            public static void Restore(Func<string, string> ask, Func<bool> available, Func<Func<string>, Task<string>> background)
            {
                ScraperPane.Ask = ask;
                ScraperPane.CommandAvailable = available;
                ScraperPane.Background = background;
            }
        }

        // The window opened as Settings opens it, its listing taken, and for each later state the pad's steps taken to reach it.
        internal static void ShowScraper(MainWindow window, string state)
        {
            ScriptedScrape script = ScriptedScrape.Install(window);
            Call(window, "ShowScraper");
            Settle(window);
            var shown = (ScraperWindow)Sheets(window).Current!;
            ScraperPane pane = shown.Pane;
            Settle(window);
            Assert.Equal(AdaptiveForm.BigScreen, pane.Form);
            Assert.Equal(3, pane.Rows.Count);
            pane.BigScreenTable.Select(pane.Rows.First(r => r.Title == ScriptedScrape.LongTitle));
            pane.FocusRole(AdaptiveRole.Content);
            Settle(window);
            // ScraperDetail: the chosen game's media stand beside the list, so A has no page to open.
            if (state == "ScraperDetail") Assert.False(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Accept));
            if (state is "ScraperPlan" or "ScraperRunning")
            {
                Assert.True(pane.SetMark(pane.Rows.First(r => r.Title == "Amber Saga (USA)"), ScraperPlan.Scrape.Key));
                Assert.True(pane.SetMark(pane.Rows.First(r => r.Title == ScriptedScrape.LongTitle), ScraperPlan.Scrape.Key));
                Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Menu));
                Assert.True(pane.IsPlanOpen);
            }
            if (state == "ScraperRunning")
            {
                Assert.True(shown.OnPad(global::EmuSen.Mistress.Input.UiButton.Menu));
                pane.Poll();
                Assert.True(pane.Following);
            }
            Settle(window);
        }
    }
}
