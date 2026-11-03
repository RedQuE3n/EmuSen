using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;

namespace EmuSen.Mistress.Pages
{
    // Display ▸ Shaders: each console's screen filter from every one EmuSen has, the slang pack and its download, and the parameters of the one in use - see EmuSen_BigPicture.md §52.8.
    public sealed partial class SettingsScreen
    {
        private string ShaderOf(string console) => _host.Graphics.Value(console, GraphicsKeys.ScreenFilter) ?? ScreenFilters.None;

        private Setting ShadersRow() =>
            Opens("shaders", "Shaders", "Each console's screen filter from every one EmuSen has: the built-in filters and the slang pack's presets, and their parameters.",
                _host.RunningConsole is { } running ? ShaderCatalog.Describe(ShaderOf(running)) : null, () =>
                {
                    Push("Shaders", ShaderConsoles, "shaders");
                    if (_host.RunningConsole is { } shown) Push(NameOf(shown), () => ShaderRows(shown), "shader:" + shown);
                });

        private IReadOnlyList<Setting> ShaderConsoles()
        {
            ShaderShelf shelf = _host.Shaders;
            string pack = shelf.Said ?? (shelf.Built is null ? "Not downloaded" : Count(shelf.Presets.Count, "preset", "presets"));
            var rows = new List<Setting>
            {
                new(new PageRow("pack", "RetroArch's shaders") { Kind = PageRowKind.Asks, Status = pack, Sentence = "libretro's slang pack, downloaded once; A downloads it, or again for a newer one.", IsEnabled = shelf.Downloading is null },
                    () => Said(shelf.Built is null ? "The pack is not downloaded: the built-in filters are offered alone." : $"{Count(shelf.Presets.Count, "preset", "presets")} in {shelf.Pack}.", "A downloads libretro's slang pack, about 54 MB."),
                    Choose: _host.DownloadShaders),
            };
            rows.AddRange(_host.Systems.Select(s => s.Console).Distinct().Select(console =>
                Opens("shader:" + console, NameOf(console), $"The {NameOf(console)}'s screen filter and its parameters.", ShaderCatalog.Describe(ShaderOf(console)),
                    () => Push(NameOf(console), () => ShaderRows(console), "shader:" + console))));
            return rows;
        }

        // A console's page: the parameters of the shader in use, a search, the recent ones, the built-in filters, and the pack's folders.
        private IReadOnlyList<Setting> ShaderRows(string console)
        {
            ShaderShelf shelf = _host.Shaders;
            string current = ShaderOf(console);
            int count = shelf.ParametersOf(current).Count(p => !SlangParameters.IsHeading(p));
            var rows = new List<Setting>
            {
                Opens("params", "Parameters", $"The parameters of {ShaderCatalog.Describe(current)}, for the {NameOf(console)}.", count == 0 ? "None" : count.ToString(CultureInfo.InvariantCulture),
                    () => Push("Parameters", () => ParameterRows(console, current, ""), "params"), enabled: count > 0),
                Asks("find", "Search", "Words in a shader's name or folder.", () => _host.Ask("Shaders", "Words in the shader's name or folder", "", words =>
                    Push($"“{words}”", () => EntryRows(console, ShaderCatalog.Shown(ShaderCatalog.BuiltIns(console), shelf.Presets, [], words, ShaderCatalog.AllCategories)), "find"))),
            };
            IReadOnlyList<ShaderEntry> builtIns = ShaderCatalog.BuiltIns(console);
            foreach (string stored in _host.Graphics.RecentFor(console))
                if (builtIns.Concat(shelf.Presets).FirstOrDefault(e => e.Stored == stored) is { } used) rows.Add(UseRow(console, used, "recent:", ShaderCatalog.RecentGroup));
            rows.AddRange(builtIns.Select(e => UseRow(console, e, "use:", ShaderCatalog.BuiltIn)));
            foreach (IGrouping<string, ShaderEntry> folder in shelf.Presets.GroupBy(e => ShaderCatalog.Category(e.Relative!), StringComparer.OrdinalIgnoreCase))
            {
                List<ShaderEntry> presets = folder.ToList();
                rows.Add(Opens("folder:" + folder.Key, folder.Key, $"The pack's {folder.Key} presets.", Count(presets.Count, "preset", "presets"),
                    () => Push(folder.Key, () => EntryRows(console, presets), "folder:" + folder.Key)));
            }
            return rows;
        }

