using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin
{
    // The "always live" model, lifted out of one host - see EmuSen_Debugging_Tools_Reference_v5.md §3.3a.
    public sealed class DianaShInterpreterScheduler
    {
        private readonly object _lock = new();
        private readonly ConcurrentQueue<string> _pending = new();
        private readonly BlockingCollection<string> _haltedLines = new();
        private volatile bool _halted;

        public bool Halted { get => _halted; set => _halted = value; }

        // Runs a read-only line now, queues anything else; null means queued - see §3.3a.
        public (bool NeedsMoreInput, string Output, HostAction? Action)? SubmitFromAnyThread(DianaShInterpreter shell, string line)
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
        public IEnumerable<(bool NeedsMoreInput, string Output, HostAction? Action)> DrainPending(DianaShInterpreter shell)
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
        public (bool NeedsMoreInput, string Output, HostAction? Action) SubmitLocked(DianaShInterpreter shell, string line)
        {
            lock (_lock) return shell.Submit(line);
        }

        // Under the same lock as every other path.
        public string[] SnapshotHistory(DianaShInterpreter shell)
        {
            lock (_lock) return new List<string>(shell.History.Entries).ToArray();
        }
    }
}
