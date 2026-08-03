using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // Per-address read/write/execute tallies for one space - see `man counters`.
    // Core-agnostic: fed from the same observer seams WatchRegistry uses.
    public class AccessCounterRegistry
    {
        private string _space = string.Empty;
        private uint[]? _reads;
        private uint[]? _writes;
        private uint[]? _executes;

        // Cleared by Arm; a read with this still false is uninitialized - see `man counters`.
        private bool[]? _written;
        private uint[]? _uninitializedReads;

        public bool IsArmed { get; private set; }
        public string Space => _space;
        public int Size => _reads?.Length ?? 0;

        // One space at a time - four arrays per byte, see `man counters`.
        public void Arm(string space, int size)
        {
            _space = space;
            _reads = new uint[size];
            _writes = new uint[size];
            _executes = new uint[size];
            _written = new bool[size];
            _uninitializedReads = new uint[size];
            IsArmed = true;
        }

        public void Disarm() => IsArmed = false;

        public void Clear()
        {
            if (_reads == null) return;
            Array.Clear(_reads);
            Array.Clear(_writes!);
            Array.Clear(_executes!);
            Array.Clear(_written!);
            Array.Clear(_uninitializedReads!);
        }

        // Must stay cheap when disarmed - this sits on every memory access.
        public void NoteRead(string space, int address)
        {
            if (!IsArmed || _reads == null) return;
            if (!Matches(space, address)) return;
            _reads[address]++;
            if (!_written![address]) _uninitializedReads![address]++;
        }

        public void NoteWrite(string space, int address)
        {
            if (!IsArmed || _writes == null) return;
            if (!Matches(space, address)) return;
            _writes[address]++;
            _written![address] = true;
        }

        // Counted against the core's PC space - see `man counters`.
        public void NoteExecute(string space, int address)
        {
            if (!IsArmed || _executes == null) return;
            if (!Matches(space, address)) return;
            _executes[address]++;
        }

        private bool Matches(string space, int address)
            => address >= 0 && address < _reads!.Length && string.Equals(space, _space, StringComparison.OrdinalIgnoreCase);

        public (uint Reads, uint Writes, uint Executes, uint UninitializedReads) At(int address)
        {
            if (_reads == null || address < 0 || address >= _reads.Length) return (0, 0, 0, 0);
            return (_reads[address], _writes![address], _executes![address], _uninitializedReads![address]);
        }

        public (long Reads, long Writes, long Executes, long UninitializedReads, int TouchedBytes) Totals(int address, int length)
        {
            if (_reads == null) return (0, 0, 0, 0, 0);
            long reads = 0, writes = 0, executes = 0, uninitialized = 0;
            int touched = 0;
            for (int i = 0; i < length; i++)
            {
                int at = address + i;
                if (at < 0 || at >= _reads.Length) continue;
                reads += _reads[at];
                writes += _writes![at];
                executes += _executes![at];
                uninitialized += _uninitializedReads![at];
                if (_reads[at] != 0 || _writes[at] != 0 || _executes[at] != 0) touched++;
            }
            return (reads, writes, executes, uninitialized, touched);
        }

        public enum SortBy { Reads, Writes, Executes, Uninitialized }

        public IReadOnlyList<(int Address, uint Reads, uint Writes, uint Executes, uint UninitializedReads)> Hottest(SortBy sortBy, int limit)
        {
            if (_reads == null) return Array.Empty<(int, uint, uint, uint, uint)>();
            var rows = new List<(int Address, uint Reads, uint Writes, uint Executes, uint Uninitialized)>();
            for (int at = 0; at < _reads.Length; at++)
            {
                uint key = sortBy switch
                {
                    SortBy.Reads => _reads[at],
                    SortBy.Writes => _writes![at],
                    SortBy.Executes => _executes![at],
                    _ => _uninitializedReads![at],
                };
                if (key == 0) continue;
                rows.Add((at, _reads[at], _writes![at], _executes![at], _uninitializedReads![at]));
            }
            return rows
                .OrderByDescending(r => sortBy switch
                {
                    SortBy.Reads => r.Reads,
                    SortBy.Writes => r.Writes,
                    SortBy.Executes => r.Executes,
                    _ => r.Uninitialized,
                })
                .ThenBy(r => r.Address)
                .Take(limit)
                .ToList();
        }

        // Addresses never touched at all - see `man counters`.
        public IReadOnlyList<int> Untouched(int address, int length, int limit)
        {
            var cold = new List<int>();
            if (_reads == null) return cold;
            for (int i = 0; i < length && cold.Count < limit; i++)
            {
                int at = address + i;
                if (at < 0 || at >= _reads.Length) continue;
                if (_reads[at] == 0 && _writes![at] == 0 && _executes![at] == 0) cold.Add(at);
            }
            return cold;
        }
    }
}