        private IReadOnlyList<Setting> EntryRows(string console, IReadOnlyList<ShaderEntry> entries) => entries.Select(e => UseRow(console, e, "use:", e.Group)).ToList();

        private Setting UseRow(string console, ShaderEntry entry, string prefix, string where)
        {
            bool inUse = entry.Stored == ShaderOf(console);
            bool there = _host.Shaders.Has(entry);
            return new Setting(new PageRow(prefix + entry.Stored, entry.Name) { Kind = PageRowKind.Action, Status = inUse ? "● In use" : null, Sentence = where, IsEnabled = there },
                () => Said(inUse ? $"{entry.Name} draws the {NameOf(console)}'s picture." : $"A draws the {NameOf(console)}'s picture with {entry.Name}.",
                    there ? entry.Relative ?? ShaderCatalog.BuiltIn : "Not in the downloaded pack, so it is drawn plain until the pack has it again."),
                Choose: () =>
                {
                    _host.Graphics.SetValue(console, GraphicsKeys.ScreenFilter, entry.Stored);
                    if (entry.Stored != ScreenFilters.None) _host.Graphics.NoteRecent(console, entry.Stored, ShaderCatalog.RecentKept);
                    _host.SaveGraphics(console);
                });
        }

        // A shader's parameters for one console, each stepped by its own step or through its named choices; a value at its default is not kept.
        private IReadOnlyList<Setting> ParameterRows(string console, string stored, string search)
        {
            IReadOnlyList<SlangParameter> parameters = _host.Shaders.ParametersOf(stored);
            IReadOnlyDictionary<string, string> set = _host.Graphics.ParametersFor(console, stored);
            ScreenFilter? builtIn = ShaderCatalog.For(stored).IsPreset ? null : ScreenFilters.Find(stored).Filter;
            var rows = new List<Setting>
            {
                Asks("paramsearch", "Search parameters", "Words in a parameter's name.", () => _host.Ask("Parameters", "Words in the parameter's name", search, words =>
                    Push($"“{words}”", () => ParameterRows(console, stored, words), "paramsearch"))),
                Act("paramreset", "Reset all", "Every parameter back to the shader's own value.", () => { _host.Graphics.ForgetParameter(console, stored); _host.SaveGraphics(console); }, enabled: set.Count > 0),
            };
            // A search keeps a parameter whose name or id matches, and every parameter under a heading that matches, each under its heading.
            var groups = new List<(SlangParameter? Heading, List<SlangParameter> Parameters)> { (null, []) };
            foreach (SlangParameter p in parameters)
            {
                if (SlangParameters.IsHeading(p)) groups.Add((string.IsNullOrWhiteSpace(p.Description.Trim(' ', '-', '=', '*', '#', '[', ']')) ? null : p, []));
                else groups[^1].Parameters.Add(p);
            }
            string LabelOf(SlangParameter p) => string.IsNullOrWhiteSpace(p.Description) ? p.Id : p.Description.Trim();
            if (search.Length > 0)
                groups = groups.Select(g => (g.Heading, g.Heading is { } h && FilterBar.MatchesWords(search, h.Description) ? g.Parameters : g.Parameters.Where(p => FilterBar.MatchesWords(search, LabelOf(p), p.Id)).ToList())).ToList();
            groups = groups.Where(g => g.Parameters.Count > 0).ToList();

            // A long list is a page per heading, split again past the page's size, so no page holds hundreds of rows - see EmuSen_BigPicture.md §52.8.
            if (search.Length == 0 && groups.Sum(g => g.Parameters.Count) > PageOfParameters)
            {
                var pages = groups.SelectMany(g => g.Parameters.Chunk(PageOfParameters).Select((chunk, i) => (g.Heading, Chunk: chunk, Part: i))).ToList();
                int first = 1;
                foreach ((SlangParameter? h, SlangParameter[] chunk, int part) in pages)
                {
                    string key = "sec:" + (h?.Id ?? "") + (part > 0 || h is null ? $"{first}" : "");
                    string title = h is { } named ? named.Description.Trim() + (part > 0 ? $" ({part + 1})" : "") : $"Parameters {first} to {first + chunk.Length - 1}";
                    rows.Add(Opens(key, title, $"{Count(chunk.Length, "parameter", "parameters")} of the shader's.", chunk.Length.ToString(CultureInfo.InvariantCulture),
                        () => Push(title, () => chunk.Select(p => ParameterRow(console, stored, p, LabelOf(p), builtIn, _host.Graphics.ParametersFor(console, stored))).ToList(), key)));
                    first += chunk.Length;
                }
                return rows;
            }
            foreach ((SlangParameter? h, List<SlangParameter> list) in groups)
            {
                if (h is { } shown) rows.Add(Act("h:" + shown.Id, shown.Description.Trim(), "A heading of the shader's parameters.", () => { }, enabled: false));
                rows.AddRange(list.Select(p => ParameterRow(console, stored, p, LabelOf(p), builtIn, set)));
            }
            return rows;
        }

