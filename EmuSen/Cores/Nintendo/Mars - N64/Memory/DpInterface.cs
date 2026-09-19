using System;
using System.Threading;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // The display processor's command registers, the stream they hand over, and the processor it feeds - see Mars_Rdp.md §2.
    public sealed class DpInterface
    {
        public const uint Start = 0x00;
        public const uint End = 0x04;
        public const uint Current = 0x08;
        public const uint Status = 0x0C;

        public const uint StatusXbus = 0x001;
        public const uint StatusFreeze = 0x002;
        public const uint StatusStartGclk = 0x008;
        public const uint StatusPipeBusy = 0x020;
        public const uint StatusBufferReady = 0x080;
        public const uint StatusStartValid = 0x400;

        // Twenty-four bits, word-aligned, for all three addresses - see §2.1.
        public const uint AddressMask = 0x00FF_FFF8;

        public readonly Rdp.Rdp Processor;

        [EmuSen.Common.SkipInState] private readonly MemoryBus _bus;

        private uint _start;
        private uint _end;
        private uint _current;
        private bool _startValid;
        private bool _xbus;
        private bool _freeze;
        private bool _running;

        // The words handed over but not yet run, and the two counts that say how far each side has come - see §2.6.
        [EmuSen.Common.SkipInState] private readonly ulong[] _ring = new ulong[1 << 16];
        [EmuSen.Common.SkipInState] private long _issued;
        [EmuSen.Common.SkipInState] private long _completed;
        [EmuSen.Common.SkipInState] private int _draining;
        [EmuSen.Common.SkipInState] private Exception? _fault;
        [EmuSen.Common.SkipInState] private bool _threaded;

        // For each page of RDRAM, the count of words that must have run before anyone else touches it; zero when nothing is due - see §2.6.
        [EmuSen.Common.SkipInState] private readonly long[] _marks;

        // The mark an image's pages take: everything handed over so far, however much that comes to be - see §2.6.
        private const long Idle = long.MaxValue;

        // The interface's own gathering of the stream, so the full sync is answered when the word arrives - see §2.6.
        [EmuSen.Common.SkipInState] private int _shadowTaken;
        [EmuSen.Common.SkipInState] private ulong _shadowFirst;
        [EmuSen.Common.SkipInState] private uint _colorImage, _depthImage, _textureImage;
        [EmuSen.Common.SkipInState] private int _colorWidth, _colorBytes, _textureWidth, _textureSize, _scissorTop, _scissorBottom;
        [EmuSen.Common.SkipInState] private bool _drawn;

        // The image extents a batch marked idle, downgraded to the batch's own count when it ends - see §2.6.
        [EmuSen.Common.SkipInState] private readonly (long From, long Count)[] _idleRanges = new (long, long)[16];
        [EmuSen.Common.SkipInState] private int _idleRangeCount;

        // What the thread did: words run and the time in them, drains started and the delay from a kick to its start - see Mars_Performance.md §28.
        [EmuSen.Common.SkipInState] public long DrainWords, DrainTicks, DrainStarts, KickTicks;
        [EmuSen.Common.SkipInState] private long _kickedAt;

        // Set by the thread before each word it runs, so a touch can be checked against the marks - see §2.6.
        [EmuSen.Common.SkipInState] private long _runningWord;

        // How often anyone waited for the thread, and for how long, by the kind of waiter - see Mars_Performance.md §28.
        [EmuSen.Common.SkipInState] public long PageWaits, PageWaitTicks, RangeWaits, RangeWaitTicks, Joins, JoinTicks;
        [EmuSen.Common.SkipInState] public readonly long[] WaitsPerPage;
        [EmuSen.Common.SkipInState] public readonly long[] WaitsPerSite = new long[12], TicksPerSite = new long[12];
        [EmuSen.Common.SkipInState] public readonly (int Site, uint Page, bool Idle, long Ticks, long Lag, uint Color, uint Depth, uint Texture)[] WaitLog = new (int, uint, bool, long, long, uint, uint, uint)[64];
        [EmuSen.Common.SkipInState] public int WaitsLogged;
        [EmuSen.Common.SkipInState] private bool _taking;
        [EmuSen.Common.SkipInState] private int _waiting;

        public DpInterface(MemoryBus bus)
        {
            _bus = bus;
            _marks = new long[bus.Rdram.Length >> 12];
            WaitsPerPage = new long[_marks.Length];
            Processor = new Rdp.Rdp(bus);
        }

        // The buffer is never full, because every word handed over has already been taken - see §2.2.
        public uint StatusWord =>
            (_xbus ? StatusXbus : 0)
            | (_freeze ? StatusFreeze : 0)
            | (_running ? StatusStartGclk | StatusPipeBusy : 0)
            | StatusBufferReady
            | (_startValid ? StatusStartValid : 0);

        public uint Read32(uint offset)
        {
            return (offset & 0x1C) switch
            {
                Start => _start,
                End => _end,
                Current => _current,
                Status => StatusWord,
                _ => 0,
            };
        }

        public void Write32(uint offset, uint value)
        {
            switch (offset & 0x1C)
            {
                case Start:
                    WriteStart(value);
                    break;

                case End:
                    WriteEnd(value);
                    break;

                case Status:
                    WriteStatus(value);
                    break;
            }
        }

        // A second start before an end is refused, so a list cannot be moved under the one being handed over - see §2.1.
        private void WriteStart(uint value)
        {
            if (_startValid) return;

            _start = value & AddressMask;
            _startValid = true;
        }

        private void WriteEnd(uint value)
        {
            _end = value & AddressMask;

            if (_startValid)
            {
                _current = _start;
                _startValid = false;
            }

            if (!_freeze) _running = true;

            Take();
        }

        // Only the four bits the corpus measures; a pair with both asserted is no request, by analogy with the RSP - see §2.4.
        private void WriteStatus(uint value)
        {
            if ((value & 3) == 1) _xbus = false;
            if ((value & 3) == 2) _xbus = true;

            if ((value & 0xC) == 4)
            {
                _freeze = false;
                Take();
            }

            if ((value & 0xC) == 8) _freeze = true;
        }

        // Everything up to end is taken at once; an address compare, so a list may run past data memory's end - see §2.3.
        private void Take()
        {
            _bus.Written++;

            if (_threaded)
            {
                TakeOntoThread();
                return;
            }

            while (!_freeze && _current < _end)
            {
                ulong word = _xbus ? ReadDataMemory(_current) : _bus.Read64(_current);
                _current += 8;

                if (Processor.Accept(word)) FullSync();
            }
        }

        private void FullSync()
        {
            _running = false;
            _bus.Mi.Raise(MiInterrupt.DisplayProcessor);
        }

        private ulong ReadDataMemory(uint address)
        {
            ulong word = 0;
            for (uint i = 0; i < 8; i++) word = (word << 8) | _bus.SpDmem[(address + i) & (MemoryMap.SpMemSize - 1)];

            return word;
        }

        // The list runs on a pool thread while the machine goes on; every other touch of a page it marks waits - see §2.6.
        public bool Threaded
        {
            get => _threaded;
            set
            {
                Join();
                _threaded = value;
                if (value) RefreshShadow();
            }
        }

        // The marks, for the bus and the processor's direct paths to test before a touch - see §2.6.
        public long[] Marks => _marks;

        public long MarkFor(uint physical) => _marks[physical >> 12];

        // The same words, read now as the immediate path reads them; each is marked for, then published, so a wait on a mark can always end - see §2.6.
        private void TakeOntoThread()
        {
            long tail = _issued;
            _drawn = false;
            _taking = true;

            while (!_freeze && _current < _end)
            {
                ulong word = _xbus ? ReadDataMemory(_current) : _bus.Read64(_current);
                _current += 8;

                if (tail - Volatile.Read(ref _completed) >= _ring.Length) MakeRoom(tail);
                _ring[(int)(tail & (_ring.Length - 1))] = word;
                tail++;

                bool sync = Shadow(word, tail);
                Volatile.Write(ref _issued, tail);
                Kick();

                if (sync) FullSync();
            }

            _taking = false;

            if (_idleRangeCount <= _idleRanges.Length) for (int i = 0; i < _idleRangeCount; i++) Mark(_idleRanges[i].From, _idleRanges[i].Count, tail);
            _idleRangeCount = 0;
        }

        // A thread from the pool drains what is published, unless one already is - see §2.6.
        private void Kick()
        {
            if (Volatile.Read(ref _draining) == 0 && Interlocked.CompareExchange(ref _draining, 1, 0) == 0)
            {
                _kickedAt = System.Diagnostics.Stopwatch.GetTimestamp();
                ThreadPool.UnsafeQueueUserWorkItem(static s => ((DpInterface)s!).Drain(), this);
            }
        }

        // A word with no room left waits for the thread, which needs nothing from here to make some - see §2.6.
        private void MakeRoom(long tail)
        {
            SpinWait spin = default;
            while (tail - Volatile.Read(ref _completed) >= _ring.Length)
            {
                Rethrow();
                spin.SpinOnce(-1);
            }
        }

        // The interface gathers as the processor does, and remembers the images and the scissor a draw will reach - see §2.6.
        private bool Shadow(ulong word, long afterThisWord)
        {
            if (_shadowTaken == 0) _shadowFirst = word;
            _shadowTaken++;

            uint id = Rdp.Rdp.Id(_shadowFirst);
            if (_shadowTaken < Rdp.Rdp.Length(id)) return false;

            _shadowTaken = 0;
            ulong first = _shadowFirst;

            switch (id)
            {
                case >= 0x08 and <= 0x0F:
                case Rdp.Rdp.TextureRectangle or Rdp.Rdp.TextureRectangleFlipped:
                case Rdp.Rdp.FillRectangle:
                    if (!_drawn) MarkImages();
                    return false;

                case Rdp.Rdp.SyncFull:
                    return true;

                case Rdp.Rdp.SetColorImage:
                    _drawn = false;
                    _colorBytes = (int)((first >> 51) & 3) switch { 1 => 1, 2 => 2, 3 => 4, _ => 1 };
                    _colorWidth = (int)((first >> 32) & 0x3FF) + 1;
                    _colorImage = (uint)first & 0x00FF_FFFF;
                    return false;

                case Rdp.Rdp.SetMaskImage:
                    _drawn = false;
                    _depthImage = (uint)first & 0x00FF_FFFF;
                    return false;

                case Rdp.Rdp.SetScissor:
                    _drawn = false;
                    _scissorTop = (int)((first >> 32) & 0xFFF) >> 2;
                    _scissorBottom = (int)(first & 0xFFF) >> 2;
                    return false;

                case Rdp.Rdp.SetTextureImage:
                    _textureSize = (int)(first >> 51) & 3;
                    _textureWidth = (int)((first >> 32) & 0x3FF) + 1;
                    _textureImage = (uint)first & 0x00FF_FFFF;
                    return false;

                case Rdp.Rdp.LoadTile or Rdp.Rdp.LoadBlock or Rdp.Rdp.LoadPalette:
                    MarkLoad(first, id == Rdp.Rdp.LoadBlock, afterThisWord);
                    return false;

                default:
                    return false;
            }
        }

        // Every row a draw may reach in the colour image and in the depth image, from the scissor's top to its bottom, marked idle once a batch - see §2.6.
        private void MarkImages()
        {
            _drawn = true;
            const long after = Idle;

            long rows = _scissorBottom - _scissorTop + 1;
            if (rows <= 0) rows = 1;
            long width = _colorWidth;

            MarkIdle(_colorImage + (long)_scissorTop * width * _colorBytes, rows * width * _colorBytes);
            MarkIdle(_depthImage + (long)_scissorTop * width * 2, rows * width * 2);
        }

        // The rows a load reads from the texture image, from its first line to its last and the columns it names - see §2.6.
        private void MarkLoad(ulong word, bool block, long after)
        {
            int sl = (int)(word >> 44) & 0xFFF, tl = (int)(word >> 32) & 0xFFF, sh = (int)(word >> 12) & 0xFFF, th = (int)word & 0xFFF;
            long bits = 4 << _textureSize;
            long rowBytes = (_textureWidth * bits + 7) / 8;

            // A block is one linear run from its row and column; a tile is its rows, from its first column to its last, each read in windows of sixteen bytes - see §2.6.
            long from, to;
            if (block)
            {
                long left = (sl << 20) >> 20;
                from = _textureImage + (tl & 0x3FF) * rowBytes + left * bits / 8 - 16;
                to = _textureImage + (tl & 0x3FF) * rowBytes + ((long)sh + 1) * bits / 8 + 32;
            }
            else
            {
                from = _textureImage + (tl >> 2) * rowBytes + (sl >> 2) * bits / 8 - 16;
                to = _textureImage + (th >> 2) * rowBytes + (((long)sh >> 2) + 1) * bits / 8 + 32;
            }

            Mark(from, Math.Max(to - from, 1), after);
        }

        // Idle until the batch ends, then the batch's count; a batch changing images more often than the list holds keeps them idle - see §2.6.
        private void MarkIdle(long from, long count)
        {
            Mark(from, count, Idle);
            if (_idleRangeCount < _idleRanges.Length) _idleRanges[_idleRangeCount++] = (from, count);
            else _idleRangeCount = _idleRanges.Length + 1;
        }

        private void Mark(long from, long count, long after)
        {
            long length = _bus.Rdram.Length;
            long first = Math.Clamp(from, 0, length) >> 12;
            long last = (Math.Clamp(from + count, 0, length) - 1) >> 12;

            for (long page = first; page <= last; page++) _marks[page] = after;
        }

        // Runs what was published, and stays a moment for more before handing the pool its thread back - see §2.6.
        private void Drain()
        {
            DrainStarts++;
            KickTicks += System.Diagnostics.Stopwatch.GetTimestamp() - _kickedAt;

            try
            {
                while (true)
                {
                    long issued = Volatile.Read(ref _issued);
                    long completed = _completed;
                    long started = System.Diagnostics.Stopwatch.GetTimestamp();
                    long before = completed;

                    while (completed < issued)
                    {
                        _runningWord = completed + 1;
                        Processor.Accept(_ring[(int)(completed & (_ring.Length - 1))]);
                        completed++;
                        Volatile.Write(ref _completed, completed);
                    }

                    DrainWords += completed - before;
                    DrainTicks += System.Diagnostics.Stopwatch.GetTimestamp() - started;

                    if (Linger(completed)) continue;

                    Volatile.Write(ref _draining, 0);
                    Interlocked.MemoryBarrier();

                    if (Volatile.Read(ref _issued) == completed) return;
                    if (Interlocked.CompareExchange(ref _draining, 1, 0) != 0) return;
                }
            }
            catch (Exception fault)
            {
                _fault = fault;
                Volatile.Write(ref _completed, Volatile.Read(ref _issued));
                Volatile.Write(ref _draining, 0);
            }
        }

        private bool Linger(long completed)
        {
            for (int i = 0; i < 400; i++)
            {
                if (Volatile.Read(ref _issued) != completed) return true;
                Thread.SpinWait(50);
            }

            return false;
        }

        // Waits until the words a page waits for have run, and clears the mark, since nothing later has marked it - see §2.6.
        public void WaitFor(uint physical) => WaitFor(physical, 0);

        public void WaitFor(uint physical, int site)
        {
            int page = (int)(physical >> 12);
            long mark = _marks[page];
            if (mark == 0) return;

            if (_taking && site == 0) site = 10;
            long started = System.Diagnostics.Stopwatch.GetTimestamp();
            WaitUntil(mark == Idle ? _issued : mark);
            _marks[page] = 0;
            WaitsPerPage[page]++;
            long took = System.Diagnostics.Stopwatch.GetTimestamp() - started;
            WaitsPerSite[site]++; TicksPerSite[site] += took;
            if (WaitsLogged < WaitLog.Length && took > System.Diagnostics.Stopwatch.Frequency / 20000) WaitLog[WaitsLogged++] = (site, (uint)page << 12, mark == Idle, took, _issued - Volatile.Read(ref _completed), _colorImage, _depthImage, _textureImage);
            if (_waiting == 0) { PageWaits++; PageWaitTicks += took; }
        }

        public void WaitForRange(long from, long count) => WaitForRange(from, count, 8);

        public void WaitForRange(long from, long count, int site)
        {
            long length = _bus.Rdram.Length;
            long first = Math.Clamp(from, 0, length) >> 12;
            long last = (Math.Clamp(from + count, 0, length) - 1) >> 12;

            long started = System.Diagnostics.Stopwatch.GetTimestamp();
            _waiting++;
            bool waited = false;
            for (long page = first; page <= last; page++)
            {
                if (_marks[page] != 0) { WaitFor((uint)(page << 12), site); waited = true; }
            }
            _waiting--;
            if (waited) { RangeWaits++; RangeWaitTicks += System.Diagnostics.Stopwatch.GetTimestamp() - started; }
        }

        // Everything handed over has run; a state, a load, a cheat and the debugger start from here - see §2.6.
        public void Join()
        {
            long started = System.Diagnostics.Stopwatch.GetTimestamp();
            bool waited = Volatile.Read(ref _completed) < _issued;
            WaitUntil(_issued);
            if (_threaded) Array.Clear(_marks);
            if (waited) { Joins++; JoinTicks += System.Diagnostics.Stopwatch.GetTimestamp() - started; }
        }

        private void WaitUntil(long words)
        {
            if (Volatile.Read(ref _completed) < words) Kick();

            SpinWait spin = default;
            while (Volatile.Read(ref _completed) < words)
            {
                Rethrow();
                spin.SpinOnce(-1);
            }

            Rethrow();
        }

        private void Rethrow()
        {
            if (_fault is { } fault)
            {
                _fault = null;
                throw new InvalidOperationException("The display processor's thread failed.", fault);
            }
        }

        // Every touch checked against the marks: on in Debug builds and when EMUSEN_MARS_VERIFY_RDP is set, since it costs a pixel a compare - see §2.6.
        public static bool VerifyMarks =
#if DEBUG
            true;
#else
            Environment.GetEnvironmentVariable("EMUSEN_MARS_VERIFY_RDP") == "1";
#endif

        // Called by the processor at every byte of RDRAM it reads or writes while threaded, to prove the marks reach it - see §2.6.
        public bool Verifying => _threaded && VerifyMarks;

        public void Touched(uint physical)
        {
            if (physical >= (uint)_bus.Rdram.Length) return;
            if (Volatile.Read(ref _marks[physical >> 12]) >= _runningWord) return;

            _fault ??= new InvalidOperationException($"The display processor touched {physical:X6} in a page its interface did not mark before word {_runningWord}.");
        }

        // After a state is read, the gathering and the images come from the processor, which the state carried - see §2.6.
        public void RefreshShadow()
        {
            (_shadowTaken, _shadowFirst) = Processor.Gathered;
            (_colorImage, _colorWidth, _colorBytes, _depthImage, _textureImage, _textureWidth, _textureSize, _scissorTop, _scissorBottom) = Processor.Images;
            _drawn = false;
            _issued = _completed;
        }
    }
}
