using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using Avalonia.Controls;
using Avalonia.Input;
using EmuSen.Mistress.Input;
using Xunit;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Fixtures
{
    // Where the recorded specification lives, how a state is recorded or held to its recording, and the presses each state answers - see EmuSen_InterfaceSpec.md §5.
    public static class InterfaceSpecFiles
    {
        private static string Here([CallerFilePath] string path = "") => Path.GetDirectoryName(path)!;

        public static string Folder => Path.GetFullPath(Path.Combine(Here(), "..", "Mistress", "Specification"));

        // Where a recording is written: the folder above, or EMUSEN_INTERFACE_SPEC_DIR's, so that two recordings can be compared with each other.
        public static string WriteFolder => Environment.GetEnvironmentVariable("EMUSEN_INTERFACE_SPEC_DIR") is { Length: > 0 } d ? d : Folder;

        public const string Variable = "EMUSEN_INTERFACE_SPEC";

        public static bool Recording => Environment.GetEnvironmentVariable(Variable) == "record";

        // The grammar's presses a state is asked, in order, after its walk: pages, ends, then back.
        public static readonly UiButton[] Grammar = [UiButton.PageUp, UiButton.PageDown, UiButton.First, UiButton.Last, UiButton.Back];

        // Text that names a run's own folders, written as the folder's part: the temporary folders and the home directory.
        public static Func<string, string> Normaliser(params string[] roots)
        {
            string temp = Path.GetTempPath().TrimEnd('/'), home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            return s =>
            {
                foreach (string root in roots.Where(r => r.Length > 0)) s = s.Replace(root, "<run>", StringComparison.Ordinal);
                return s.Replace(temp, "<temp>", StringComparison.Ordinal).Replace(home, "<home>", StringComparison.Ordinal);
            };
        }

        // Presses each button and records what it reached: where the focus is, and what observe reports of the window.
        public static void Presses(SpecDocument doc, PadDriver pad, TopLevel top, Func<SortedDictionary<string, string>> observe, params UiButton[] buttons)
        {
            foreach (UiButton b in buttons)
            {
                pad.Press(b);
                SortedDictionary<string, string> seen = observe();
                if (top.FocusManager?.GetFocusedElement() is InputElement focused)
                    seen["focus"] = InterfaceRecorder.Normalise(PadAudit.Describe(focused));
                doc.Presses.Add(new SpecPress(b.ToString().ToLowerInvariant(), seen));
            }
        }

        // The recording written, or the state held to it byte for byte; then the rules over it, less the faults recorded as known, returned for the caller to assert once every page of a state is recorded.
        public static List<string> Check(SpecDocument doc, ITestOutputHelper output, IReadOnlyDictionary<string, string>? known = null)
        {
            string text = doc.Write();
            string path = Path.Combine(Folder, doc.State + ".spec");
            if (Recording)
            {
                Directory.CreateDirectory(WriteFolder);
                File.WriteAllText(Path.Combine(WriteFolder, doc.State + ".spec"), text);
            }
            else
            {
                Assert.True(File.Exists(path), $"{doc.State} has no recording; {Variable}=record writes it");
                string held = File.ReadAllText(path);
                if (held != text)
                {
                    string[] a = held.Split('\n'), b = text.Split('\n');
                    int at = Enumerable.Range(0, Math.Min(a.Length, b.Length)).FirstOrDefault(i => a[i] != b[i], Math.Min(a.Length, b.Length));
                    Assert.Fail($"{doc.State} differs from its recording at line {at + 1}:\n  recorded: {(at < a.Length ? a[at] : "(end)")}\n  now:      {(at < b.Length ? b[at] : "(end)")}");
                }
            }
            List<string> faults = InterfaceRules.All(doc).Where(f => known is null || !known.ContainsKey(f)).ToList();
            foreach (string f in faults) output.WriteLine($"{doc.State}: {f}");
            return faults.Select(f => $"{doc.State}: {f}").ToList();
        }
    }
}