        // One parameter, stepped by its own step or through its named choices; a value at its default is not kept.
        private Setting ParameterRow(string console, string stored, SlangParameter p, string label, ScreenFilter? builtIn, IReadOnlyDictionary<string, string> set)
        {
            // A built-in filter's default may be this console's own - see EmuSen_Serenity.md §3.9.
            double initial = Exact(builtIn?.DefaultFor(p, RuntimeCatalog.HostOf(console)) ?? p.Initial);
            double value = set.TryGetValue(p.Id, out string? text) && float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out float given) ? Exact(given) : initial;
            double step = p.Step > 0 ? Exact(p.Step) : (Exact(p.Maximum) - Exact(p.Minimum)) / 100;
            string id = p.Id;
            void Store(double to)
            {
                if (to == initial) _host.Graphics.ForgetParameter(console, stored, id);
                else _host.Graphics.SetParameter(console, stored, id, ((float)Math.Round(to, 6)).ToString(CultureInfo.InvariantCulture));
                _host.SaveGraphics(console);
            }
            if (p.Choices is { Count: > 0 } choices)
            {
                int own = Math.Clamp((int)Math.Round((initial - Exact(p.Minimum)) / step), 0, choices.Count - 1);
                return Choice("p:" + id, label, $"{label}, for the {NameOf(console)}; {choices[own]} is the shader's own.", choices.ToList(), (int)Math.Round((value - Exact(p.Minimum)) / step), i => Store(Exact(p.Minimum) + i * step));
            }
            return new Setting(new PageRow("p:" + id, label) { Kind = PageRowKind.Steps, Status = value.ToString("0.####", CultureInfo.InvariantCulture), Sentence = $"{label}, for the {NameOf(console)}; {initial.ToString("0.####", CultureInfo.InvariantCulture)} is the shader's own." },
                null, by => Store(Math.Clamp(Math.Round(value + by * step, 6), Exact(p.Minimum), Exact(p.Maximum))));
        }

        // The most parameters a page holds before a long list is split by its headings.
        private const int PageOfParameters = 120;

        // A float as the decimal it was written as, so 0.041f is 0.041 and not 0.041000001.
        private static double Exact(float value) => (double)(decimal)value;
    }
}
