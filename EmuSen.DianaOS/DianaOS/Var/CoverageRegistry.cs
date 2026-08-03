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
    // "Did control flow ever reach this address", whole-run - see `man cov`.
    public class CoverageRegistry
    {
        private const int AddressSpace = 0x1000000;
        private const int BitmapBytes = AddressSpace / 8;

        // Serialized maps carry this so a stale file fails loudly - see `man cov`.
        private static readonly byte[] FileMagic = { (byte)'E', (byte)'M', (byte)'C', (byte)'V', 1 };

        // One bit per 24-bit address, allocated only once armed - 2MB, which
        // is why this is not on by default.
        private byte[]? _seen;
        private long _instructionsRecorded;

        // A copy of _seen taken by Mark(), so NewSince can subtract it - see `man cov`.
        private byte[]? _mark;

        // Call targets, with how many times each was entered - see `man cov`.
        private readonly Dictionary<int, EntryPoint> _entryPoints = new();

        private struct EntryPoint
        {
            public CallFrameKind Kind;
            public long Entries;
        }

        public bool IsArmed { get; private set; }

        public long InstructionsRecorded => _instructionsRecorded;

        public bool HasMark => _mark != null;

        public int EntryPointCount => _entryPoints.Count;

        public void Arm()
        {
            _seen ??= new byte[BitmapBytes];
            IsArmed = true;
        }

        public void Disarm() => IsArmed = false;

        public void Clear()
        {
            if (_seen != null) Array.Clear(_seen);
            _instructionsRecorded = 0;
            _mark = null;
            _entryPoints.Clear();
        }

        // Called once per instruction, before it executes. Must stay cheap
        // when disarmed - that is the whole cost this imposes on a normal run.
        public void Record(int address)
        {
            if (!IsArmed || _seen == null) return;
            int index = address & 0xFFFFFF;
            _seen[index >> 3] |= (byte)(1 << (index & 7));
            _instructionsRecorded++;
        }

        // Called from the call-stack seam for every call/interrupt taken - see `man cov`.
        public void RecordEntryPoint(int address, CallFrameKind kind)
        {
            if (!IsArmed) return;
            int index = address & 0xFFFFFF;
            if (_entryPoints.TryGetValue(index, out var existing))
            {
                existing.Entries++;
                _entryPoints[index] = existing;
                return;
            }
            _entryPoints[index] = new EntryPoint { Kind = kind, Entries = 1 };
        }

        public bool WasExecuted(int address)
        {
            if (_seen == null) return false;
            int index = address & 0xFFFFFF;
            return (_seen[index >> 3] & (1 << (index & 7))) != 0;
        }

        public int CountExecuted(int address, int length)
        {
            int hits = 0;
            for (int i = 0; i < length; i++) if (WasExecuted(address + i)) hits++;
            return hits;
        }

        // The executed addresses in a range, in order - the answer to "which
        // part of this routine ran", not just "how much of it".
        public IReadOnlyList<int> ExecutedAddresses(int address, int length, int limit)
        {
            var hits = new List<int>();
            for (int i = 0; i < length && hits.Count < limit; i++)
            {
                if (WasExecuted(address + i)) hits.Add(address + i);
            }
            return hits;
        }

        // Freezes what has run so far, so NewSince can answer "and what ran after that".
        public void Mark()
        {
            if (_seen == null) { _mark = new byte[BitmapBytes]; return; }
            _mark = (byte[])_seen.Clone();
        }

        public void ClearMark() => _mark = null;

        private bool WasMarked(int address)
        {
            if (_mark == null) return false;
            int index = address & 0xFFFFFF;
            return (_mark[index >> 3] & (1 << (index & 7))) != 0;
        }

        // Addresses that have run since Mark() and had not run before it - see `man cov`.
        public IReadOnlyList<int> NewSinceMark(int address, int length, int limit)
        {
            var fresh = new List<int>();
            if (_mark == null) return fresh;
            for (int i = 0; i < length && fresh.Count < limit; i++)
            {
                int at = address + i;
                if (WasExecuted(at) && !WasMarked(at)) fresh.Add(at);
            }
            return fresh;
        }

        public int CountNewSinceMark(int address, int length)
        {
            if (_mark == null) return 0;
            int fresh = 0;
            for (int i = 0; i < length; i++)
            {
                int at = address + i;
                if (WasExecuted(at) && !WasMarked(at)) fresh++;
            }
            return fresh;
        }

        // Discovered routines, most-entered first - see `man cov`.
        public IReadOnlyList<(int Address, CallFrameKind Kind, long Entries)> EntryPoints(int limit)
            => _entryPoints
                .Select(e => (Address: e.Key, e.Value.Kind, e.Value.Entries))
                .OrderByDescending(e => e.Entries)
                .ThenBy(e => e.Address)
                .Take(limit)
                .ToList();

        // Every discovered routine in a range, address order - what an auto-labeller wants.
        public IReadOnlyList<int> EntryPointsIn(int address, int length)
            => _entryPoints.Keys
                .Where(a => a >= address && a < address + length)
                .OrderBy(a => a)
                .ToList();

        // Executed-byte totals over a range, for "how much of this bank ever ran".
        public (int Executed, int Total, int NewSinceMark) Statistics(int address, int length)
            => (CountExecuted(address, length), length, CountNewSinceMark(address, length));

        // The bitmap plus its entry points, for `cov save` - see `man cov`.
        public byte[] Export()
        {
            var buffer = new byte[FileMagic.Length + 4 + (_entryPoints.Count * 13) + BitmapBytes];
            int at = 0;
            Array.Copy(FileMagic, 0, buffer, at, FileMagic.Length);
            at += FileMagic.Length;

            BitConverter.TryWriteBytes(buffer.AsSpan(at), _entryPoints.Count);
            at += 4;
            foreach (var entry in _entryPoints)
            {
                BitConverter.TryWriteBytes(buffer.AsSpan(at), entry.Key);
                buffer[at + 4] = (byte)entry.Value.Kind;
                BitConverter.TryWriteBytes(buffer.AsSpan(at + 5), entry.Value.Entries);
                at += 13;
            }

            if (_seen != null) Array.Copy(_seen, 0, buffer, at, BitmapBytes);
            return buffer;
        }

        // Merges into whatever is already recorded, so maps from several runs accumulate.
        public bool Import(byte[] data)
        {
            if (data.Length < FileMagic.Length + 4) return false;
            for (int i = 0; i < FileMagic.Length; i++) if (data[i] != FileMagic[i]) return false;

            int at = FileMagic.Length;
            int count = BitConverter.ToInt32(data, at);
            at += 4;
            if (count < 0 || data.Length < at + (count * 13) + BitmapBytes) return false;

            for (int i = 0; i < count; i++)
            {
                int address = BitConverter.ToInt32(data, at);
                var kind = (CallFrameKind)data[at + 4];
                long entries = BitConverter.ToInt64(data, at + 5);
                at += 13;

                if (_entryPoints.TryGetValue(address, out var existing))
                {
                    existing.Entries += entries;
                    _entryPoints[address] = existing;
                    continue;
                }
                _entryPoints[address] = new EntryPoint { Kind = kind, Entries = entries };
            }

            _seen ??= new byte[BitmapBytes];
            for (int i = 0; i < BitmapBytes; i++) _seen[i] |= data[at + i];
            return true;
        }
    }
}
