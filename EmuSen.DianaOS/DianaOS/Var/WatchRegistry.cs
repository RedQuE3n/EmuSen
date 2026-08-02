using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // Which kind of access a watch triggers on - the same three-way split
    // real hardware debuggers use (GDB's watch/rwatch/awatch: write-only/
    // read-only/both). Write is the default everywhere a caller doesn't
    // specify one, matching this mechanism's original write-only behavior
    // before read watching existed - existing `watch add` calls (and any
    // saved investigation notes referencing them) keep meaning exactly
    // what they always meant.
    public enum WatchKind
    {
        Write,
        Read,
        Both,
    }

    // One recorded access that matched an active watch - a write, or
    // (once a core wires up an IReadObserver-equivalent hook) a read.
    public readonly struct DebugWatchEvent
    {
        public long Sequence { get; }
        public WatchKind AccessKind { get; }
        public int Address { get; }
        public byte Value { get; }
        // Free-text context (typically "PC=0x00A358") - kept as a string
        // rather than a structured field since what's useful context
        // varies by core (a PC for a CPU write, maybe a scanline+dot for a
        // PPU-driven one later) and this shouldn't need an interface
        // change every time a new kind of context becomes relevant.
        public string Context { get; }

        public DebugWatchEvent(long sequence, WatchKind accessKind, int address, byte value, string context)
        {
            Sequence = sequence;
            AccessKind = accessKind;
            Address = address;
            Value = value;
            Context = context;
        }
    }

    internal sealed class Watch
    {
        public int Id;
        public string SpaceName = "";
        public int StartAddress;
        public int Length;
        public WatchKind Kind = WatchKind.Write;
        public readonly List<DebugWatchEvent> Events = new();
        public const int MaxStoredEvents = 500; // ring-buffer cap - a long session shouldn't grow this unbounded

        // Per-site totals for the whole run, kept outside the ring so
        // `watch summary` cannot report a truncated site list as the
        // complete one - see EmuSen_Debugging_Tools_Reference_v5.md §3.9.
        public readonly Dictionary<string, long> SiteHits = new();
        public long TotalEvents;
    }

    // The reusable mechanism behind every "log writes to this address range"
    // trace this project has built ad hoc so far (CameraRamLogging,
    // MosaicWriteLogging, DmaSourceAddrLogging, the Yoshi WRAM trace...).
    // Generalizes that recurring pattern into one thing: register a watch
    // on a (memory space, address range, access kind), and every matching
    // access gets recorded - both printed live (so the existing "play,
    // then grep the console log" workflow keeps working unchanged) and
    // kept in a bounded per-watch buffer so it's also queryable on demand
    // (via DianaOSInterpreter's `watch` commands today, and eventually
    // a GUI debug window's watch panel, without needing its own new
    // mechanism).
    //
    // Core-agnostic on purpose - lives here rather than under
    // Cores/Nintendo/Venus - SNES/ since nothing about it is SNES-specific. A
    // future core's MemoryBus would own its own instance the same way
    // Venus's does and call RecordWrite/RecordRead from its own read/
    // write path(s).
    public class WatchRegistry
    {
        private readonly List<Watch> _watches = new();
        private int _nextId = 1;
        private long _nextSequence = 1;

        public int AddWatch(string spaceName, int address, int length, WatchKind kind = WatchKind.Write)
        {
            var w = new Watch { Id = _nextId++, SpaceName = spaceName, StartAddress = address, Length = length, Kind = kind };
            _watches.Add(w);
            return w.Id;
        }

        public bool RemoveWatch(int id)
        {
            int removed = _watches.RemoveAll(w => w.Id == id);
            return removed > 0;
        }

        public IReadOnlyList<(int Id, string SpaceName, int StartAddress, int Length, WatchKind Kind)> GetWatches()
        {
            return _watches.Select(w => (w.Id, w.SpaceName, w.StartAddress, w.Length, w.Kind)).ToList();
        }

        public IReadOnlyList<DebugWatchEvent> GetEvents(int id, int maxCount = 100)
        {
            var w = _watches.FirstOrDefault(x => x.Id == id);
            if (w == null) return Array.Empty<DebugWatchEvent>();
            return w.Events.Skip(Math.Max(0, w.Events.Count - maxCount)).ToList();
        }

        // Every access site since the watch was added, with whole-run counts -
        // unlike GetEvents, this never loses a site to the ring's eviction.
        public IReadOnlyList<(string Context, long Count)> GetSiteHits(int id)
        {
            var w = _watches.FirstOrDefault(x => x.Id == id);
            if (w == null) return Array.Empty<(string, long)>();
            return w.SiteHits.Select(kv => (kv.Key, kv.Value)).OrderByDescending(s => s.Value).ToList();
        }

        // How many accesses the watch has seen in total, against how many the
        // event ring still holds - the gap is what `watch log` cannot show.
        public (long Total, int Retained) GetEventCounts(int id)
        {
            var w = _watches.FirstOrDefault(x => x.Id == id);
            return w == null ? (0, 0) : (w.TotalEvents, w.Events.Count);
        }

        public void ClearEvents(int id)
        {
            var w = _watches.FirstOrDefault(x => x.Id == id);
            if (w == null) return;
            w.Events.Clear();
            w.SiteHits.Clear();
            w.TotalEvents = 0;
        }

        // Called from an emulation-side write path (e.g. MemoryBus.Write8)
        // for EVERY write to a given space - deliberately cheap to call
        // when no watch matches (a quick linear scan over however many
        // watches are active, normally a handful at most), so call sites
        // don't need their own "is logging enabled" guard the way the old
        // ad hoc DebugSettings flags did. Prints live (matching every prior
        // trace's behavior - the existing "play, then grep console.log"
        // workflow keeps working with zero changes) in addition to storing
        // the event for on-demand querying later.
        public void RecordWrite(string spaceName, int address, byte value, Func<string> contextFactory)
        {
            Record(WatchKind.Write, spaceName, address, value, contextFactory);
        }

        // Mirror of RecordWrite for reads - called from an emulation-side
        // read path (e.g. MemoryBus.Read8) for every read of a given
        // space. Same "cheap when nothing matches" contract: a read
        // happens far more often than a write (every instruction fetch,
        // every operand read, not just the writes a game actually makes),
        // so this has to stay just as cheap to call when no read watch is
        // registered - the linear scan below costs the same either way,
        // it's the contextFactory() call and event storage that only
        // happen on an actual match.
        public void RecordRead(string spaceName, int address, byte value, Func<string> contextFactory)
        {
            Record(WatchKind.Read, spaceName, address, value, contextFactory);
        }

        private void Record(WatchKind accessKind, string spaceName, int address, byte value, Func<string> contextFactory)
        {
            foreach (var w in _watches)
            {
                if (w.Kind != WatchKind.Both && w.Kind != accessKind) continue;
                if (!string.Equals(w.SpaceName, spaceName, StringComparison.OrdinalIgnoreCase)) continue;
                if (address < w.StartAddress || address >= w.StartAddress + w.Length) continue;

                string context = contextFactory();
                var ev = new DebugWatchEvent(_nextSequence++, accessKind, address, value, context);
                w.Events.Add(ev);
                if (w.Events.Count > Watch.MaxStoredEvents) w.Events.RemoveAt(0);

                w.SiteHits[context] = w.SiteHits.TryGetValue(context, out long hits) ? hits + 1 : 1;
                w.TotalEvents++;

                // Storing the event above always happens regardless of
                // DianaOSLogging.MasterEnabled - watch/log commands querying
                // stored events (`watch log <id>`) should keep working even
                // with logging silenced. Only the live console echo respects
                // the master switch, matching every EmuSen.Debug.DebugSettings
                // *Logging flag's own behavior - previously this printed
                // unconditionally, which meant "turn all logging off" didn't
                // actually silence an active watch, a real gap from every
                // other trace in the project respecting that switch.
                if (DianaOSLogging.MasterEnabled)
                {
                    string tag = accessKind == WatchKind.Write ? "W" : "R";
                    Console.WriteLine($"[WATCH #{w.Id}] {tag} {spaceName}:0x{address:X} = 0x{value:X2} ({context})");
                }
            }
        }
    }
}
