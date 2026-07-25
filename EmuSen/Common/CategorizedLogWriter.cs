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
    // actual buffered file I/O, flushing every category to disk on a
    // fixed timer (see FlushIntervalMs) rather than after every line, so
    // a hard kill or crash that never reaches Dispose() only loses a
    // fraction of a second of output instead of the whole session -
    // every category, not just whichever ones happen to be high-volume
    // enough to fill their own internal buffer (see FlushIntervalMs's own
    // comment for the real session that exposed this). Console echo
    // stays synchronous and un-queued on purpose - it's cheap relative to
    // disk I/O, and live debugging benefits from seeing output
    // immediately rather than delayed behind a queue.
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
        // flush-every-line behavior. Flushed on a fixed timer by
        // ConsumeQueue (see FlushIntervalMs) and unconditionally on
        // Dispose.
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

        // Every category gets an actual disk flush at least this often -
        // frequent enough that a hard kill or native crash (a Raylib
        // segfault, a Ctrl+C that skips .NET's normal unwind, anything
        // that never reaches Dispose()) loses at most a fraction of a
        // second of buffered lines, rare enough that it's nowhere near
        // the per-line syscall cost AutoFlush=true used to pay during a
        // real burst (CpuVerboseLogging etc.). Dispose()'s own
        // drain-then-flush is still the real guarantee for a graceful
        // shutdown; this is damage control for the ungraceful ones.
        //
        // Deliberately time-based, not "flush whenever the queue happens
        // to go idle" (the first version of this did that, via TryTake's
        // own timeout) - a real session showed why that's not good
        // enough: with one shared queue across every category, a handful
        // of high-volume categories (ppu/memory/debug under their normal
        // default-on trace flags) can keep the queue non-empty
        // continuously for an entire play session, so an idle-triggered
        // flush might never fire even once. .NET's own internal
        // StreamWriter/FileStream buffers happened to flush those
        // high-volume files anyway purely from sheer data volume - but
        // low-volume categories (cpu/apu/general, often just a session's
        // one-time startup banner) never filled that buffer and got
        // flushed exactly zero times before a non-graceful exit, ending
        // up completely empty. A plain elapsed-time check on every loop
        // iteration - independent of whether TryTake found anything -
        // flushes all categories on the same cadence regardless of how
        // busy any single one of them is.
        private const int FlushIntervalMs = 500;

        // Runs entirely on _worker. TryTake blocks up to FlushIntervalMs
        // waiting for the next entry (bounding how long a flush check can
        // be delayed even if the queue's truly empty); if one arrives,
        // write it - no flush yet, that's what makes batching cheaper
        // than the old per-line AutoFlush. The elapsed-time check runs on
        // every iteration regardless of whether TryTake found something,
        // which is what fixes the starvation case above. IsCompleted
        // (CompleteAdding() called AND the queue is empty) is what
        // actually ends the loop - see Dispose() for why that combination
        // matters.
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
        // process/ROM-switch shutdown forever.
        //
        // The original version of this closed the files unconditionally
        // even when Join() timed out - "close what they can" - but that's
        // a real cross-thread race, not a graceful degradation: if
        // _worker is still mid-write when file.Dispose() runs on this
        // thread, the two race on the same FileStream, and the result is
        // exactly the truncated-mid-line tails a real session turned up
        // (a line cut off partway through, no trailing newline) even on a
        // clean window-close that went through this exact path. Only
        // closing the files once Join() confirms the worker actually
        // exited removes that race; a timeout now leaves the handles open
        // for the OS to reclaim on process exit instead, which loses
        // whatever was still mid-flight but never corrupts what's already
        // on disk.
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
                    // Visible on the real console (not routed through the
                    // queue - the worker that would drain it is the very
                    // thing that didn't finish) so this doesn't disappear
                    // silently if it ever actually happens.
                    _console.WriteLine("[LOG] Warning: background log writer did not finish draining within 5s; some log output may be incomplete.");
                }
                _queue.Dispose();
            }
            base.Dispose(disposing);
        }
    }
}
