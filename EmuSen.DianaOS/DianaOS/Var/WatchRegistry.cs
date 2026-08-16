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
    // The same three-way split GDB's watch/rwatch/awatch uses; Write stays the default.
    public enum WatchKind
    {
        Write,
        Read,
        Both,
    }

    // One recorded access that matched an active watch.
    public readonly struct DebugWatchEvent
    {
        public long Sequence { get; }
        public WatchKind AccessKind { get; }
        public int Address { get; }
        public byte Value { get; }
        // Free-text, because what counts as useful context varies by core.
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

        // Whole-run totals kept outside the ring, so a summary cannot look complete when it is not.
        public readonly Dictionary<string, long> SiteHits = new();
        public long TotalEvents;
    }

    // The reusable mechanism behind every ad hoc write trace - see EmuSen_Debugging_Tools_Reference_v5.md §3.5.
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

        // Unlike GetEvents, this never loses a site to the ring's eviction.
        public IReadOnlyList<(string Context, long Count)> GetSiteHits(int id)
        {
            var w = _watches.FirstOrDefault(x => x.Id == id);
            if (w == null) return Array.Empty<(string, long)>();
            return w.SiteHits.Select(kv => (kv.Key, kv.Value)).OrderByDescending(s => s.Value).ToList();
        }

        // The gap between the two is what `watch log` cannot show.
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

        // Called for every write, so it must stay cheap when nothing matches - see §3.5.
        public void RecordWrite(string spaceName, int address, byte value, Func<string> contextFactory)
        {
            Record(WatchKind.Write, spaceName, address, value, contextFactory);
        }

        // Same cheap-when-nothing-matches contract as RecordWrite, held tighter - see §3.5.
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

                // Storing always happens; only the live echo respects the master switch - see §3.5.
                if (DianaOSLogging.MasterEnabled)
                {
                    string tag = accessKind == WatchKind.Write ? "W" : "R";
                    Console.WriteLine($"[WATCH #{w.Id}] {tag} {spaceName}:0x{address:X} = 0x{value:X2} ({context})");
                }
            }
        }
    }
}
