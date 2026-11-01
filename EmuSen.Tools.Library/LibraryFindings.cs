using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Galaxia.Native;
using EmuSen.LunaP.Controls;
using static EmuSen.Galaxia.Native.ToolEvents;

namespace EmuSen.Tools.Library
{
    // One member of a `rom dups` group: its path in the library, its size, and the package that claims it.
    public sealed record DumpMember(string Source, long Size, string? Package);

    // One file `rom check` failed, with its system and each check it failed.
    public sealed record CheckedFile(string Source, string Console, long Size, IReadOnlyList<FailedCheck> Failed);

    // The library manager's findings from the library tool's events and the host's records, its marks, plan and saved list, free of any control - see EmuSen_LibraryTools.md §10.
    public static class LibraryFindings
    {
        public static readonly ManagerMark Hide = new("hide", "hide");
        public static readonly ManagerMark Unhide = new("unhide", "show again");

        public const string CheckRequest = """{"op":"rom-check","paths":[""],"recursive":true}""";
        public const string DupsRequest = """{"op":"rom-dups","console":null}""";

        // A check's name as the window says it; "header" is T3's, named here so its findings need no new words.
        public static string CheckName(string check) => check switch
        {
            "size" => "Size",
            "sheet" => "Disc sheet",
            "archive" => "Archive",
            "chd" => "CHD",
            "header" => "Header checksum",
            _ => check,
        };

        public static string KindName(Finding kind) => kind switch
        {
            Finding.SameDump => "Same dump",
            Finding.SameGame => "Same game",
            Finding.Broken => "Broken or suspect",
            _ => "Unrecognised",
        };

        // The group a `rom dups` event names, null for any other event.
        public static IReadOnlyList<DumpMember>? Group(JsonElement e) =>
            Text(e, "event") == "group" && e.TryGetProperty("members", out JsonElement m) && m.ValueKind == JsonValueKind.Array
                ? m.EnumerateArray().Select(x => new DumpMember(Text(x, "name") ?? "", Number(x, "size"), Text(x, "package"))).Where(x => x.Source.Length > 0).ToList()
                : null;

        // The failed file a `rom check` event names, null for a file that passed and any other event.
        public static CheckedFile? Failed(JsonElement e)
        {
            if (Text(e, "event") != "file" || Text(e, "state") != "failed" || Text(e, "source") is not { } source) return null;
            List<FailedCheck> failed = e.TryGetProperty("failed", out JsonElement f) && f.ValueKind == JsonValueKind.Array
                ? f.EnumerateArray().Select(x => new FailedCheck(Text(x, "check") ?? "", Text(x, "words") ?? "")).ToList()
                : [];
            return new CheckedFile(source, Text(e, "system") ?? "", 0, failed);
        }

        // The rows of every finding: each dump group with a package and the files it claims counted once, each game ScreenScraper knows in more than one dump, each failed file, and each file it did not recognise.
        public static List<LibraryRow> Rows(string library, IReadOnlyList<IReadOnlyList<DumpMember>> dumps, IReadOnlyList<CheckedFile> broken,
            IReadOnlyList<ScrapedFile> scraped, IReadOnlySet<string> hidden, IReadOnlyDictionary<string, string> consoles, Func<string, bool> exists, Func<string, string?> claimant)
        {
            var rows = new List<LibraryRow>();
            var dumpOf = new Dictionary<string, int>(StringComparer.Ordinal);
            int group = 0;
            foreach (IReadOnlyList<DumpMember> members in dumps)
            {
                List<DumpMember> games = members.GroupBy(m => m.Package ?? "file:" + m.Source).Select(g => g.First()).ToList();
                if (games.Count < 2) continue;
                group++;
                foreach (DumpMember m in members) dumpOf[m.Source] = group;
                foreach (DumpMember m in games)
                    rows.Add(Row(library, Finding.SameDump, m.Source, hidden, consoles) with { Group = group, Package = m.Package, Size = m.Size, Others = games.Where(o => o != m).Select(o => o.Source).ToList() });
            }

            // One dump: the files a package claims, else a group of the same dump, else ScreenScraper's hashes.
            string DumpOf(ScrapedFile f, string source) =>
                claimant(f.Path) is { } package ? "package:" + package : dumpOf.TryGetValue(source, out int d) ? $"dump:{d}" : $"{f.Md5}:{f.Bytes}";

            var known = scraped.Select(s => (File: s, Source: Relative(library, s.Path))).Where(s => s.Source is not null && exists(s.File.Path)).ToList();
            int game = 0;
            foreach (var same in known.Where(s => s.File.Recognised && s.File.GameId is not null).GroupBy(s => s.File.GameId!.Value).OrderBy(g => g.Key))
            {
                var files = same.OrderBy(s => s.Source, StringComparer.Ordinal).GroupBy(s => claimant(s.File.Path) is { } p ? "package:" + p : "file:" + s.Source).Select(g => g.First()).ToList();
                if (files.Select(s => DumpOf(s.File, s.Source!)).Distinct().Count() < 2) continue;
                game++;
                foreach (var s in files)
                    rows.Add(Row(library, Finding.SameGame, s.Source!, hidden, consoles) with
                    {
                        Group = game, GameId = same.Key, GameName = s.File.Name, Size = s.File.Bytes, Package = claimant(s.File.Path), Others = files.Where(o => o.Source != s.Source).Select(o => o.Source!).ToList(),
                    });
            }

            foreach (CheckedFile f in broken)
                rows.Add(Row(library, Finding.Broken, f.Source, hidden, consoles) with { Failed = f.Failed, Console = f.Console.Length > 0 ? f.Console : consoles.GetValueOrDefault(f.Source, "") });

            foreach (var s in known.Where(s => !s.File.Recognised).OrderBy(s => s.Source, StringComparer.Ordinal))
                rows.Add(Row(library, Finding.Unrecognised, s.Source!, hidden, consoles) with { Size = s.File.Bytes });
            return rows;
        }

