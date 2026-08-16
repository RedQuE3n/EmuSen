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
    // One per-frame sample: a frame number and what a registered entry read.
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

    // Sampled every frame regardless of access, and stored silently rather than printed - see §3.13.
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

        // A delegate rather than an IDebugTarget, so this class need not know that interface.
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
