using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Debug
{
    // One recorded write that matched an active watch.
    public readonly struct DebugWatchEvent
    {
        public long Sequence { get; }
        public int Address { get; }
        public byte Value { get; }
        // Free-text context (typically "PC=0x00A358") - kept as a string
        // rather than a structured field since what's useful context
        // varies by core (a PC for a CPU write, maybe a scanline+dot for a
        // PPU-driven one later) and this shouldn't need an interface
        // change every time a new kind of context becomes relevant.
        public string Context { get; }

        public DebugWatchEvent(long sequence, int address, byte value, string context)
        {
            Sequence = sequence;
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
        public readonly List<DebugWatchEvent> Events = new();
        public const int MaxStoredEvents = 500; // ring-buffer cap - a long session shouldn't grow this unbounded
    }

    // The reusable mechanism behind every "log writes to this address range"
    // trace this project has built ad hoc so far (CameraRamLogging,
    // MosaicWriteLogging, DmaSourceAddrLogging, the Yoshi WRAM trace...).
    // Generalizes that recurring pattern into one thing: register a watch
    // on a (memory space, address range), and every matching write gets
    // recorded - both printed live (so the existing "play, then grep the
    // console log" workflow keeps working unchanged) and kept in a bounded
    // per-watch buffer so it's also queryable on demand (via
    // DebugCommandProcessor's `watch` commands today, and eventually a GUI
    // debug window's watch panel, without needing its own new mechanism).
    //
    // Core-agnostic on purpose - lives here rather than under
    // Cores/Nintendo/Venus - SNES/ since nothing about it is SNES-specific. A
    // future core's MemoryBus would own its own instance the same way
    // Venus's does and call RecordWrite from its own write path(s).
    public class WatchRegistry
    {
        private readonly List<Watch> _watches = new();
        private int _nextId = 1;
        private long _nextSequence = 1;

        public int AddWatch(string spaceName, int address, int length)
        {
            var w = new Watch { Id = _nextId++, SpaceName = spaceName, StartAddress = address, Length = length };
            _watches.Add(w);
            return w.Id;
        }

        public bool RemoveWatch(int id)
        {
            int removed = _watches.RemoveAll(w => w.Id == id);
            return removed > 0;
        }

        public IReadOnlyList<(int Id, string SpaceName, int StartAddress, int Length)> GetWatches()
        {
            return _watches.Select(w => (w.Id, w.SpaceName, w.StartAddress, w.Length)).ToList();
        }

        public IReadOnlyList<DebugWatchEvent> GetEvents(int id, int maxCount = 100)
        {
            var w = _watches.FirstOrDefault(x => x.Id == id);
            if (w == null) return Array.Empty<DebugWatchEvent>();
            return w.Events.Skip(Math.Max(0, w.Events.Count - maxCount)).ToList();
        }

        public void ClearEvents(int id)
        {
            _watches.FirstOrDefault(x => x.Id == id)?.Events.Clear();
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
            foreach (var w in _watches)
            {
                if (!string.Equals(w.SpaceName, spaceName, StringComparison.OrdinalIgnoreCase)) continue;
                if (address < w.StartAddress || address >= w.StartAddress + w.Length) continue;

                string context = contextFactory();
                var ev = new DebugWatchEvent(_nextSequence++, address, value, context);
                w.Events.Add(ev);
                if (w.Events.Count > Watch.MaxStoredEvents) w.Events.RemoveAt(0);

                Console.WriteLine($"[WATCH #{w.Id}] {spaceName}:0x{address:X} = 0x{value:X2} ({context})");
            }
        }
    }
}
