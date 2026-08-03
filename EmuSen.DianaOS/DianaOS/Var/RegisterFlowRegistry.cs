using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // One access to a watched register window - see `man copflow`.
    public readonly struct RegisterFlowEntry
    {
        public long Sequence { get; }
        public long FrameNumber { get; }
        public int Register { get; }
        public byte Value { get; }
        public bool IsWrite { get; }
        public string Actor { get; }

        public RegisterFlowEntry(long sequence, long frameNumber, int register, byte value, bool isWrite, string actor)
        {
            Sequence = sequence;
            FrameNumber = frameNumber;
            Register = register;
            Value = value;
            IsWrite = isWrite;
            Actor = actor;
        }
    }

    // Whole-run tallies for one register, not just the retained tail.
    public readonly struct RegisterFlowStat
    {
        public int Register { get; }
        public long Reads { get; }
        public long Writes { get; }
        public int DistinctValues { get; }
        public byte LastValue { get; }

        public RegisterFlowStat(int register, long reads, long writes, int distinctValues, byte lastValue)
        {
            Register = register;
            Reads = reads;
            Writes = writes;
            DistinctValues = distinctValues;
            LastValue = lastValue;
        }
    }

    // Traffic across a coprocessor's register window, plus poll-run detection
    // for handshakes that never complete - see `man copflow`.
    public class RegisterFlowRegistry
    {
        private const int DefaultCapacity = 4096;

        private long[]? _reads;
        private long[]? _writes;
        private HashSet<byte>[]? _values;
        private byte[]? _last;

        private RegisterFlowEntry[]? _ring;
        private int _ringNext;
        private long _sequence;

        private int _pollRegister = -1;
        private byte _pollValue;
        private long _pollRun;

        public bool IsArmed { get; private set; }
        public string WindowName { get; private set; } = "";
        public int WindowSize { get; private set; }
        public int Capacity { get; private set; } = DefaultCapacity;
        public long TotalAccesses => _sequence;

        public long LongestPollRun { get; private set; }
        public int LongestPollRegister { get; private set; } = -1;
        public byte LongestPollValue { get; private set; }

        // The run in progress, which is what matters while halted.
        public long CurrentPollRun => _pollRun;
        public int CurrentPollRegister => _pollRegister;

        public Func<long>? FrameNumberProvider { get; set; }

        public void Arm(string windowName, int windowSize, int capacity = DefaultCapacity)
        {
            WindowName = windowName;
            WindowSize = Math.Max(1, windowSize);
            Capacity = Math.Max(16, capacity);
            _reads = new long[WindowSize];
            _writes = new long[WindowSize];
            _values = new HashSet<byte>[WindowSize];
            _last = new byte[WindowSize];
            _ring = new RegisterFlowEntry[Capacity];
            _ringNext = 0;
            _sequence = 0;
            ResetPollRun();
            LongestPollRun = 0;
            LongestPollRegister = -1;
            IsArmed = true;
        }

        public void Disarm() => IsArmed = false;

        public void Clear()
        {
            if (!IsArmed) return;
            Arm(WindowName, WindowSize, Capacity);
        }

        // Must stay cheap when disarmed - that is the whole cost on a normal run.
        public void Note(int register, byte value, bool isWrite, string actor)
        {
            if (!IsArmed || _ring == null) return;
            if ((uint)register >= (uint)WindowSize) return;

            _sequence++;
            if (isWrite) _writes![register]++;
            else _reads![register]++;

            (_values![register] ??= new HashSet<byte>()).Add(value);
            _last![register] = value;

            long frame = FrameNumberProvider?.Invoke() ?? 0;
            _ring[_ringNext] = new RegisterFlowEntry(_sequence, frame, register, value, isWrite, actor);
            _ringNext = (_ringNext + 1) % Capacity;

            TrackPollRun(register, value, isWrite);
        }

        // A write by either side is progress, so it always breaks the run.
        private void TrackPollRun(int register, byte value, bool isWrite)
        {
            if (isWrite) { ResetPollRun(); return; }

            bool continues = register == _pollRegister && value == _pollValue;
            _pollRun = continues ? _pollRun + 1 : 1;
            _pollRegister = register;
            _pollValue = value;

            // A one-read run still counts, or a never-repeating window reports nothing.
            if (_pollRun <= LongestPollRun) return;
            LongestPollRun = _pollRun;
            LongestPollRegister = register;
            LongestPollValue = value;
        }

        private void ResetPollRun()
        {
            _pollRegister = -1;
            _pollRun = 0;
        }

        // Oldest-first, at most <count> entries.
        public IReadOnlyList<RegisterFlowEntry> Tail(int count)
        {
            if (_ring == null) return Array.Empty<RegisterFlowEntry>();
            int retained = (int)Math.Min(_sequence, Capacity);
            int take = Math.Min(count, retained);
            var result = new List<RegisterFlowEntry>(take);
            for (int i = retained - take; i < retained; i++)
            {
                result.Add(_ring[(_ringNext - retained + i + Capacity * 2) % Capacity]);
            }
            return result;
        }

        // Only registers actually touched, busiest first.
        public IReadOnlyList<RegisterFlowStat> Stats()
        {
            if (_reads == null) return Array.Empty<RegisterFlowStat>();
            var result = new List<RegisterFlowStat>();
            for (int i = 0; i < WindowSize; i++)
            {
                if (_reads[i] + _writes![i] == 0) continue;
                result.Add(new RegisterFlowStat(i, _reads[i], _writes[i], _values![i]?.Count ?? 0, _last![i]));
            }
            return result.OrderByDescending(s => s.Reads + s.Writes).ToList();
        }
    }
}
