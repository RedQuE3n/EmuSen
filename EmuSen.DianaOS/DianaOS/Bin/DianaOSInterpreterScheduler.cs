using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin
{
    // The "always live" model, lifted out of one host - see EmuSen_Debugging_Tools_Reference_v5.md §3.3a.
    public sealed class DianaOSInterpreterScheduler
    {
        private readonly object _lock = new();
        private readonly ConcurrentQueue<string> _pending = new();
        private readonly BlockingCollection<string> _haltedLines = new();
        private volatile bool _halted;

        public bool Halted { get => _halted; set => _halted = value; }

        // Runs a read-only line now, queues anything else; null means queued - see §3.3a.
        public (bool NeedsMoreInput, string Output, HostAction? Action)? SubmitFromAnyThread(DianaOSInterpreter shell, string line)
        {
            if (_halted)
            {
                _haltedLines.Add(line);
                return null;
            }

            string trimmed = line.Trim();
            if (trimmed.Length == 0) return null;

            if (shell.TryGetReadOnlyFastPath(trimmed, out _))
            {
                lock (_lock) return shell.Submit(trimmed);
            }

            _pending.Enqueue(trimmed);
            return null;
        }

        // Only from the thread that owns the core - see §3.3a.
        public IEnumerable<(bool NeedsMoreInput, string Output, HostAction? Action)> DrainPending(DianaOSInterpreter shell)
        {
            while (_pending.TryDequeue(out string? line))
            {
                (bool NeedsMoreInput, string Output, HostAction? Action) result;
                lock (_lock) result = shell.Submit(line!);
                yield return result;
            }
        }

        // Blocks until a line arrives while Halted - see §3.3a.
        public string TakeHaltedLine() => _haltedLines.Take();

        // For a caller that already decided this line runs now - see §3.3a.
        public (bool NeedsMoreInput, string Output, HostAction? Action) SubmitLocked(DianaOSInterpreter shell, string line)
        {
            lock (_lock) return shell.Submit(line);
        }

        // Under the same lock as every other path.
        public string[] SnapshotHistory(DianaOSInterpreter shell)
        {
            lock (_lock) return new List<string>(shell.History.Entries).ToArray();
        }
    }
}
