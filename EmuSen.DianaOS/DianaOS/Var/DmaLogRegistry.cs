using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.DianaOS.DianaOS.Var
{
    public enum DmaTransferKind { General, Hdma }

    // One completed block transfer - see `man dma`.
    public readonly struct DmaTransfer
    {
        public long Sequence { get; }
        public long FrameNumber { get; }
        public int Scanline { get; }
        public int Channel { get; }
        public DmaTransferKind Kind { get; }

        public byte Control { get; }
        public byte DestinationRegister { get; }
        public int SourceAddress { get; }
        public int Length { get; }

        public DmaTransfer(long sequence, long frameNumber, int scanline, int channel, DmaTransferKind kind,
            byte control, byte destinationRegister, int sourceAddress, int length)
        {
            Sequence = sequence;
            FrameNumber = frameNumber;
            Scanline = scanline;
            Channel = channel;
            Kind = kind;
            Control = control;
            DestinationRegister = destinationRegister;
            SourceAddress = sourceAddress;
            Length = length;
        }
    }

    // Whole-run totals for one destination register, not just the retained tail.
    public readonly struct DmaDestinationStat
    {
        public byte DestinationRegister { get; }
        public long Transfers { get; }
        public long Bytes { get; }
        public int ChannelMask { get; }

        public DmaDestinationStat(byte destinationRegister, long transfers, long bytes, int channelMask)
        {
            DestinationRegister = destinationRegister;
            Transfers = transfers;
            Bytes = bytes;
            ChannelMask = channelMask;
        }
    }

    // Per-channel totals, split by kind because the two cost very differently.
    public readonly struct DmaChannelStat
    {
        public int Channel { get; }
        public long GeneralTransfers { get; }
        public long GeneralBytes { get; }
        public long HdmaTransfers { get; }
        public long HdmaBytes { get; }

        public DmaChannelStat(int channel, long generalTransfers, long generalBytes, long hdmaTransfers, long hdmaBytes)
        {
            Channel = channel;
            GeneralTransfers = generalTransfers;
            GeneralBytes = generalBytes;
            HdmaTransfers = hdmaTransfers;
            HdmaBytes = hdmaBytes;
        }
    }

    // Every block transfer a core performs, as a ring plus whole-run tallies - see `man dma`.
    public class DmaLogRegistry
    {
        private const int DefaultCapacity = 4096;
        private const int MaxChannels = 16;
        private const int DestinationCount = 256;

        private DmaTransfer[]? _ring;
        private int _ringNext;
        private long _sequence;

        private readonly long[] _generalTransfers = new long[MaxChannels];
        private readonly long[] _generalBytes = new long[MaxChannels];
        private readonly long[] _hdmaTransfers = new long[MaxChannels];
        private readonly long[] _hdmaBytes = new long[MaxChannels];

        private readonly long[] _destinationTransfers = new long[DestinationCount];
        private readonly long[] _destinationBytes = new long[DestinationCount];
        private readonly int[] _destinationChannels = new int[DestinationCount];

        public bool IsArmed { get; private set; }
        public int Capacity { get; private set; } = DefaultCapacity;
        public long TotalTransfers => _sequence;

        public long TotalBytes { get; private set; }

        // Whichever frame the last transfer landed in, so `dma stats` can report a per-frame rate.
        public long FirstFrame { get; private set; } = -1;
        public long LastFrame { get; private set; } = -1;

        public Func<long>? FrameNumberProvider { get; set; }
        public Func<int>? ScanlineProvider { get; set; }

        public void Arm(int capacity = DefaultCapacity)
        {
            Capacity = Math.Max(16, capacity);
            _ring = new DmaTransfer[Capacity];
            IsArmed = true;
            ClearCounters();
        }

        public void Disarm() => IsArmed = false;

        public void Clear()
        {
            if (_ring != null) Array.Clear(_ring);
            ClearCounters();
        }

        private void ClearCounters()
        {
            _ringNext = 0;
            _sequence = 0;
            TotalBytes = 0;
            FirstFrame = -1;
            LastFrame = -1;
            Array.Clear(_generalTransfers);
            Array.Clear(_generalBytes);
            Array.Clear(_hdmaTransfers);
            Array.Clear(_hdmaBytes);
            Array.Clear(_destinationTransfers);
            Array.Clear(_destinationBytes);
            Array.Clear(_destinationChannels);
        }

        // Must stay cheap when disarmed - that is the whole cost on a normal run.
        public void Note(int channel, DmaTransferKind kind, byte control, byte destinationRegister, int sourceAddress, int length)
        {
            if (!IsArmed || _ring == null) return;
            if ((uint)channel >= MaxChannels) return;

            _sequence++;
            TotalBytes += length;

            if (kind == DmaTransferKind.General) { _generalTransfers[channel]++; _generalBytes[channel] += length; }
            else { _hdmaTransfers[channel]++; _hdmaBytes[channel] += length; }

            _destinationTransfers[destinationRegister]++;
            _destinationBytes[destinationRegister] += length;
            _destinationChannels[destinationRegister] |= 1 << channel;

            long frame = FrameNumberProvider?.Invoke() ?? 0;
            int scanline = ScanlineProvider?.Invoke() ?? -1;
            if (FirstFrame < 0) FirstFrame = frame;
            LastFrame = frame;

            _ring[_ringNext] = new DmaTransfer(_sequence, frame, scanline, channel, kind, control, destinationRegister, sourceAddress, length);
            _ringNext = (_ringNext + 1) % Capacity;
        }

        // Oldest-first, at most <count> entries.
        public IReadOnlyList<DmaTransfer> Tail(int count)
        {
            if (_ring == null) return Array.Empty<DmaTransfer>();
            int retained = (int)Math.Min(_sequence, Capacity);
            int take = Math.Min(count, retained);
            var result = new List<DmaTransfer>(take);
            for (int i = retained - take; i < retained; i++)
            {
                result.Add(_ring[(_ringNext - retained + i + Capacity * 2) % Capacity]);
            }
            return result;
        }

        // Only channels that actually moved something, busiest first.
        public IReadOnlyList<DmaChannelStat> ChannelStats()
        {
            var result = new List<DmaChannelStat>();
            for (int i = 0; i < MaxChannels; i++)
            {
                if (_generalTransfers[i] + _hdmaTransfers[i] == 0) continue;
                result.Add(new DmaChannelStat(i, _generalTransfers[i], _generalBytes[i], _hdmaTransfers[i], _hdmaBytes[i]));
            }
            return result.OrderByDescending(s => s.GeneralBytes + s.HdmaBytes).ToList();
        }

        // Which B-bus register the bandwidth actually went to, biggest first.
        public IReadOnlyList<DmaDestinationStat> DestinationStats()
        {
            var result = new List<DmaDestinationStat>();
            for (int i = 0; i < DestinationCount; i++)
            {
                if (_destinationTransfers[i] == 0) continue;
                result.Add(new DmaDestinationStat((byte)i, _destinationTransfers[i], _destinationBytes[i], _destinationChannels[i]));
            }
            return result.OrderByDescending(s => s.Bytes).ToList();
        }
    }
}
