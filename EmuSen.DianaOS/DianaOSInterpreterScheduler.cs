using System;
using System.Collections.Concurrent;
using System.Collections.Generic;

namespace EmuSen.DianaOS
{
    // Lifts the "Diana always live" threading model out of one host
    // (originally EmuSen.Hotaru's GameWindow.axaml.cs - see that file's
    // own header comment for the full design history this was extracted
    // from) into DianaOS itself, so every host gets the same real-time-safe
    // behavior instead of reimplementing it. EmuSen.Mistress9's own console
    // window had none of this until now - it called DianaOSInterpreter.Submit
    // directly from the UI thread with no synchronization at all against
    // its own, separately-running emulation thread.
    //
    // The shape: a read-only line (DianaOSInterpreter.TryGetReadOnlyFastPath)
    // runs immediately, on whichever thread called SubmitFromAnyThread -
    // never touching whatever thread owns the core - because a read-only
    // snapshot read (see EmuSen.Providers.IRealtimeProvider) is safe from
    // anywhere. Anything else queues, to be drained once per frame/tick by
    // DrainPending, called from the thread that actually owns the core -
    // so a mutating command typed at a live console never races RunFrame().
    // A single lock serializes every DianaOSInterpreter.Submit call this
    // scheduler ever makes, from either path, since the interpreter has
    // real mutable state beyond core reads (variables, history, $?) - never
    // held across a blocking read, so the thread that owns the core can
    // never end up waiting on user input through this.
    //
    // Halted is a separate escape hatch for a host's own blocking prompt
    // (F4-style, halted at a breakpoint) - while set, SubmitFromAnyThread
    // forwards lines to TakeHaltedLine's queue instead of the fast-path/
    // pending split above, so a host's prompt loop can pull them one at a
    // time on whichever thread is actually blocked waiting (typically the
    // same thread that owns the core, freeing the UI/reader thread to stay
    // responsive - see Hotaru's own RunDebugPrompt for the pattern this
    // exists to serve).
    public sealed class DianaOSInterpreterScheduler
    {
        private readonly object _lock = new();
        private readonly ConcurrentQueue<string> _pending = new();
        private readonly BlockingCollection<string> _haltedLines = new();
        private volatile bool _halted;

        public bool Halted { get => _halted; set => _halted = value; }

        // Classifies <line> and either runs it immediately (a read-only
        // fast-path line, or anything while Halted - see this class's own
        // header comment) or queues it for DrainPending to run later.
        // Returns the immediate result when one happened, null when the
        // line was queued/forwarded instead - a caller wanting to tell a
        // user "you'll see this once the game ticks" (or "once the halted
        // prompt reads it") can check for null.
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

        // Runs every line queued by SubmitFromAnyThread since the last
        // call, in submission order, one Submit per iteration (not one
        // lock held across the whole batch) - so a fast-path line from
        // another thread can still interleave between drains. Must only
        // be called from the thread that actually owns the core - see
        // this class's own header comment. A caller that needs to stop
        // early (e.g. on a shutdown action) can just break out of the
        // foreach; whatever's left stays queued for the next call.
        public IEnumerable<(bool NeedsMoreInput, string Output, HostAction? Action)> DrainPending(DianaOSInterpreter shell)
        {
            while (_pending.TryDequeue(out string? line))
            {
                (bool NeedsMoreInput, string Output, HostAction? Action) result;
                lock (_lock) result = shell.Submit(line!);
                yield return result;
            }
        }

        // Blocks the calling thread until a line arrives while Halted is
        // set - the other half of the forwarding SubmitFromAnyThread does
        // above. Intended for a host's own blocking prompt loop, called
        // from whichever thread that prompt actually blocks on.
        public string TakeHaltedLine() => _haltedLines.Take();

        // Submits <line> directly, under the same lock every other path
        // through this class uses - for a caller (like a halted prompt)
        // that already decided out-of-band this line should run right now,
        // bypassing the fast-path/pending classification entirely.
        public (bool NeedsMoreInput, string Output, HostAction? Action) SubmitLocked(DianaOSInterpreter shell, string line)
        {
            lock (_lock) return shell.Submit(line);
        }

        // Reads <shell>'s command history under the same lock - so a
        // caller building a "history so far" snapshot (e.g. before a
        // blocking console read) never races a concurrent Submit call
        // appending to it.
        public string[] SnapshotHistory(DianaOSInterpreter shell)
        {
            lock (_lock) return new List<string>(shell.History.Entries).ToArray();
        }
    }
}
