using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.LunaP.Controls;

namespace EmuSen.Tools.Packages
{
    // One job of a confirmed plan: the request it starts, what it says while it runs, and the rows it was asked for.
    public sealed record PlanStep(string Mark, string Request, string Doing, IReadOnlyList<PackageRow> Rows);

    // The changes a row is offered, and the marked rows turned into the library tool's requests and the words of Apply's summary - see EmuSen_Packages.md §3, §3a and §7.
    public static class PackagesPlan
    {
        public static readonly ManagerMark Make = new("make", "make, linked");
        public static readonly ManagerMark MakeSelfContained = new("make-self", "make self-contained");
        public static readonly ManagerMark MakeZip = new("make-zip", "make and zip");
        public static readonly ManagerMark Zip = new("zip", "zip");
        public static readonly ManagerMark Check = new("check", "check");
        public static readonly ManagerMark Convert = new("convert", "convert to CHD");

        // The order the steps run in, whatever order the rows were marked in.
        public static readonly IReadOnlyList<ManagerMark> Order = [Make, MakeSelfContained, MakeZip, Zip, Convert, Check];

        // What a row is offered: a game not yet packaged is made, a package checked first where it has a problem, a disc converted, a linked package made self-contained or zipped, a self-contained one zipped.
        public static IReadOnlyList<ManagerMark> MarksFor(PackageRow row)
        {
            var marks = new List<ManagerMark>();
            if (!row.Packaged)
            {
                if (row.Source is not null) marks.AddRange([Make, MakeSelfContained, MakeZip]);
                if (row.Convertible && row.Source is not null) marks.Add(Convert);
                return marks;
            }
            marks.Add(Check);
            if (row.Problem) return marks;
            if (row.Convertible && row.Source is not null) marks.Add(Convert);
            if (row.Mode == "linked" && row.Source is not null) marks.AddRange([MakeSelfContained, MakeZip]);
            if (row.Mode == "self-contained") marks.Add(Zip);
            return marks;
        }

        // The marked rows as the jobs that do them, one request a kind of change, in Order.
        public static IReadOnlyList<PlanStep> Steps(IReadOnlyList<ManagerMarked<PackageRow>> marked)
        {
            var steps = new List<PlanStep>();
            foreach (ManagerMark mark in Order)
            {
                List<PackageRow> rows = marked.Where(m => m.Mark.Key == mark.Key).Select(m => m.Row).ToList();
                if (rows.Count == 0) continue;
                string[] sources = rows.Select(r => r.Source!).ToArray();
                string[] packages = rows.Select(r => r.Package!).ToArray();
                (string request, string doing) = mark.Key switch
                {
                    "make" => (JsonSerializer.Serialize(new { op = "pkg-make", paths = sources }), "Making packages"),
                    "make-self" => (JsonSerializer.Serialize(new { op = "pkg-make", paths = sources, self_contained = true }), "Making packages self-contained"),
                    "make-zip" => (JsonSerializer.Serialize(new { op = "pkg-make", paths = sources, zip = true }), "Making packages and their zips"),
                    "zip" => (JsonSerializer.Serialize(new { op = "pkg-zip", paths = packages }), "Zipping packages"),
                    "convert" => (JsonSerializer.Serialize(new { op = "convert", paths = sources }), "Converting discs to CHD"),
                    _ => (JsonSerializer.Serialize(new { op = "pkg-check", paths = packages }), "Checking packages"),
                };
                steps.Add(new PlanStep(mark.Key, request, doing, rows));
            }
            return steps;
        }

        private static string Count(int n, string one, string many) => $"{n} {(n == 1 ? one : many)}";

        // The apply bar's line: what is marked, by kind.
        public static string Summary(IReadOnlyList<ManagerMarked<PackageRow>> marked)
        {
            if (marked.Count == 0) return "Nothing marked.";
            IEnumerable<string> kinds = Order.Select(m => (m, n: marked.Count(x => x.Mark.Key == m.Key))).Where(p => p.n > 0).Select(p => $"{p.m.Label} {p.n}");
            return $"{Count(marked.Count, "marked", "marked")}: {string.Join(", ", kinds)}";
        }

        // Apply's summary: what each step will do and to which games, said before anything runs; nothing in the library is written by any of it.
        public static string Text(IReadOnlyList<ManagerMarked<PackageRow>> marked)
        {
            var text = new StringBuilder();
            foreach (PlanStep step in Steps(marked))
            {
                int n = step.Rows.Count;
                text.AppendLine(step.Mark switch
                {
                    "make" => $"Make {Count(n, "package", "packages")}, linked: each game stays where it is in the library.",
                    "make-self" => $"Make {Count(n, "package", "packages")} self-contained: each game is copied into its package, and the library's copy is left as it is.",
                    "make-zip" => $"Make {Count(n, "package", "packages")} self-contained and write each as one zip beside its folder.",
                    "zip" => $"Zip {Count(n, "self-contained package", "self-contained packages")}, each as one zip beside its folder; a zip already there is never replaced.",
                    "convert" => $"Convert {Count(n, "disc", "discs")} to CHD, each checked three ways once it is written and given a package. The originals are never removed.",
                    _ => $"Check {Count(n, "package", "packages")}: the game and every file hashed against the manifest.",
                });
                foreach (PackageRow row in step.Rows.Take(8)) text.AppendLine($"    {row.Title} ({row.Console})");
                if (n > 8) text.AppendLine($"    and {n - 8} more");
            }
            text.Append("Nothing in the library is changed, moved or removed.");
            return text.ToString();
        }
    }
}
