using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Threading;

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
    //
    // File writes happen on a dedicated background thread, not the
    // calling thread. Every Console.WriteLine in the emulation core's hot
    // path (every instruction under CpuVerboseLogging, every write under
    // DmaVerboseLogging, etc.) used to route straight into a StreamWriter
    // with AutoFlush=true - a real disk write/OS syscall per line, on
    // whichever thread is driving emulation (the Avalonia GUI's 60fps
    // DispatcherTimer tick, in EmuSen.TestingStudio's case). The calling
    // thread now just resolves which category a line belongs to (cheap -
    // a prefix match against an in-memory table) and hands the line to a
    // bounded queue; a single consumer thread drains it and does the
    // actual buffered file I/O. Console echo stays synchronous and
    // un-queued on purpose - it's cheap relative to disk I/O, and live
    // debugging benefits from seeing output immediately rather than
    // delayed behind a queue.
    public class CategorizedLogWriter : System.IO.TextWriter
    {
        // A queue this deep would mean tens of thousands of log lines
        // backed up behind disk I/O - past any burst this project's own
        // verbose flags realistically produce against normal storage.
        // BlockingCollection<T>.Add blocks the calling (producer) thread
        // once full rather than growing without bound - an intentional
        // safety valve against unbounded memory growth if disk I/O ever
        // falls catastrophically behind, accepted even though it
        // reintroduces the exact blocking this class exists to avoid,
        // because the alternative (unbounded growth) is worse.
        private const int QueueCapacity = 10_000;

        private readonly struct LogEntry
        {
            public readonly StreamWriter Target;
            public readonly string? Text;
            public readonly bool IsLine;

            public LogEntry(StreamWriter target, string? text, bool isLine)
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

            // Background, not foreground - if something skips Dispose(),
            // this thread must never be the reason the process won't exit.
            // Dispose() still drains it properly for the normal path (see
            // its own comment on why that matters for a long-lived
            // frontend switching ROMs mid-session).
            _worker = new Thread(ConsumeQueue) { IsBackground = true, Name = "EmuSen-LogWriter" };
            _worker.Start();
        }

        // Deliberately no AutoFlush here, unlike the old synchronous
        // design - StreamWriter's own internal buffer is what makes
        // batched background writes actually cheaper than the old
        // flush-every-line behavior. Flushed explicitly on Dispose (and,
        // best-effort, whenever Flush() is called - see that override).
        private static StreamWriter OpenFile(string logDir, string category)
        {
            return new StreamWriter(Path.Combine(logDir, category + ".log"), append: false);
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

        // Runs entirely on _worker. GetConsumingEnumerable() blocks
        // whenever the queue is empty and returns cleanly once
        // CompleteAdding() has been called AND every already-queued entry
        // has been drained - exactly the "finish everything, then stop"
        // behavior Dispose() needs.
        private void ConsumeQueue()
        {
            foreach (LogEntry entry in _queue.GetConsumingEnumerable())
            {
                if (entry.IsLine) entry.Target.WriteLine(entry.Text);
                else entry.Target.Write(entry.Text);
            }
        }

        public override System.Text.Encoding Encoding => _console.Encoding;

        public override void Write(char value)
        {
            _console.Write(value);
            _queue.Add(new LogEntry(_general, value.ToString(), isLine: false));
        }

        public override void Write(string? value)
        {
            _console.Write(value);
            _queue.Add(new LogEntry(ResolveFile(value), value, isLine: false));
        }

        public override void WriteLine(string? value)
        {
            _console.WriteLine(value);
            _queue.Add(new LogEntry(ResolveFile(value), value, isLine: true));
        }

        public override void WriteLine()
        {
            _console.WriteLine();
            _queue.Add(new LogEntry(_general, null, isLine: true));
        }

        // Deliberately does NOT wait for the background thread to catch up
        // and actually flush the files - nothing in this codebase calls
        // Flush() explicitly today (verified before making this async;
        // TextWriter requires the override regardless), so there's no
        // existing caller relying on a synchronous guarantee here. Only
        // the console copy is flushed synchronously. Dispose() is the one
        // place with a real, waited-for guarantee - see its own comment.
        public override void Flush()
        {
            _console.Flush();
        }

        // Both frontends now call this explicitly on every exit path
        // (EmuSen.RaylibFrontend's Program.cs in a finally block;
        // EmuSen.TestingStudio's MainWindow on window-close and before
        // starting a fresh session for the next loaded ROM). That used to
        // only matter for the long-lived Avalonia build - the console
        // build got away with never disposing at all, back when
        // AutoFlush=true meant every write already reached disk
        // immediately and there was nothing buffered to lose. Now that
        // file writes are queued to a background thread with no
        // AutoFlush, skipping Dispose() on process exit would silently
        // drop however much was still sitting in the queue/StreamWriter
        // buffers - a real behavior change this refactor had to account
        // for, not just an optional cleanup.
        //
        // This is the one place a synchronous wait is actually correct:
        // CompleteAdding() lets ConsumeQueue's loop drain the rest of the
        // queue and exit on its own, and Join() blocks until it does -
        // guaranteeing every line queued before Dispose() was called
        // actually reaches its file before the handles get closed
        // underneath it. Bounded to 5 seconds so a stuck disk can't hang
        // process/ROM-switch shutdown forever; if it times out, the
        // Dispose calls below still run and close what they can.
        protected override void Dispose(bool disposing)
        {
            if (disposing)
            {
                _queue.CompleteAdding();
                _worker.Join(TimeSpan.FromSeconds(5));

                foreach (var file in _files.Values) file.Dispose();
                _general.Dispose();
                _queue.Dispose();
            }
            base.Dispose(disposing);
        }
    }
}