        private static LibraryRow Row(string library, Finding kind, string source, IReadOnlySet<string> hidden, IReadOnlyDictionary<string, string> consoles)
        {
            string full = Full(library, source);
            return new LibraryRow { Kind = kind, Source = source, FullPath = full, Title = Path.GetFileNameWithoutExtension(source), Console = consoles.GetValueOrDefault(source, ""), Hidden = hidden.Contains(full) };
        }

        // A path in the library as games.db names it.
        public static string Full(string library, string source) => Path.GetFullPath(Path.Combine(library, source));

        // A full path's place in the library with / between its parts; null for a path outside it.
        public static string? Relative(string library, string path)
        {
            string rel = Path.GetRelativePath(Path.GetFullPath(library), Path.GetFullPath(path));
            return rel == "." || rel.StartsWith("..", StringComparison.Ordinal) || Path.IsPathRooted(rel) ? null : rel.Replace(Path.DirectorySeparatorChar, '/');
        }

        // What a row says in the Finding column.
        public static string Words(LibraryRow row) => row.Kind switch
        {
            Finding.SameDump => row.Others.Count == 1 ? "Same dump as 1 other file" : $"Same dump as {row.Others.Count} other files",
            Finding.SameGame => (row.Others.Count == 1 ? "Same game as 1 other file" : $"Same game as {row.Others.Count} other files") + (row.GameName is { Length: > 0 } n ? $" ({n})" : ""),
            Finding.Broken => row.Failed.Count > 0 ? $"{CheckName(row.Failed[0].Check)}: {row.Failed[0].Words}" : "A check failed",
            _ => "Not recognised by ScreenScraper (a hint, not a fault)",
        };

        public static string GroupText(LibraryRow row) => row.Group == 0 ? "" : row.Kind == Finding.SameDump ? $"Dump {row.Group}" : $"Game {row.Group}";

        // What a row is offered: hidden from the library, or shown again where it is hidden; nothing where no host lends games.db.
        public static IReadOnlyList<ManagerMark> MarksFor(LibraryRow row, bool canHide) => !canHide ? [] : row.Hidden ? [Unhide] : [Hide];

        // The marked games, each file once, with whether it is to be hidden.
        public static IReadOnlyList<(string Path, bool Hide)> Changes(IReadOnlyList<ManagerMarked<LibraryRow>> marked) =>
            marked.GroupBy(m => m.Row.FullPath, StringComparer.Ordinal).Select(g => (g.Key, g.First().Mark.Key == Hide.Key)).ToList();

        public static string Summary(IReadOnlyList<ManagerMarked<LibraryRow>> marked)
        {
            var changes = Changes(marked);
            int hide = changes.Count(c => c.Hide), show = changes.Count - hide;
            var parts = new List<string>();
            if (hide > 0) parts.Add($"{hide} to hide");
            if (show > 0) parts.Add($"{show} to show again");
            return string.Join(", ", parts);
        }

        // The plan before anything changes: each game by name, and that only games.db changes.
        public static string Plan(IReadOnlyList<ManagerMarked<LibraryRow>> marked)
        {
            var changes = Changes(marked);
            var text = new StringBuilder();
            foreach ((bool hide, string heading) in new[] { (true, "Hide from the library"), (false, "Show in the library again") })
            {
                var paths = changes.Where(c => c.Hide == hide).Select(c => c.Path).ToList();
                if (paths.Count == 0) continue;
                text.Append(heading).Append(" (").Append(paths.Count).AppendLine("):");
                foreach (string p in paths) text.Append("  ").AppendLine(Path.GetFileName(p));
                text.AppendLine();
            }
            text.Append("Only games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\".");
            return text.ToString();
        }

        // The findings as plain text, one paragraph a group and one line a file, so the list pipes as `rom dups` does.
        public static string Report(string library, IReadOnlyList<LibraryRow> rows)
        {
            var text = new StringBuilder();
            text.Append("# EmuSen library manager: ").AppendLine(library);
            foreach (Finding kind in Enum.GetValues<Finding>())
            {
                var found = rows.Where(r => r.Kind == kind).OrderBy(r => r.Group).ThenBy(r => r.Source, StringComparer.Ordinal).ToList();
                text.AppendLine().Append("## ").Append(KindName(kind)).Append(" (").Append(found.Count).AppendLine(")");
                int? last = null;
                foreach (LibraryRow r in found)
                {
                    if (r.Group != 0 && last is { } l && l != r.Group) text.AppendLine();
                    last = r.Group;
                    string hidden = r.Hidden ? "\thidden" : "";
                    text.Append(r.Source);
                    text.Append(kind switch
                    {
                        Finding.SameGame => $"\tscreenscraper {r.GameId}",
                        Finding.Broken => "\t" + string.Join("; ", r.Failed.Select(f => $"{f.Check}: {f.Words}")),
                        Finding.SameDump when r.Package is { } p => $"\tpackage {p}",
                        _ => "",
                    });
                    text.AppendLine(hidden);
                }
            }
            return text.ToString();
        }
    }
}
