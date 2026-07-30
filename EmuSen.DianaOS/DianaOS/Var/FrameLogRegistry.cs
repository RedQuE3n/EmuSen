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
    // One recorded per-frame sample - a frame number and whatever value a
    // registered entry read at that frame's boundary.
    public readonly struct FrameLogSample
    {
        public long FrameCount { get; }
        public long Value { get; }

        public FrameLogSample(long frameCount, long value)
        {
            FrameCount = frameCount;
            Value = value;
        }
    }

    internal sealed class FrameLogEntry
    {
        public int Id;
        public string SpaceName = "";
        public int Address;
        public int Width = 1;
        public readonly List<FrameLogSample> Samples = new();
        public const int MaxStoredSamples = 3600; // ~60 seconds at 60fps - generous without being unbounded
    }

    // Frame-scoped value logging: unlike WatchRegistry (which records
    // whenever a matching memory access actually happens), this samples a
    // fixed set of addresses once per frame regardless of whether they
    // were touched - for tracking a value's evolution over time (a
    // counter, a state machine variable, a position) even when it never
    // triggers a read/write watch because the game just holds it steady
    // between updates, or updates it through a code path this session's
    // watches don't happen to cover.
    //
    // Deliberately does NOT print live the way WatchRegistry does - a
    // watch fires on a comparatively rare event (a specific write/read),
    // so printing every match keeps the existing "play, then grep the
    // console log" workflow useful. A frame log fires 60 times a second by
    // design; printing every sample would flood the console for no
    // benefit. Samples are stored silently and pulled on demand via
    // `framelog show`, the same query-on-demand half of the workflow
    // WatchRegistry's GetEvents already provides.
    //
    // Core-agnostic on purpose, same as WatchRegistry - lives under
    // Shell/, not Cores/Nintendo/Venus - SNES/. A future core's IDebugTarget would
    // own its own instance and feed it from its own per-frame boundary,
    // whatever that core's equivalent of MemoryBus.FrameObserver is.
    public class FrameLogRegistry
    {
        private readonly List<FrameLogEntry> _entries = new();
        private int _nextId = 1;

        public int AddEntry(string spaceName, int address, int width)
        {
            var e = new FrameLogEntry { Id = _nextId++, SpaceName = spaceName, Address = address, Width = width };
            _entries.Add(e);
            return e.Id;
        }

        public bool RemoveEntry(int id)
        {
            return _entries.RemoveAll(e => e.Id == id) > 0;
        }

        public IReadOnlyList<(int Id, string SpaceName, int Address, int Width)> GetEntries()
        {
            return _entries.Select(e => (e.Id, e.SpaceName, e.Address, e.Width)).ToList();
        }

        public IReadOnlyList<FrameLogSample> GetSamples(int id, int maxCount = 20)
        {
            var e = _entries.FirstOrDefault(x => x.Id == id);
            if (e == null) return Array.Empty<FrameLogSample>();
            return e.Samples.Skip(Math.Max(0, e.Samples.Count - maxCount)).ToList();
        }

        public void ClearSamples(int id)
        {
            _entries.FirstOrDefault(x => x.Id == id)?.Samples.Clear();
        }

        // Called once per frame (see MemoryBus.FrameObserver / VenusCore.
        // RunFrame) with a way to read a (space, address, width) value -
        // a delegate rather than an IDebugTarget reference, so this class
        // doesn't need to know that interface exists, matching
        // WatchRegistry's own "generic contextFactory callback" approach.
        public void RecordFrame(long frameCount, Func<string, int, int, long> readValue)
        {
            foreach (var e in _entries)
            {
                long value = readValue(e.SpaceName, e.Address, e.Width);
                e.Samples.Add(new FrameLogSample(frameCount, value));
                if (e.Samples.Count > FrameLogEntry.MaxStoredSamples) e.Samples.RemoveAt(0);
            }
        }
    }
}
