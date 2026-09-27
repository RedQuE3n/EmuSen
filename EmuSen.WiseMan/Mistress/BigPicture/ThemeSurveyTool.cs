using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;
using EmuSen.Mistress.BigPicture.Theme;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public sealed class ThemeSurveyFactAttribute : FactAttribute
    {
        public ThemeSurveyFactAttribute(string phase)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_THEME_SURVEY") != phase)
                Skip = $"The theme survey's {phase} phase; set EMUSEN_THEME_SURVEY={phase}. It reads ~/.cache/emusen/bigpicture/theme-survey/, never the repository - see EmuSen_BigPicture.md §25";
        }
    }

    // Pass 3's survey: the listed themes' XML files only, fetched once and gently, then the loader over each for EmuSen's five systems - see EmuSen_BigPicture.md §25.
    public class ThemeSurveyTool
    {
        public static readonly string Folder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "theme-survey");

        private static readonly string[] Systems = ["nes", "snes", "gb", "gbc", "n64"];
        private static readonly TimeSpan ApiGap = TimeSpan.FromSeconds(2), FileGap = TimeSpan.FromMilliseconds(200);
        private const long Ceiling = 200L << 20;

        private readonly ITestOutputHelper _out;

        public ThemeSurveyTool(ITestOutputHelper output) => _out = output;

        private void Log(string line)
        {
            _out.WriteLine(line);
            File.AppendAllText(Path.Combine(Folder, "fetch.log"), $"{DateTime.Now:HH:mm:ss} {line}\n");
        }

        private static ThemeList List() => ThemeList.Parse(File.ReadAllText(Path.Combine(Folder, "themes.json")), File.GetLastWriteTimeUtc(Path.Combine(Folder, "themes.json")));

        // Sequential, one request at a time with a pause between; GitHub's hourly allowance is read from each answer and waited out, never exceeded.
        [ThemeSurveyFact("fetch")]
        public async Task Fetch_the_listed_themes_xml()
        {
            ThemeList list = List();
            using var http = new HttpClient { Timeout = TimeSpan.FromMinutes(2) };
            http.DefaultRequestHeaders.UserAgent.ParseAdd("EmuSen-theme-survey/1 (a one-time research survey of ES-DE's theme list)");
            long total = Directory.Exists(Path.Combine(Folder, "xml")) ? Directory.EnumerateFiles(Path.Combine(Folder, "xml"), "*", SearchOption.AllDirectories).Sum(f => new FileInfo(f).Length) : 0;
            int requests = 0;
            foreach (ThemeListEntry theme in list.Themes)
            {
                ThemeSource s = theme.Source!;
                string treeFile = Path.Combine(Folder, "trees", s.Repository + ".json");
                List<(string Path, long Size)> xml;
                if (File.Exists(treeFile)) xml = JsonSerializer.Deserialize<List<Blob>>(File.ReadAllText(treeFile))!.Select(b => (b.Path, b.Size)).ToList();
                else
                {
                    xml = await TreeAsync(http, s);
                    requests++;
                    Directory.CreateDirectory(Path.GetDirectoryName(treeFile)!);
                    File.WriteAllText(treeFile, JsonSerializer.Serialize(xml.Select(x => new Blob(x.Path, x.Size))));
                    Log($"{theme.Name}: {xml.Count} XML files, {xml.Sum(x => x.Size) / 1e6:F2} MB declared");
                    await Task.Delay(ApiGap);
                }
                foreach ((string path, long size) in xml)
                {
                    string target = Path.Combine(Folder, "xml", s.Repository, path);
                    if (File.Exists(target)) continue;
                    if (total + size > Ceiling) throw new InvalidOperationException($"the survey would pass {Ceiling >> 20} MB; stopped before {theme.Name}/{path}");
                    string address = s.Host == ThemeHost.GitHub
                        ? $"https://raw.githubusercontent.com/{s.Owner}/{s.Repository}/HEAD/{string.Join('/', path.Split('/').Select(Uri.EscapeDataString))}"
                        : $"https://gitlab.com/{s.Owner}/{s.Repository}/-/raw/HEAD/{string.Join('/', path.Split('/').Select(Uri.EscapeDataString))}";
                    using HttpResponseMessage r = await http.GetAsync(address);
                    requests++;
                    if (!r.IsSuccessStatusCode) { Log($"  {(int)r.StatusCode} {address}"); await Task.Delay(FileGap); continue; }
                    byte[] bytes = await r.Content.ReadAsByteArrayAsync();
                    Directory.CreateDirectory(Path.GetDirectoryName(target)!);
                    File.WriteAllBytes(target, bytes);
                    total += bytes.Length;
                    await Task.Delay(FileGap);
                }
            }
            Log($"done: {requests} requests this run, {total / 1e6:F2} MB of XML held");
        }

        private sealed record Blob(string Path, long Size);

        private async Task<List<(string, long)>> TreeAsync(HttpClient http, ThemeSource s)
        {
            if (s.Host == ThemeHost.GitHub)
            {
                using HttpResponseMessage r = await http.GetAsync($"https://api.github.com/repos/{s.Owner}/{s.Repository}/git/trees/HEAD?recursive=1");
                await RespectLimitAsync(r);
                if (!r.IsSuccessStatusCode)
                {
                    if (r.Headers.TryGetValues("x-ratelimit-remaining", out var left) && left.FirstOrDefault() == "0") return await TreeAsync(http, s);
                    throw new HttpRequestException($"{s.Url}: tree answered {(int)r.StatusCode}");
                }
                using JsonDocument doc = JsonDocument.Parse(await r.Content.ReadAsStringAsync());
                if (doc.RootElement.TryGetProperty("truncated", out JsonElement t) && t.GetBoolean()) Log($"  {s.Url}: the tree listing was truncated");
                return doc.RootElement.GetProperty("tree").EnumerateArray()
                    .Where(e => e.GetProperty("type").GetString() == "blob" && e.GetProperty("path").GetString()!.EndsWith(".xml", StringComparison.OrdinalIgnoreCase))
                    .Select(e => (e.GetProperty("path").GetString()!, e.TryGetProperty("size", out JsonElement z) ? z.GetInt64() : 0)).ToList();
            }
            var all = new List<(string, long)>();
            string project = Uri.EscapeDataString($"{s.Owner}/{s.Repository}");
            for (int page = 1; page < 100; page++)
            {
                using HttpResponseMessage r = await http.GetAsync($"https://gitlab.com/api/v4/projects/{project}/repository/tree?recursive=true&per_page=100&page={page}");
                r.EnsureSuccessStatusCode();
                using JsonDocument doc = JsonDocument.Parse(await r.Content.ReadAsStringAsync());
                int n = 0;
                foreach (JsonElement e in doc.RootElement.EnumerateArray())
                {
                    n++;
                    if (e.GetProperty("type").GetString() == "blob" && e.GetProperty("path").GetString()!.EndsWith(".xml", StringComparison.OrdinalIgnoreCase)) all.Add((e.GetProperty("path").GetString()!, 0));
                }
                await Task.Delay(ApiGap);
                if (n < 100) break;
            }
            return all;
        }

        private async Task RespectLimitAsync(HttpResponseMessage r)
        {
            if (!r.Headers.TryGetValues("x-ratelimit-remaining", out var left) || !int.TryParse(left.FirstOrDefault(), out int remaining) || remaining > 1) return;
            long reset = r.Headers.TryGetValues("x-ratelimit-reset", out var at) && long.TryParse(at.FirstOrDefault(), out long v) ? v : DateTimeOffset.UtcNow.AddHours(1).ToUnixTimeSeconds();
            TimeSpan wait = DateTimeOffset.FromUnixTimeSeconds(reset) - DateTimeOffset.UtcNow + TimeSpan.FromSeconds(15);
            if (wait < TimeSpan.Zero) return;
            Log($"  GitHub's hourly allowance is spent; waiting {wait.TotalMinutes:F0} minutes until it resets");
            await Task.Delay(wait);
        }

        // The loader over every theme, every selectable variant and EmuSen's five systems; what the scene does not draw is counted by theme.
        [ThemeSurveyFact("analyse")]
        public void Analyse_the_fetched_themes()
        {
            ThemeList list = List();
            var report = new StringBuilder();
            var themeRows = new List<object>();
            var elementThemes = new Dictionary<string, HashSet<string>>();
            var propertyThemes = new Dictionary<string, HashSet<string>>();
            var codeThemes = new Dictionary<string, HashSet<string>>();
            var carouselTypes = new Dictionary<string, HashSet<string>>();
            int loaded = 0, fullyDrawn = 0, errorFree = 0;
            var clock = Stopwatch.StartNew();
            foreach (ThemeListEntry theme in list.Themes)
            {
                string dir = Path.Combine(Folder, "xml", theme.Source!.Repository);
                if (!Directory.Exists(dir)) { themeRows.Add(new { theme.Name, State = "not fetched" }); continue; }
                string root = File.Exists(Path.Combine(dir, "capabilities.xml")) ? dir
                    : Directory.GetDirectories(dir).FirstOrDefault(d => File.Exists(Path.Combine(d, "capabilities.xml"))) ?? dir;
                ThemeCapabilities caps = ThemeCapabilitiesReader.Read(root);
                var variants = caps.Variants.Where(v => v.Selectable).Select(v => (string?)v.Name).DefaultIfEmpty(null).ToList();
                int themedSystems = 0, errors = 0, loads = 0;
                var codes = new Dictionary<string, int>();
                var undrawnElements = new HashSet<string>();
                var samples = new HashSet<string>();
                var undrawnProperties = new HashSet<string>();
                foreach (ThemeDiagnostic d in caps.Diagnostics.Where(d => d.Severity >= ThemeSeverity.Warning))
                    codes[$"{d.Severity}:{d.Code}"] = codes.GetValueOrDefault($"{d.Severity}:{d.Code}") + 1;
                foreach (string system in Systems)
                {
                    bool any = false;
                    foreach (string? variant in variants)
                    {
                        ResolvedTheme t = ThemeLoader.Load(caps, new ThemeSystem(system, system, system), new ThemeChoices { Variant = variant });
                        loads++;
                        any |= t.IsThemed;
                        errors += t.Errors.Count();
                        foreach (ThemeDiagnostic x in t.Errors.Take(Math.Max(0, 4 - samples.Count))) samples.Add($"{x.Code} {Path.GetRelativePath(root, x.File)}:{x.Line}: {x.Message}");
                        foreach (ThemeDiagnostic d in t.Diagnostics.Where(d => d.Severity >= ThemeSeverity.Warning))
                            codes[$"{d.Severity}:{d.Code}"] = codes.GetValueOrDefault($"{d.Severity}:{d.Code}") + 1;
                        foreach (ResolvedElement e in t.SystemView.Elements.Concat(t.GamelistView.Elements))
                        {
                            if (!SceneMapping.Drawn.Contains(e.Type)) { undrawnElements.Add(e.Type); continue; }
                            foreach (string p in e.Explicit.Keys)
                                if (!SceneMapping.IsMapped(e.Type, p, _ => true)) undrawnProperties.Add($"{e.Type}.{p}");
                            if (e.Type == "carousel") Add(carouselTypes, e.String("type") ?? "horizontal", theme.Name);
                        }
                    }
                    if (any) themedSystems++;
                }
                if (themedSystems == Systems.Length) loaded++;
                if (themedSystems == Systems.Length && errors == 0) errorFree++;
                bool whole = undrawnElements.Count == 0 && undrawnProperties.Count == 0
                             && !carouselTypes.Any(c => c.Key.EndsWith("Wheel", StringComparison.OrdinalIgnoreCase) && c.Value.Contains(theme.Name));
                if (whole && themedSystems == Systems.Length) fullyDrawn++;
                foreach (string e in undrawnElements) Add(elementThemes, e, theme.Name);
                foreach (string p in undrawnProperties) Add(propertyThemes, p, theme.Name);
                foreach (string c in codes.Keys) Add(codeThemes, c, theme.Name);
                themeRows.Add(new
                {
                    theme.Name, Variants = variants.Count, Loads = loads, ThemedSystems = themedSystems, Errors = errors, Codes = codes,
                    UndrawnElements = undrawnElements.Order().ToList(), UndrawnProperties = undrawnProperties.Order().ToList(), FullyDrawn = whole, ErrorSamples = samples.ToList(),
                    XmlFiles = Directory.GetFiles(dir, "*.xml", SearchOption.AllDirectories).Length,
                    XmlBytes = Directory.GetFiles(dir, "*.xml", SearchOption.AllDirectories).Sum(f => new FileInfo(f).Length),
                });
            }
            var summary = new
            {
                Themes = list.Themes.Count, LoadedForAllFive = loaded, LoadedWithNoError = errorFree, FullyDrawn = fullyDrawn, Seconds = clock.Elapsed.TotalSeconds,
                XmlBytes = Directory.GetFiles(Path.Combine(Folder, "xml"), "*", SearchOption.AllDirectories).Sum(f => new FileInfo(f).Length),
                XmlFiles = Directory.GetFiles(Path.Combine(Folder, "xml"), "*", SearchOption.AllDirectories).Length,
                UndrawnElements = elementThemes.OrderByDescending(k => k.Value.Count).ToDictionary(k => k.Key, k => k.Value.Count),
                UndrawnProperties = propertyThemes.OrderByDescending(k => k.Value.Count).ToDictionary(k => k.Key, k => k.Value.Count),
                CarouselTypes = carouselTypes.ToDictionary(k => k.Key, k => k.Value.Count),
                DiagnosticCodes = codeThemes.OrderByDescending(k => k.Value.Count).ToDictionary(k => k.Key, k => k.Value.Count),
            };
            File.WriteAllText(Path.Combine(Folder, "survey.json"), JsonSerializer.Serialize(new { summary, themes = themeRows }, new JsonSerializerOptions { WriteIndented = true }));
            _out.WriteLine(JsonSerializer.Serialize(summary, new JsonSerializerOptions { WriteIndented = true }));
        }

        private static void Add(Dictionary<string, HashSet<string>> map, string key, string theme)
        {
            if (!map.TryGetValue(key, out HashSet<string>? set)) map[key] = set = new HashSet<string>();
            set.Add(theme);
        }
    }
}
