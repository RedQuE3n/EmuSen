using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text.Json;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The Compressed Discs window in each of its states, fed a scripted job so that every state is the same in every run: no discs, a listing, a batch running, finished and failed - see EmuSen_GamePackages.md §13.
    public partial class WindowFitAuditTests
    {
        // A job that hands out the events it was given, and ends after them or never.
        internal sealed class ScriptedJob(IEnumerable<object> events, bool ends) : IToolJob
        {
            private readonly Queue<string> _events = new(events.Select(e => JsonSerializer.Serialize(e)));
            public bool Ended { get; private set; }
            public bool Stopped { get; private set; }

            public string? Next(int waitMs)
            {
                if (_events.TryDequeue(out string? line)) return line;
                if (ends) Ended = true;
                return null;
            }

            public void Stop() => Stopped = true;

            public void Dispose() { }
        }

        private static readonly FieldInfo StartJob = typeof(CompressedDiscsWindow).GetField("StartJob", BindingFlags.Static | BindingFlags.NonPublic)!;
        private static readonly FieldInfo Processors = typeof(CompressedDiscsWindow).GetField("Processors", BindingFlags.Static | BindingFlags.NonPublic)!;
        private static readonly FieldInfo ToolAvailable = typeof(CompressedDiscsWindow).GetField("ToolAvailable", BindingFlags.Static | BindingFlags.NonPublic)!;

        private const string LongDisc = "Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue";

        private static object Disc(string source, string system, string state, long size, long original = 0, long chd = 0) =>
            original > 0 ? new { @event = "disc", source, system, state, size, output = Path.ChangeExtension(source, ".chd"), original, chd } : new { @event = "disc", source, system, state, size };

        private static IEnumerable<object> Listing(bool empty)
        {
            object[] discs = empty ? [] :
            [
                Disc("psx/Amber Saga (USA)/Amber Saga (USA).cue", "PlayStation", "converted", 512_345_678, 512_000_000, 281_600_000),
                Disc("psx/" + LongDisc, "PlayStation", "not converted", 690_000_000),
                Disc("psx/Cobalt Harbor (Japan).bin", "PlayStation", "changed", 401_000_000),
                Disc("psx/Dusk Lantern (USA).zip", "PlayStation", "missing", 300_100_000),
                Disc("SCD/Ember Choir (USA).cue", "Sega CD", "not converted", 560_000_000),
            ];
            return discs.Prepend(new { @event = "start", op = "ls", discs = discs.Length, refused = Array.Empty<string>() })
                .Append(new { @event = "end", op = "ls", counts = new Dictionary<string, int>(), seconds = 0.1 });
        }

        private static object Passed => new { chdman = "passed", tracks = "passed", toc = "passed" };

        private static object Worked(int index, string source, string system, string state, object? more = null) => more switch
        {
            null => new { @event = "disc", index, of = 3, source, system, state },
            _ => MergeEvent(new { @event = "disc", index, of = 3, source, system, state }, more),
        };

        private static Dictionary<string, object?> MergeEvent(object head, object more)
        {
            var merged = JsonSerializer.Deserialize<Dictionary<string, object?>>(JsonSerializer.Serialize(head))!;
            foreach (var (k, v) in JsonSerializer.Deserialize<Dictionary<string, object?>>(JsonSerializer.Serialize(more))!) merged[k] = v;
            return merged;
        }

        // The conversion's events, of the three discs the listing has not converted, to the state named.
        private static IEnumerable<object> Batch(string state)
        {
            string log = Path.Combine(DataStore.Logs, "chd_2026-10-25.log");
            yield return new { @event = "start", op = "convert", discs = 3, chdman = "0.289 (unknown)", codecs = "cdlz,cdzl,cdfl", threads = 8, output = DataStore.Compressed, log, refused = Array.Empty<string>() };
            string basalt = "psx/" + LongDisc, cobalt = "psx/Cobalt Harbor (Japan).bin", ember = "SCD/Ember Choir (USA).cue";
            yield return Worked(1, basalt, "PlayStation", "working");
            if (state == "Failed")
            {
                yield return Worked(1, basalt, "PlayStation", "failed", new { words = "a check failed", checks = new { chdman = "passed", tracks = "failed: track 2 rebuilt from the CHD has SHA-1 3f786850e387550fdab836ed7e6dc881de23001b and the original 89e6c98d92887913cadf06b2adb97f26cde4849b", toc = "passed" } });
                yield return Worked(2, cobalt, "PlayStation", "working");
                yield return Worked(2, cobalt, "PlayStation", "skipped", new { words = "the original does not read as a disc: track 1 is shorter than its pregap" });
                yield return Worked(3, ember, "Sega CD", "working");
                yield return Worked(3, ember, "Sega CD", "failed", new { words = $"1048576 bytes are free at {Path.Combine(DataStore.Compressed, "SCD")}, and the disc needs 627108864 with the margin" });
                yield return new { @event = "end", op = "convert", counts = new Dictionary<string, int> { ["failed"] = 2, ["skipped"] = 1 }, seconds = 31.4 };
                yield break;
            }
            yield return Worked(1, basalt, "PlayStation", "converted", new { output = Path.ChangeExtension(basalt, ".chd"), package = "PlayStation/" + Path.ChangeExtension(LongDisc, ".chd"), original = 690_000_000, chd = 389_000_000, seconds = 25.3, checks = Passed });
            yield return Worked(2, cobalt, "PlayStation", "working");
            if (state == "Running") yield break;
            yield return Worked(2, cobalt, "PlayStation", "converted", new { original = 401_000_000, chd = 226_000_000, seconds = 18.9, checks = Passed });
            yield return Worked(3, ember, "Sega CD", "working");
            yield return Worked(3, ember, "Sega CD", "converted", new { original = 560_000_000, chd = 279_000_000, seconds = 20.2, checks = Passed });
            yield return new { @event = "end", op = "convert", counts = new Dictionary<string, int> { ["converted"] = 3 }, seconds = 64.4 };
        }

        // The window opened as Settings opens it, its listing taken, and for a batch's state Convert pressed on the three discs not converted.
        internal static void ShowCompressedDiscs(MainWindow window, string state)
        {
            object realStart = StartJob.GetValue(null)!, realProcessors = Processors.GetValue(null)!, realAvailable = ToolAvailable.GetValue(null)!;
            window.Closed += (_, _) =>
            {
                StartJob.SetValue(null, realStart);
                Processors.SetValue(null, realProcessors);
                ToolAvailable.SetValue(null, realAvailable);
            };
            Processors.SetValue(null, (Func<int>)(() => 16));
            ToolAvailable.SetValue(null, (Func<bool>)(() => true));
            StartJob.SetValue(null, (Func<string, IToolJob>)(request =>
            {
                string op = JsonDocument.Parse(request).RootElement.GetProperty("op").GetString()!;
                return op == "ls" ? new ScriptedJob(Listing(empty: state == "Empty"), ends: true) : new ScriptedJob(Batch(state), ends: state != "Running");
            }));
            Call(window, "ShowCompressedDiscs");
            Settle(window);
            var shown = (CompressedDiscsWindow)Sheets(window).Current!;
            shown.Poll();
            Settle(window);
            if (state is "Running" or "Finished" or "Failed")
            {
                foreach (int row in new[] { 1, 2, 4 })
                    Sheets(window).SheetOf(shown)!.GetVisualDescendants().OfType<CheckBox>().First(c => c.Name == $"CompressedDiscs.Disc.{row}").IsChecked = true;
                Sheets(window).SheetOf(shown)!.GetVisualDescendants().OfType<Button>().First(b => b.Name == "CompressedDiscsConvert").RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
                shown.Poll();
                Settle(window);
            }
            if (state == "Beside")
            {
                Sheets(window).SheetOf(shown)!.GetVisualDescendants().OfType<EmuSen.LunaP.Controls.Dropdown>().First(d => d.Name == "CompressedDiscsWhere").SelectedItem = CompressedDiscsWindow.BesideOriginals;
                Settle(window);
                Assert.True(shown.Beside);
            }
            Assert.Equal(state == "Empty" ? 0 : 5, shown.Discs.Count);
            Assert.Equal(state == "Running", shown.Busy);
        }
    }
}
