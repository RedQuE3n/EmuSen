using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Threading;

namespace EmuSen.Common
{
    // Console output split by tag, written on a background thread - see EmuSen_Debugging_Tools_Reference_v5.md §2.1.
    public class CategorizedLogWriter : System.IO.TextWriter
    {
        // Blocks the producer once full, which beats unbounded growth - see §2.1.
        private const int QueueCapacity = 100_000;

        // TextWriter, not StreamWriter, so the console can share this queued path - see §2.1.
        private readonly struct LogEntry
        {
            public readonly TextWriter Target;
            public readonly string? Text;
            public readonly bool IsLine;

            public LogEntry(TextWriter target, string? text, bool isLine)
            {
                Target = target;
                Text = text;
                IsLine = isLine;
            }
        }

        private readonly TextWriter _console;
        private readonly Dictionary<string, StreamWriter> _files;
        private readonly StreamWriter _general;
        private readonly BlockingCollection<LogEntry> _queue = new(QueueCapacity);
        private readonly Thread _worker;

        // Every tag is a distinct literal, so first match wins and order does not matter.
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

            // Background, so skipping Dispose can never hold the process open - see §2.1.
            _worker = new Thread(ConsumeQueue) { IsBackground = true, Name = "EmuSen-LogWriter" };
            _worker.Start();
        }

        // No AutoFlush: the internal buffer is what makes batching cheaper - see §2.1.
        private static StreamWriter OpenFile(string logDir, string category)
        {
            return new StreamWriter(Path.Combine(logDir, category + ".log"), append: false);
        }

        // cpu and apu only; nothing else logs per instruction - see §2.1.
        private static readonly HashSet<string> NoConsoleEchoCategories = new(StringComparer.Ordinal) { "cpu", "apu" };

        private (StreamWriter File, bool EchoToConsole) Resolve(string? value)
        {
            if (value == null) return (_general, true);
            string trimmed = value.TrimStart('\n', '\r', ' ');
            foreach (var (prefix, category) in Routes)
            {
                if (trimmed.StartsWith(prefix, StringComparison.Ordinal))
                {
                    return (_files[category], !NoConsoleEchoCategories.Contains(category));
                }
            }
            return (_general, true);
        }

        // Damage control for a hard kill, not the durability guarantee - see §2.1.
        private const int FlushIntervalMs = 500;

        // TryTake bounds the wait; the elapsed check runs regardless, which fixes starvation - see §2.1.
        private void ConsumeQueue()
        {
            var sinceLastFlush = System.Diagnostics.Stopwatch.StartNew();

            while (!_queue.IsCompleted)
            {
                if (_queue.TryTake(out LogEntry entry, FlushIntervalMs))
                {
                    if (entry.IsLine) entry.Target.WriteLine(entry.Text);
                    else entry.Target.Write(entry.Text);
                }

                if (sinceLastFlush.ElapsedMilliseconds >= FlushIntervalMs)
                {
                    FlushAllFiles();
                    sinceLastFlush.Restart();
                }
            }
        }

        private void FlushAllFiles()
        {
            foreach (var file in _files.Values) file.Flush();
            _general.Flush();
        }

        public override System.Text.Encoding Encoding => _console.Encoding;

        // Each override enqueues the console copy too, because it was never cheap - see §2.1.
        public override void Write(char value)
        {
            string text = value.ToString();
            _queue.Add(new LogEntry(_general, text, isLine: false));
            _queue.Add(new LogEntry(_console, text, isLine: false));
        }

        public override void Write(string? value)
        {
            var (file, echo) = Resolve(value);
            _queue.Add(new LogEntry(file, value, isLine: false));
            if (echo) _queue.Add(new LogEntry(_console, value, isLine: false));
        }

        public override void WriteLine(string? value)
        {
            var (file, echo) = Resolve(value);
            _queue.Add(new LogEntry(file, value, isLine: true));
            if (echo) _queue.Add(new LogEntry(_console, value, isLine: true));
        }

        public override void WriteLine()
        {
            _queue.Add(new LogEntry(_general, null, isLine: true));
            _queue.Add(new LogEntry(_console, null, isLine: true));
        }

        // No caller relies on a synchronous Flush; Dispose is the real guarantee - see §2.1.
        public override void Flush()
        {
            _console.Flush();
        }

        // Files close only once Join confirms the worker exited, or the two race - see §2.1.
        protected override void Dispose(bool disposing)
        {
            if (disposing)
            {
                _queue.CompleteAdding();
                bool drained = _worker.Join(TimeSpan.FromSeconds(5));

                if (drained)
                {
                    foreach (var file in _files.Values) file.Dispose();
                    _general.Dispose();
                }
                else
                {
                    // Straight to the console: the worker that would drain the queue is what failed.
                    _console.WriteLine("[LOG] Warning: background log writer did not finish draining within 5s; some log output may be incomplete.");
                }
                _queue.Dispose();
            }
            base.Dispose(disposing);
        }
    }
}
