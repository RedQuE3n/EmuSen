using System;
using System.Collections.Generic;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // The audio interface: two buffers of samples read from RDRAM and played at the DAC's rate off the one clock - see Mars_Audio.md.
    public sealed class AiInterface
    {
        public const uint DramAddress = 0x00;
        public const uint Length = 0x04;
        public const uint Control = 0x08;
        public const uint Status = 0x0C;
        public const uint DacRate = 0x10;
        public const uint BitRate = 0x14;

        public const uint StatusFull = 0x8000_0001;
        public const uint StatusBusy = 0x4000_0000;
        public const uint StatusEnabled = 0x0200_0000;

        // Two bits that read as one whatever the interface is doing, as the FPGA core builds the register - see Mars_Audio.md §2.
        public const uint StatusAlwaysSet = 0x0110_0000;

        // Past this, undrained samples are dropped oldest first and a stereo pair at a time - see Mars_Audio.md §4.
        public const int MaxBufferedSamples = 128000;

        // What a frontend opens its device at before a game has asked for a rate.
        public const int DefaultSampleRate = 44100;

        // A DAC period shorter than this plays at this, as the FPGA core clamps it - see Mars_Audio.md §2.
        public const uint ShortestPeriod = 0x200;

        public const uint PageSize = 0x2000;

        private const long ProcessorClock = 93_750_000;
        private const uint AddressMask = 0x00FF_FFF8;
        private const uint LengthMask = 0x0003_FFF8;

        private readonly MemoryBus _bus;
        private readonly Queue<short> _samples = new();

        private uint _address;
        private uint _length;
        private uint _nextAddress;
        private uint _nextLength;
        private int _queued;
        private bool _carry;
        private bool _dmaEnabled;
        private uint _dacRate;
        private long _debt;

        public AiInterface(MemoryBus bus) => _bus = bus;

        public int BufferedSamples => _samples.Count;

        // Stereo samples played since power-on, drained or not.
        public long SamplesPlayed { get; private set; }

        // The DAC divides the video clock; before a game sets a rate, the frontend's default stands - see Mars_Audio.md §2.
        public int SampleRate => _dacRate == 0 ? DefaultSampleRate : (int)Math.Round(_bus.Vi.VideoClock / (double)Period);

        private uint Period => Math.Max(_dacRate + 1, ShortestPeriod);

        public uint Read32(uint offset) => (offset & 0x1C) switch
        {
            Status => (_queued > 1 ? StatusFull : 0) | (_queued > 0 ? StatusBusy : 0) | (_dmaEnabled ? StatusEnabled : 0) | StatusAlwaysSet,

            // Every other register reads back what the playing buffer has left, which is how a game paces itself - see §3.
            _ => _length,
        };

        public void Write32(uint offset, uint value)
        {
            switch (offset & 0x1C)
            {
                case DramAddress:
                    if (_queued == 0) _address = value & AddressMask;
                    else if (_queued == 1) _nextAddress = value & AddressMask;
                    break;

                case Length:
                    Queue(value & LengthMask);
                    break;

                case Control:
                    _dmaEnabled = (value & 1) != 0;
                    break;

                case Status:
                    _bus.Mi.Clear(MiInterrupt.AudioInterface);
                    break;

                case DacRate:
                    _dacRate = value & 0x3FFF;
                    break;

                // The bit clock's half period, which the FPGA core ignores too, since the DAC rate alone paces the samples - see §2.
                case BitRate:
                    break;
            }
        }

        // Called from the bus's one counter; nothing moves while no buffer plays or the DMA is off - see Mars_Audio.md §3.
        public void Step(long cycles)
        {
            if (_queued == 0 || !_dmaEnabled)
            {
                _debt = 0;
                return;
            }

            _debt += cycles * _bus.Vi.VideoClock;
            long period = Period * ProcessorClock;

            while (_debt >= period && _queued > 0)
            {
                _debt -= period;
                Play();
            }
        }

        // Destructive, interleaved left then right, and whole pairs only - what ICore.DequeueAudioSamples names.
        public short[] Drain(int maxFrames)
        {
            long wanted = Math.Min((long)maxFrames * 2, _samples.Count);
            wanted -= wanted & 1;
            if (wanted <= 0) return Array.Empty<short>();

            var samples = new short[wanted];
            for (int i = 0; i < wanted; i++) samples[i] = _samples.Dequeue();
            return samples;
        }

        // A snapshot for audiodump; the queue keeps everything.
        public short[] Peek() => _samples.ToArray();

        // A buffer written while none plays begins at once; a second waits its turn; a third is dropped - see Mars_Audio.md §3.
        private void Queue(uint length)
        {
            if (_queued == 0)
            {
                _length = length;
                _queued = 1;
                Begin();
            }
            else if (_queued == 1)
            {
                _nextLength = length;
                _queued = 2;
            }
        }

        // The interrupt marks a buffer beginning, not one ending - see Mars_Audio.md §3.1.
        private void Begin()
        {
            _bus.Mi.Raise(MiInterrupt.AudioInterface);
            if (_length == 0) End();
        }

        private void End()
        {
            if (_queued < 2)
            {
                _queued = 0;
                return;
            }

            _address = _nextAddress;
            _length = _nextLength;
            _queued = 1;
            Begin();
        }

        // One stereo sample, sixteen bits a side and big-endian, left first - see Mars_Audio.md §2.
        private void Play()
        {
            // The carry into the page above lands one fetch late, which is only heard when a buffer ends on a page - see §3.2.
            if (_carry)
            {
                _address = (_address + PageSize) & AddressMask;
                _carry = false;
            }

            byte[] rdram = _bus.Rdram;
            uint at = _address;
            short left = 0, right = 0;

            if (at + 3 < rdram.Length)
            {
                left = (short)((rdram[at] << 8) | rdram[at + 1]);
                right = (short)((rdram[at + 2] << 8) | rdram[at + 3]);
            }

            Enqueue(left, right);
            SamplesPlayed++;

            _address = (_address & ~(PageSize - 1)) | ((_address + 4) & (PageSize - 1));
            if ((_address & (PageSize - 1)) == 0) _carry = true;

            _length -= 4;
            if (_length == 0) End();
        }

        private void Enqueue(short left, short right)
        {
            if (_samples.Count + 2 > MaxBufferedSamples)
            {
                _samples.Dequeue();
                _samples.Dequeue();
            }

            _samples.Enqueue(left);
            _samples.Enqueue(right);
        }
    }
}
