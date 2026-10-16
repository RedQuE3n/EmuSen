using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Security.Cryptography;
using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // A golden trace: per frame eight bytes of SHA-256 of the state, the picture and the sound, or a chain written every so many frames or instructions - see Moon_Native.md §9.2.
    public sealed class GoldenTrace
    {
        public const int Version = 1;

        public string Heading = "";
        public string Name = "";
        public string Rom = "";
        public string Input = "";
        public string Rendering = "";
        public string RecordedBy = "";
        public string Reason = "";

        // "frame": rows (index, state, picture, sound); "chain": rows (index, chain) over frames; "step": rows (index, chain) over instructions.
        public string Unit = "frame";

        public readonly List<string[]> Rows = new();
        public readonly SortedDictionary<string, string> Transcript = new(StringComparer.Ordinal);

        public string Text
        {
            get
            {
                var text = new StringBuilder();
                if (Heading.Length > 0) text.Append("# ").Append(Heading).Append('\n');
                text.Append($"golden {Version}\nname {Name}\nrom {Rom}\ninput {Input}\nrendering {Rendering}\nunit {Unit}\nrecorded {RecordedBy}\nreason {Reason}\n");
                string[]? last = null;
                foreach (string[] row in Rows)
                {
                    text.Append(row[0]);
                    for (int i = 1; i < row.Length; i++) text.Append(' ').Append(last is not null && last.Length == row.Length && last[i] == row[i] ? "=" : row[i]);
                    text.Append('\n');
                    last = row;
                }
                foreach (var (key, value) in Transcript) text.Append($"{key} {value}\n");
                return text.ToString();
            }
        }

        public static GoldenTrace Parse(string text)
        {
            var trace = new GoldenTrace();
            string[]? last = null;
            foreach (string raw in text.Split('\n'))
            {
                string line = raw.TrimEnd('\r');
                if (line.Length == 0 || line[0] == '#') continue;
                string[] p = line.Split(' ');
                if (char.IsDigit(line[0]))
                {
                    string[] row = p.Select((v, i) => v == "=" && last is not null ? last[i] : v).ToArray();
                    trace.Rows.Add(row);
                    last = row;
                    continue;
                }
                string value = line.Length > p[0].Length ? line[(p[0].Length + 1)..] : "";
                switch (p[0])
                {
                    case "golden": Assert.Equal(Version, int.Parse(value, CultureInfo.InvariantCulture)); break;
                    case "name": trace.Name = value; break;
                    case "rom": trace.Rom = value; break;
                    case "input": trace.Input = value; break;
                    case "rendering": trace.Rendering = value; break;
                    case "unit": trace.Unit = value; break;
                    case "recorded": trace.RecordedBy = value; break;
                    case "reason": trace.Reason = value; break;
                    default: trace.Transcript[p[0]] = value; break;
                }
            }
            return trace;
        }

        // The first difference from another trace of the same run, or null: the header, then row by row, then the transcript.
        public string? FirstDifference(GoldenTrace now)
        {
            foreach (var (what, a, b) in new[] { ("rom", Rom, now.Rom), ("input", Input, now.Input), ("rendering", Rendering, now.Rendering), ("unit", Unit, now.Unit) })
                if (a != b) return $"{what}: recorded {a}, now {b}";
            string[] parts = Unit == "frame" ? new[] { "", "state", "picture", "sound" } : new[] { "", "chain" };
            string unit = Unit == "step" ? "step" : "frame";
            for (int i = 0; i < Math.Min(Rows.Count, now.Rows.Count); i++)
            {
                if (Rows[i].SequenceEqual(now.Rows[i])) continue;
                int at = Enumerable.Range(0, Rows[i].Length).First(k => k >= now.Rows[i].Length || Rows[i][k] != now.Rows[i][k]);
                return $"{unit} {Rows[i][0]}: the {parts[Math.Min(at, parts.Length - 1)]} differs (recorded {Rows[i][at]}, now {(at < now.Rows[i].Length ? now.Rows[i][at] : "nothing")})";
            }
            if (Rows.Count != now.Rows.Count) return $"recorded {Rows.Count} rows, now {now.Rows.Count}";
            foreach (string key in Transcript.Keys.Union(now.Transcript.Keys).Order(StringComparer.Ordinal))
            {
                string? a = Transcript.GetValueOrDefault(key), b = now.Transcript.GetValueOrDefault(key);
                if (a != b) return $"{key}: recorded {a ?? "nothing"}, now {b ?? "nothing"}";
            }
            return null;
        }

        public static string Hash(ReadOnlySpan<byte> bytes) => Convert.ToHexString(SHA256.HashData(bytes))[..16];

        // chain = the first eight bytes of SHA-256(chain ‖ link), where link is eight bytes of a hash.
        public static void Fold(byte[] chain, ReadOnlySpan<byte> link)
        {
            Span<byte> both = stackalloc byte[16];
            chain.CopyTo(both);
            link[..8].CopyTo(both[8..]);
            SHA256.HashData(both)[..8].CopyTo(chain, 0);
        }
    }
}
