using System;
using System.Collections.Generic;
using System.IO;

namespace EmuSen.Common
{
    // Splits console output across several smaller, topically-grouped log
    // files instead of one ever-growing console.log - the single-file
    // version routinely reached hundreds of thousands of lines in a single
    // play session (WATCH/SCROLL/DMA-SRC traces are all extremely
    // high-volume), which made the file slow to search and, in practice,
    // too large to hand off for review at all. Categories mirror the same
    // CPU/PPU/APU/Memory split Man pages/Hardware/ already uses, plus a
    // "debug" bucket for the core-agnostic debug toolchain's own output
    // (watch/state/status/error) and a "general" catch-all for anything
    // that isn't tagged with a recognized bracketed prefix at all (ROM
    // load, cartridge info, screenshot/recording confirmations, the
    // interactive prompt itself).
    //
    // Still writes everything to the real console too (same as
    // TeeTextWriter, which this otherwise mirrors) - nothing about live
    // viewing changes, only where the file copy of each line ends up.
    public class CategorizedLogWriter : System.IO.TextWriter
    {
        private readonly TextWriter _console;
        private readonly Dictionary<string, StreamWriter> _files;
        private readonly StreamWriter _general;

        // Longer/more specific prefixes aren't needed here since every tag
        // below is a distinct literal string - first match wins, order
        // doesn't otherwise matter.
        private static readonly (string Prefix, string Category)[] Routes =
        {
            ("[CPU HALT]", "cpu"), ("[IRQ FIRE]", "cpu"), ("[DEBUG]", "cpu"), ("[CPU]", "cpu"),

            ("[BACKDROP]", "ppu"), ("[BG SCROLL]", "ppu"), ("[BG3 CHECK]", "ppu"),
            ("[BG3SCROLL]", "ppu"), ("[BGMODE]", "ppu"), ("[BLACK TILE]", "ppu"),
            ("[CAMRAM]", "ppu"), ("[CGWRITE]", "ppu"), ("[MOSAIC]", "ppu"),
            ("[OAM DUMP]", "ppu"), ("[RENDER-READ]", "ppu"), ("[SCROLL]", "ppu"), ("[WINDOW]", "ppu"),

            ("[DEBUG APU]", "apu"), ("[PORT]", "apu"), ("[SPC700]", "apu"), ("[TRACE]", "apu"),

            ("[DMA-SRC]", "memory"), ("[DMA]", "memory"), ("[HDMA-BLOCK]", "memory"),
            ("[HDMA-WINDOW]", "memory"), ("[HVIRQ]", "memory"), ("[MATH]", "memory"),

            ("[WATCH", "debug"), ("[STATE]", "debug"), ("[STATUS]", "debug"), ("[ERROR]", "debug"),
        };

        public CategorizedLogWriter(TextWriter console, string logDir)
        {
            _console = console;
            _files = new Dictionary<string, StreamWriter>();
            foreach (var (_, category) in Routes)
            {
                if (_files.ContainsKey(category)) continue;
                _files[category] = OpenFile(logDir, category);
            }
            _general = OpenFile(logDir, "general");
        }

        private static StreamWriter OpenFile(string logDir, string category)
        {
            return new StreamWriter(Path.Combine(logDir, category + ".log"), append: false) { AutoFlush = true };
        }

        private StreamWriter ResolveFile(string? value)
        {
            if (value == null) return _general;
            string trimmed = value.TrimStart('\n', '\r', ' ');
            foreach (var (prefix, category) in Routes)
            {
                if (trimmed.StartsWith(prefix, StringComparison.Ordinal)) return _files[category];
            }
            return _general;
        }

        public override System.Text.Encoding Encoding => _console.Encoding;

        public override void Write(char value)
        {
            _console.Write(value);
            _general.Write(value);
        }

        public override void Write(string? value)
        {
            _console.Write(value);
            ResolveFile(value).Write(value);
        }

        public override void WriteLine(string? value)
        {
            _console.WriteLine(value);
            ResolveFile(value).WriteLine(value);
        }

        public override void WriteLine()
        {
            _console.WriteLine();
            _general.WriteLine();
        }

        public override void Flush()
        {
            _console.Flush();
            _general.Flush();
            foreach (var file in _files.Values) file.Flush();
        }
    }
}
