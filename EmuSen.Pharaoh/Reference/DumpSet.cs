using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using Directory2 = System.IO.Directory;

namespace EmuSen.Pharaoh.Reference
{
    // One emulator's output, read without knowing which emulator made it - see §3.48.
    public sealed class DumpSet
    {
        public string Backend { get; private set; } = "";
        public string SystemName { get; private set; } = "";
        public string Rom { get; private set; } = "";
        public string Board { get; private set; } = "";
        public string Region { get; private set; } = "";
        public string HeaderTrust { get; private set; } = "";
        public long PrgBytes { get; private set; }
        public long ChrBytes { get; private set; }
        public bool SaveLoaded { get; private set; }
        public string ScreenFormat { get; private set; } = "";
        public string Directory { get; private set; } = "";

        // frame -> column -> crc. Empty when the set was produced without --sig.
        public Dictionary<long, Dictionary<string, uint>> Signature { get; } = new();
        public List<string> Columns { get; } = new();

        private static readonly Regex ManifestName = new(@"^(?<backend>.+?)_manifest_f(?<frame>\d+)\.json$");

        public static DumpSet? Load(string dir)
        {
            var set = new DumpSet { Directory = dir };

            string? sig = Directory2.EnumerateFiles(dir, "*_sig.csv").FirstOrDefault();
            if (sig is not null) set.ReadSignature(sig);

            string? manifest = Directory2.EnumerateFiles(dir, "*_manifest_f*.json")
                .OrderBy(p => p).FirstOrDefault();
            if (manifest is not null) set.ReadManifest(manifest);

            return set.Backend.Length > 0 ? set : null;
        }

        private void ReadSignature(string path)
        {
            foreach (string line in File.ReadLines(path))
            {
                if (line.StartsWith('#'))
                {
                    int eq = line.IndexOf('=');
                    if (eq < 0) continue;
                    string key = line[1..eq].Trim();
                    string value = line[(eq + 1)..].Trim();
                    switch (key)
                    {
                        case "backend": Backend = value; break;
                        case "system": SystemName = value; break;
                        case "rom": Rom = value; break;
                        case "board": Board = value; break;
                        case "region": Region = value; break;
                        case "headerTrust": HeaderTrust = value; break;
                        case "prg": PrgBytes = Parse(value); break;
                        case "chr": ChrBytes = Parse(value); break;
                        case "saveLoaded": SaveLoaded = value == "1"; break;
                        case "screenFormat": ScreenFormat = value; break;
                    }
                    continue;
                }

                string[] fields = line.Split(',');
                if (fields.Length < 2) continue;

                if (fields[0] == "frame")
                {
                    Columns.Clear();
                    Columns.AddRange(fields[1..]);
                    continue;
                }

                if (!long.TryParse(fields[0], out long frame)) continue;
                var row = new Dictionary<string, uint>();
                for (int i = 0; i < Columns.Count && i + 1 < fields.Length; i++)
                {
                    row[Columns[i]] = uint.Parse(fields[i + 1], NumberStyles.HexNumber, CultureInfo.InvariantCulture);
                }
                Signature[frame] = row;
            }
        }

        // Regex rather than a JSON dependency, matching how the manifest is written.
        private void ReadManifest(string path)
        {
            string text = File.ReadAllText(path);
            Match name = ManifestName.Match(Path.GetFileName(path));
            if (Backend.Length == 0 && name.Success) Backend = name.Groups["backend"].Value;

            Backend = Pick(text, "backend", Backend);
            SystemName = Pick(text, "system", SystemName);
            Rom = Pick(text, "rom", Rom);
            Board = Pick(text, "board", Board);
            Region = Pick(text, "region", Region);
            HeaderTrust = Pick(text, "headerTrust", HeaderTrust);
            ScreenFormat = Pick(text, "format", ScreenFormat);

            if (PrgBytes == 0) PrgBytes = PickNumber(text, "prg");
            if (ChrBytes == 0) ChrBytes = PickNumber(text, "chr");
        }

        private static string Pick(string text, string key, string current)
        {
            Match m = Regex.Match(text, "\"" + key + "\"\\s*:\\s*\"(?<v>[^\"]*)\"");
            return m.Success && m.Groups["v"].Value.Length > 0 ? m.Groups["v"].Value : current;
        }

        private static long PickNumber(string text, string key)
        {
            Match m = Regex.Match(text, "\"" + key + "\"\\s*:\\s*(?<v>\\d+)");
            return m.Success ? long.Parse(m.Groups["v"].Value, CultureInfo.InvariantCulture) : 0;
        }

        private static long Parse(string value) =>
            long.TryParse(value, NumberStyles.Integer, CultureInfo.InvariantCulture, out long n) ? n : 0;

        public IEnumerable<long> ReportFrames() =>
            Directory2.EnumerateFiles(Directory, $"{Backend}_manifest_f*.json")
                .Select(p => ManifestName.Match(Path.GetFileName(p)))
                .Where(m => m.Success)
                .Select(m => long.Parse(m.Groups["frame"].Value, CultureInfo.InvariantCulture))
                .OrderBy(f => f);

        public ScreenImage? Screen(long frame)
        {
            string path = Path.Combine(Directory, $"{Backend}_screen_f{frame:D5}.bin");
            return File.Exists(path) ? ScreenImage.Read(File.ReadAllBytes(path), ScreenFormat) : null;
        }
    }
}
