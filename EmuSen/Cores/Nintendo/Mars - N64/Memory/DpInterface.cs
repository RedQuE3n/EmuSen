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

        // Asked of the thread between two words, and answered when it stands there - see §2.7.
        [EmuSen.Common.SkipInState] private int _pauseRequested;
        [EmuSen.Common.SkipInState] private int _paused;
        [EmuSen.Common.SkipInState] private bool _replaying;

        // For each page of RDRAM, the count of words that must have run before anyone writes it, and before anyone reads it; zero when nothing is due - see §2.6.
        [EmuSen.Common.SkipInState] private readonly long[] _marks;
        [EmuSen.Common.SkipInState] private readonly long[] _writeMarks;

        // The byte ranges behind the marks, in the order of their counts, so a bystander on a marked page can be told from the range - see §2.6.1.
        private const int RangeCount = 1 << 13;
        [EmuSen.Common.SkipInState] private readonly (long From, long To, long Start, long Mark, bool Write)[] _ranges = new (long, long, long, long, bool)[RangeCount];
        [EmuSen.Common.SkipInState] private long _rangesAppended;

        // The mark an image's pages take: everything handed over so far, however much that comes to be - see §2.6.
        private const long Idle = long.MaxValue;

        // The interface's own gathering of the stream, so the full sync is answered when the word arrives - see §2.6.
        [EmuSen.Common.SkipInState] private int _shadowTaken;
        [EmuSen.Common.SkipInState] private ulong _shadowFirst;
        [EmuSen.Common.SkipInState] private uint _colorImage, _depthImage, _textureImage;
        [EmuSen.Common.SkipInState] private int _colorWidth, _colorBytes, _textureWidth, _textureSize, _scissorTop, _scissorBottom, _scissorRight;
        [EmuSen.Common.SkipInState] private bool _drawn;

        // The image extents a batch marked idle, downgraded to the batch's own count when it ends; the sequence lets the thread read them whole - see §2.6.1.
        [EmuSen.Common.SkipInState] private readonly (long From, long To, long Start)[] _idleRanges = new (long, long, long)[256];
        [EmuSen.Common.SkipInState] private int _idleRangeCount;
        [EmuSen.Common.SkipInState] private int _idleSequence;

        // Where the batch being taken began, so a batch that outgrows the extents above can be given one range of everything - see §2.6.1.
        [EmuSen.Common.SkipInState] private long _batchStart;

        // What the thread did: words run and the time in them, drains started and the delay from a kick to its start - see Mars_Performance.md §28.
        [EmuSen.Common.SkipInState] public long DrainWords, DrainTicks, DrainStarts, KickTicks;
        [EmuSen.Common.SkipInState] private long _kickedAt;

        // Set by the thread before each word it runs, so a touch can be checked against the marks - see §2.6.
        [EmuSen.Common.SkipInState] private long _runningWord;

        // How often anyone waited for the thread, and for how long, by the kind of waiter - see Mars_Performance.md §28.
        [EmuSen.Common.SkipInState] public long PageWaits, PageWaitTicks, RangeWaits, RangeWaitTicks, Joins, JoinTicks, Bystanders;
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
            _writeMarks = new long[_marks.Length];
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

        // The marks a writer tests, and the ones a reader tests, for the bus and the processor's direct paths - see §2.6.1.
        public long[] Marks => _marks;

        public long[] WriteMarks => _writeMarks;

        public long MarkFor(uint physical) => _marks[physical >> 12];

        // Words handed over that the thread has not run, which a snapshot carries - see §2.7.
        public long Pending => _issued - Volatile.Read(ref _completed);

        // A snapshot's tail is this many words whatever is pending, so every snapshot of a machine is the same length - see §2.7.
        public const int SnapshotWords = 1 << 15;

        // Waits for the backlog to fit a snapshot's tail, then holds the thread - see §2.7.
        public void Hold()
        {
            if (Pending > SnapshotWords) WaitUntil(_issued - SnapshotWords);
            Pause();
        }

        // The same words, read now as the immediate path reads them; each is marked for, then published, so a wait on a mark can always end - see §2.6.
        private void TakeOntoThread()
        {
            long tail = _issued;
            _batchStart = tail;
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

            // A batch that named more images than the extents hold keeps its pages idle and takes one range of the whole memory, which is the first version's behaviour - see §2.6.1.
            if (_idleRangeCount > _idleRanges.Length) Append(0, _bus.Rdram.Length, _batchStart, tail, write: true);
            else
            {
                for (int i = 0; i < _idleRangeCount; i++)
                {
                    (long from, long to, long start) = _idleRanges[i];
                    Mark(from, to - from, tail, write: true, downgrade: true);
                    Append(from, to, start, tail, write: true);
                }
            }

            Volatile.Write(ref _idleSequence, _idleSequence + 1);
            _idleRangeCount = 0;
            Volatile.Write(ref _idleSequence, _idleSequence + 1);
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
                    if (!_drawn) MarkImages(afterThisWord);
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
                    _scissorBottom = ((int)(first & 0xFFF) + 3) >> 2;
                    _scissorRight = ((int)((first >> 12) & 0xFFF) + 3) >> 2;
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

        // The pixels a draw may address, which is not the scissor's rectangle: a span's right edge is bounded by the scissor and not by the image's width, so an address runs past its row - see §2.6.1.
        private void MarkImages(long start)
        {
            _drawn = true;

            long width = _colorWidth;
            long rows = _scissorBottom - _scissorTop;
            if (rows <= 0) rows = 1;

            // The first pixel the top row can name and the last the bottom row can, each with two pixels of slack for a read beside the span - see §2.6.1.
            long first = Math.Max((long)_scissorTop * width - 2, 0);
            long last = Math.Max((_scissorTop + rows) * width, (_scissorTop + rows - 1) * width + _scissorRight + 3);

            MarkIdle(_colorImage + first * _colorBytes, (last - first) * _colorBytes, start);
            MarkIdle(_depthImage + first * 2, (last - first) * 2, start);
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

            to = Math.Max(to, from + 1);
            Mark(from, to - from, after, write: false);
            Append(from, to, after, after, write: false);
        }

        // Idle until the batch ends, then the batch's count; a batch changing images more often than the list holds keeps them idle - see §2.6.
        private void MarkIdle(long from, long count, long start)
        {
            Mark(from, count, Idle, write: true);

            Volatile.Write(ref _idleSequence, _idleSequence + 1);
            if (_idleRangeCount < _idleRanges.Length) _idleRanges[_idleRangeCount++] = (from, from + count, start);
            else _idleRangeCount = _idleRanges.Length + 1;
            Volatile.Write(ref _idleSequence, _idleSequence + 1);
        }

        // A later mark is never smaller, so the greater of the two keeps an image's idle mark over a load's, until the batch's end writes its count over it - see §2.6.1.
        private void Mark(long from, long count, long after, bool write, bool downgrade = false)
        {
            long length = _bus.Rdram.Length;
            long first = Math.Clamp(from, 0, length) >> 12;
            long last = (Math.Clamp(from + count, 0, length) - 1) >> 12;

            for (long page = first; page <= last; page++)
            {
                _marks[page] = downgrade ? after : Math.Max(_marks[page], after);
                if (write) _writeMarks[page] = downgrade ? after : Math.Max(_writeMarks[page], after);
            }
        }

        // The range is written whole before its count moves, so the thread never reads one half-written - see §2.6.1.
        private void Append(long from, long to, long start, long mark, bool write)
        {
            long length = _bus.Rdram.Length;
            _ranges[(int)(_rangesAppended & (RangeCount - 1))] = (Math.Clamp(from, 0, length), Math.Clamp(to, 0, length), start, mark, write);
            Volatile.Write(ref _rangesAppended, _rangesAppended + 1);
        }

        // Whether any range still pending overlaps the bytes, for a writer any range and for a reader one the processor writes - see §2.6.1.
        private bool Reaches(long from, long to, long completed, bool write)
        {
            if (_idleRangeCount > _idleRanges.Length) return true;
            for (int i = 0; i < _idleRangeCount; i++) if (from < _idleRanges[i].To && to > _idleRanges[i].From) return true;

            long newest = _rangesAppended - 1, oldest = Math.Max(_rangesAppended - RangeCount, 0);
            for (long i = newest; i >= oldest; i--)
            {
                ref var range = ref _ranges[(int)(i & (RangeCount - 1))];
                if (range.Mark <= completed) return false;
                if (from < range.To && to > range.From && (write || range.Write)) return true;
            }

            return newest - oldest + 1 >= RangeCount;
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
                        if (Volatile.Read(ref _pauseRequested) != 0) StandStill();
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
                if (Volatile.Read(ref _pauseRequested) != 0) return false;
                if (Volatile.Read(ref _issued) != completed) return true;
                Thread.SpinWait(50);
            }

            return false;
        }

        // The thread, between two words, until the request is withdrawn - see §2.7.
        private void StandStill()
        {
            Volatile.Write(ref _paused, 1);
            SpinWait spin = default;
            while (Volatile.Read(ref _pauseRequested) != 0) spin.SpinOnce(-1);
            Volatile.Write(ref _paused, 0);
        }

        // Nothing runs on the thread until Resume: it stands between two words, or it has left and cannot be started from here - see §2.7.
        public void Pause()
        {
            Volatile.Write(ref _pauseRequested, 1);
            SpinWait spin = default;
            while (Volatile.Read(ref _draining) != 0 && Volatile.Read(ref _paused) == 0) spin.SpinOnce(-1);
            Rethrow();
        }

        public void Resume()
        {
            Volatile.Write(ref _pauseRequested, 0);
            if (Volatile.Read(ref _completed) < _issued) Kick();
        }

        // The words handed over and not yet run, for a state written while the thread stands - see §2.7.
        public void WritePending(System.IO.BinaryWriter w)
        {
            long completed = Volatile.Read(ref _completed), issued = _issued;
            if (issued - completed > SnapshotWords) throw new InvalidOperationException("More words are pending than a snapshot's tail holds; Hold was not called.");

            w.Write((int)(issued - completed));
            for (long i = completed; i < issued; i++) w.Write(_ring[(int)(i & (_ring.Length - 1))]);
            for (long i = issued - completed; i < SnapshotWords; i++) w.Write(0UL);
        }

        // Run here and now, as the immediate path would have, with the full sync already answered when the words were handed over - see §2.7.
        public void ReadPending(System.IO.BinaryReader r)
        {
            int count = r.ReadInt32();
            _replaying = true;
            try
            {
                for (int i = 0; i < count; i++) Processor.Accept(r.ReadUInt64());
            }
            finally
            {
                _replaying = false;
            }

            r.BaseStream.Seek((SnapshotWords - count) * 8L, System.IO.SeekOrigin.Current);
        }

        // A writer waits for every range on its page, a reader for the ones the processor writes; a bystander on the page waits for neither - see §2.6.1.
        public void WaitFor(uint physical, int site)
        {
            int page = (int)(physical >> 12);
            long mark = _marks[page];
            if (mark != 0) Wait(physical, physical + 1, page, mark, site, write: true);
        }

        public void WaitForRead(uint physical, int site)
        {
            int page = (int)(physical >> 12);
            long mark = _writeMarks[page];
            if (mark != 0) Wait(physical, physical + 1, page, mark, site, write: false);
        }

        private bool Wait(long from, long to, int page, long mark, int site, bool write)
        {
            long completed = Volatile.Read(ref _completed);
            if (mark != Idle && completed >= mark)
            {
                Clear(page, mark);
                return false;
            }

            if (!Reaches(from, to, completed, write))
            {
                Bystanders++;
                return false;
            }

            if (_taking && site == 0) site = 10;
            long started = System.Diagnostics.Stopwatch.GetTimestamp();
            WaitUntil(mark == Idle ? _issued : mark);
            if (mark != Idle) Clear(page, mark);
            WaitsPerPage[page]++;
            long took = System.Diagnostics.Stopwatch.GetTimestamp() - started;
            WaitsPerSite[site]++; TicksPerSite[site] += took;
            if (WaitsLogged < WaitLog.Length && took > System.Diagnostics.Stopwatch.Frequency / 20000) WaitLog[WaitsLogged++] = (site, (uint)page << 12, mark == Idle, took, _issued - Volatile.Read(ref _completed), _colorImage, _depthImage, _textureImage);
            if (_waiting == 0) { PageWaits++; PageWaitTicks += took; }
            return true;
        }

        // A mark no greater than the one waited for has been run; an idle one, or a later load's, stands - see §2.6.1.
        private void Clear(int page, long mark)
        {
            if (_marks[page] <= mark) _marks[page] = 0;
            if (_writeMarks[page] <= mark) _writeMarks[page] = 0;
        }

        public void WaitForRange(long from, long count, int site) => WaitRange(from, count, site, write: true);

        public void WaitForReadRange(long from, long count, int site) => WaitRange(from, count, site, write: false);

        private void WaitRange(long from, long count, int site, bool write)
        {
            long length = _bus.Rdram.Length;
            long first = Math.Clamp(from, 0, length) >> 12;
            long last = (Math.Clamp(from + count, 0, length) - 1) >> 12;
            long[] marks = write ? _marks : _writeMarks;

            long started = System.Diagnostics.Stopwatch.GetTimestamp();
            _waiting++;
            bool waited = false;
            for (long page = first; page <= last; page++)
            {
                long mark = marks[page];
                if (mark == 0) continue;

                long pageFrom = Math.Max(from, page << 12), pageTo = Math.Min(from + count, (page + 1) << 12);
                waited |= Wait(pageFrom, pageTo, (int)page, mark, site, write);
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
            if (_threaded)
            {
                Array.Clear(_marks);
                Array.Clear(_writeMarks);
                _rangesAppended = 0;
            }
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

        // Called by the processor at every byte of RDRAM it reads or writes while threaded, to prove the marks and their ranges reach it - see §2.6.1.
        public bool Verifying => _threaded && VerifyMarks && !_replaying;

        public void Touched(uint physical) => Verify(physical, write: false);

        public void Wrote(uint physical) => Verify(physical, write: true);

        private void Verify(uint physical, bool write)
        {
            if (physical >= (uint)_bus.Rdram.Length) return;
            long word = _runningWord;
            long[] marks = write ? _writeMarks : _marks;

            if (Volatile.Read(ref marks[physical >> 12]) < word)
            {
                _fault ??= new InvalidOperationException($"The display processor {(write ? "wrote" : "read")} {physical:X6} in a page its interface did not mark before word {word}.");
                return;
            }

            if (!Covers(physical, word, write)) _fault ??= new InvalidOperationException($"The display processor {(write ? "wrote" : "read")} {physical:X6} outside every range its interface marked for word {word}. {Nearest(physical, word)}. {Ranges()}");
        }

        // For a fault's message: the nearest range in the ring that holds the byte at all, whatever its counts said.
        private string Nearest(uint physical, long word)
        {
            long appended = Volatile.Read(ref _rangesAppended);
            for (long i = appended - 1; i >= Math.Max(appended - RangeCount, 0); i--)
            {
                var range = _ranges[(int)(i & (RangeCount - 1))];
                if (physical >= range.From && physical < range.To)
                {
                    return $"nearest holder at {i} of {appended}: [{range.From:X6}-{range.To:X6} from {range.Start} to {range.Mark}{(range.Write ? " w" : " r")}]"
                        + (range.Start > word ? " START TOO LATE" : "") + (range.Mark < word ? " MARK TOO EARLY" : "");
                }
            }

            return $"no range in the ring of {appended} holds it";
        }

        // What was marked, for a fault's message: the images the shadow holds and every range still pending.
        private string Ranges()
        {
            var text = new System.Text.StringBuilder($"color {_colorImage:X6} {_colorWidth}x{_colorBytes}B rows {_scissorTop}-{_scissorBottom}, depth {_depthImage:X6}, texture {_textureImage:X6} {_textureWidth}w size {_textureSize}; open");
            for (int i = 0; i < Math.Min(_idleRangeCount, _idleRanges.Length); i++) text.Append($" [{_idleRanges[i].From:X6}-{_idleRanges[i].To:X6} from {_idleRanges[i].Start}]");

            text.Append("; ring");
            for (long i = _rangesAppended - 1; i >= Math.Max(_rangesAppended - 12, 0); i--)
            {
                var range = _ranges[(int)(i & (RangeCount - 1))];
                text.Append($" [{range.From:X6}-{range.To:X6} from {range.Start} to {range.Mark}{(range.Write ? " w" : " r")}]");
            }

            return text.ToString();
        }

        // The batch still open, read whole by its sequence; then the ring newest first, stopping where the counts fall below the word - see §2.6.1.
        private bool Covers(uint physical, long word, bool write)
        {
            while (true)
            {
                int sequence = Volatile.Read(ref _idleSequence);
                if ((sequence & 1) != 0) continue;

                int count = _idleRangeCount;
                bool covered = count > _idleRanges.Length;
                for (int i = 0; i < count && !covered; i++)
                {
                    var range = _idleRanges[i];
                    covered = range.Start <= word && physical >= range.From && physical < range.To;
                }

                Interlocked.MemoryBarrier();
                if (Volatile.Read(ref _idleSequence) != sequence) continue;
                if (covered) return true;
                break;
            }

            long appended = Volatile.Read(ref _rangesAppended);
            for (long i = appended - 1; i >= Math.Max(appended - RangeCount, 0); i--)
            {
                var range = _ranges[(int)(i & (RangeCount - 1))];

                // The slot may have been written over while it was read, which leaves the scan with nothing to say - see §2.6.1.
                if (Volatile.Read(ref _rangesAppended) - i >= RangeCount) return true;
                if (range.Mark < word) return false;
                if (range.Start <= word && physical >= range.From && physical < range.To && (!write || range.Write)) return true;
            }

            // Every range the ring holds is newer than the word, so the one that spoke for it is gone - see §2.6.1.
            return appended >= RangeCount;
        }

        // After a state is read, the gathering and the images come from the processor, which the state carried - see §2.6.
        public void RefreshShadow()
        {
            (_shadowTaken, _shadowFirst) = Processor.Gathered;
            (_colorImage, _colorWidth, _colorBytes, _depthImage, _textureImage, _textureWidth, _textureSize, _scissorTop, _scissorBottom, _scissorRight) = Processor.Images;
            _drawn = false;
            _issued = _completed;
        }
    }
}
