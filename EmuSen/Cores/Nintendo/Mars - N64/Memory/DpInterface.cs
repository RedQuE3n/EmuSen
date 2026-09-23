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

        // Sees each word the list runs inline, before and after the processor takes it, for a differential against another - see Mars_Native.md §5.3.
        public interface IWordWatcher
        {
            void Before(ulong word);

            void After(ulong word, bool fullSync);
        }

        [EmuSen.Common.SkipInState] public IWordWatcher? Watcher;

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

        // Drawing at a multiple: a memory of the scale squared and a processor per thread that draws into it beside the native one - see Mars_Rdp.md §11.
        [EmuSen.Common.SkipInState] private int _scale = 1;
        [EmuSen.Common.SkipInState] private byte[] _scaledRdram = Array.Empty<byte>();
        [EmuSen.Common.SkipInState] private byte[] _scaledHidden = Array.Empty<byte>();
        [EmuSen.Common.SkipInState] private Rdp.Rdp? _scaledProcessor;

        // The compute device the multiple is shaded on, when one was asked for and one exists - see Mars_Gpu.md §11.
        [EmuSen.Common.SkipInState] private Rdp.Gpu.GpuDevice? _device;
        [EmuSen.Common.SkipInState] private Rdp.Gpu.GpuRasteriser? _gpu;
        [EmuSen.Common.SkipInState] private bool _wantsGpu;

        // What the setting asked for and what it got, which a frontend shows and a test reads. Not machine state:
        // a device is a property of the host, and a state carried one string of it into the serializer's walk.
        [EmuSen.Common.SkipInState] private string _gpuReport = "off";

        public string GpuReport => _gpuReport;

        public bool Gpu
        {
            get => _wantsGpu;
            set
            {
                if (value == _wantsGpu) return;
                Join();
                StopWorkers();
                _wantsGpu = value;
                RebuildGpu();
                if (_scale > 1) { _scaledProcessor = NewScaled(); }
                StartWorkers();
            }
        }

        // A device and a rasteriser for this multiple, or neither and the reason why - see §11.1.
        private void RebuildGpu()
        {
            _scanOuts++;
            _gpu?.Dispose();
            _gpu = null;
            _device?.Dispose();
            _device = null;

            if (!_wantsGpu || _scale <= 1) { _gpuReport = _wantsGpu ? "off at one" : "off"; return; }

            _device = Rdp.Gpu.GpuDevice.TryCreate(null, out string report);
            if (_device is null) { _gpuReport = report; return; }

            _gpu = Rdp.Gpu.GpuRasteriser.TryCreate(_device, _scaledRdram.Length, out report);
            if (_gpu is null) { _device.Dispose(); _device = null; }
            _gpuReport = report;
        }

        // True when the device holds the memory at the multiple and can walk the picture out of it itself - see Mars_Gpu.md §13.
        public bool CanScanOut => _gpu is not null;

        // Counts the device's scans and its rebuilds, so a job can tell whether the picture the device holds is still its own - see Mars_Gpu.md §14.
        [EmuSen.Common.SkipInState] private long _scanOuts;

        public long ScanOuts => _scanOuts;

        // The VI's walk over the device's own memory, once the drawing is finished, submitted without waiting - see Mars_Gpu.md §14.
        public void ScanOut(in Rdp.Gpu.GpuRasteriser.ScanParameters scan)
        {
            Join();
            _gpu!.ScanOut(scan);
            _scanOuts++;
        }

        // The last scan's picture once the device has walked it; valid until the next scan or a change of the device.
        public ReadOnlySpan<uint> ScannedPicture => _gpu!.ScannedPicture;

        // True when the device already holds the raster at the multiple of this size, so no seed need go with the next submission.
        public bool HoldsRaster(int width, int height) => _gpu is not null && _gpu.HoldsRaster(width, height);

        // The VI's clears and walk into the raster the device keeps while it averages, submitted without waiting - see Mars_Gpu.md §15.
        public void ScanIntoRaster(int width, int height, int side, ReadOnlySpan<byte> seed, bool clear, ReadOnlySpan<uint> spans, bool walk, in Rdp.Gpu.GpuRasteriser.ScanParameters scan)
        {
            // Joined even with no walk: the leading worker submits its own flushes, and the queue takes one thread at a time.
            Join();
            _gpu!.ScanIntoRaster(width, height, side, seed, clear, spans, walk, scan);
            if (walk) _scanOuts++;
        }

        // The raster averaged on the device, once it has finished; valid until the next submission to it or a change of the device.
        public ReadOnlySpan<uint> AveragedRaster => _gpu!.AveragedRaster;

        // Everything the device holds, read back into the shadow the scan-out walks - see §11.2.
        public void ReadBackScaled(uint from, int count)
        {
            if (_gpu is null || count <= 0) return;
            Join();
            _gpu.Read(from, count, _scaledRdram, _scaledHidden);
        }

        // The processors that share a list when more than one does, each on a thread of its own, the first being Processor - see §2.8.
        [EmuSen.Common.SkipInState] private Worker[] _workers = Array.Empty<Worker>();
        [EmuSen.Common.SkipInState] private int _workerCount = 1;
        [EmuSen.Common.SkipInState] private readonly SpinBarrier _barrier;
        [EmuSen.Common.SkipInState] private int _stopping;
        [EmuSen.Common.SkipInState] private int _faulted;
        [EmuSen.Common.SkipInState] private long _pauseAt = -1;

        // How often a waiter at a barrier raised the pause point to its own word, which a test reads to know the case was met - see §2.8.
        [EmuSen.Common.SkipInState] public long PausesRaised;

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

        // The batch's colour and depth extents, grown by each draw's own rows, and what bounds them - see Mars_Rdp.md §2.6.2.
        private struct Extent { public int Index; public long First, Last; }
        [EmuSen.Common.SkipInState] private Extent _colorExtent = new() { Index = -1 }, _depthExtent = new() { Index = -1 };
        [EmuSen.Common.SkipInState] private int _scissorTopRaw, _scissorBottomRaw, _shadowCycle;

        // Each draw's rows and columns, the images it writes, and the word it ends on, newest last; a small read waits only for the ones holding it - see Mars_Rdp.md §2.6.3.
        private struct Box
        {
            public long End;
            public uint Color, Depth;
            public int Bytes, Width, ScissorRight, RowFirst, RowLast, Kind;

            // The command's own words, kept as they came; its columns are worked out from them only when a read or the verifier asks.
            public ulong First, Low, High, Middle;
        }

        private const int Empty = 0, Rectangle = 1, Triangle = 2, WholeRows = 3;
        private const int BoxCount = 1 << 14;
        [EmuSen.Common.SkipInState] private readonly Box[] _boxes = new Box[BoxCount];
        [EmuSen.Common.SkipInState] private long _boxesAppended;
        [EmuSen.Common.SkipInState] public long ReadsNarrowed, ReadsFreed;

        // A command already being gathered when a state was read has its first words in the processor, not the ring, so its columns are not the ring's to tell.
        [EmuSen.Common.SkipInState] private bool _gatheredBeforeTheRing;
        [EmuSen.Common.SkipInState] private bool _shadowDepth, _shadowDepthUpdate;

        // Where the batch being taken began, so a batch that outgrows the extents above can be given one range of everything - see §2.6.1.
        [EmuSen.Common.SkipInState] private long _batchStart;

        // What the thread did: words run and the time in them, drains started and the delay from a kick to its start - see Mars_Performance.md §28.
        [EmuSen.Common.SkipInState] public long DrainWords, DrainTicks, DrainStarts, KickTicks;
        [EmuSen.Common.SkipInState] private long _kickedAt;

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
            _barrier = new SpinBarrier(this);
        }

        // One processor's thread: how far it has come, whether it sleeps, and where it stands when asked to - see §2.8.
        private sealed class Worker
        {
            public readonly int Index;
            public readonly Rdp.Rdp Processor;
            public Rdp.Rdp? Scaled;
            public readonly ManualResetEventSlim Wake = new(false);
            public Thread? Thread;
            public long Completed;
            public long Standing = -1;
            public int Sleeping;
            public long Words, Ticks;

            public Worker(int index, Rdp.Rdp processor) => (Index, Processor) = (index, processor);
        }

        // Every worker arrives before any leaves; a fault lets them all through - see §2.8.
        private sealed class SpinBarrier
        {
            private readonly DpInterface _owner;
            private int _parties, _arrived, _generation;
            public long Passed;

            public SpinBarrier(DpInterface owner) => _owner = owner;

            public void Reset(int parties) => (_parties, _arrived) = (parties, 0);

            public void Arrive(long word)
            {
                int generation = Volatile.Read(ref _generation);
                if (Interlocked.Increment(ref _arrived) == _parties)
                {
                    Passed++;
                    _arrived = 0;
                    Volatile.Write(ref _generation, generation + 1);
                    return;
                }

                SpinWait spin = default;
                while (Volatile.Read(ref _generation) == generation && Volatile.Read(ref _owner._faulted) == 0)
                {
                    _owner.RaisePauseTo(word);
                    spin.SpinOnce(-1);
                }
            }
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

                Watcher?.Before(word);
                bool sync = Processor.Accept(word);
                Watcher?.After(word, sync);
                if (sync) FullSync();
                _scaledProcessor?.Accept(word);
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
                StopWorkers();
                _threaded = value;
                if (value) RefreshShadow();
                StartWorkers();
            }
        }

        // The multiple the picture is drawn at beside the machine's own, into a memory of its own; one is the machine's alone - see Mars_Rdp.md §11.
        public int Scale
        {
            get => _scale;
            set
            {
                int scale = Math.Clamp(value, 1, 8);
                if (scale == _scale) return;
                Join();
                StopWorkers();
                _scale = scale;
                _scaledRdram = scale > 1 ? new byte[_bus.Rdram.Length * scale * scale] : Array.Empty<byte>();
                _scaledHidden = scale > 1 ? new byte[_bus.RdramHidden.Length * scale * scale] : Array.Empty<byte>();
                RebuildGpu();
                _scaledProcessor = scale > 1 ? NewScaled() : null;
                StartWorkers();
            }
        }

        public byte[] ScaledRdram => _scaledRdram;

        public byte[] ScaledHidden => _scaledHidden;

        // True once a draw has reached the scaled memory since it was last emptied, which is when the scan-out may show it - see Mars_Rdp.md §11.
        public bool ScaledDrawn
        {
            get
            {
                if (_scale == 1) return false;
                if (_gpu is not null) return _scaledProcessor is { Drew: true };
                if (_scaledProcessor is { Drew: true }) return true;
                foreach (Worker worker in _workers) if (worker.Scaled is { Drew: true }) return true;
                return false;
            }
        }

        // ScaledDrawn as of every word handed over: on the drain the flag is the workers' progress, so while it is false the drain is joined first - see Mars_Rdp.md §11.
        public bool ScaledDrawnHandedOver
        {
            get
            {
                if (_scale == 1) return false;
                if (!ScaledDrawn && _threaded && Completed() < _issued) Join();
                return ScaledDrawn;
            }
        }

        // A processor at the multiple, standing where the native one stands - see Mars_Rdp.md §11.
        private Rdp.Rdp NewScaled()
        {
            var scaled = new Rdp.Rdp(_bus);
            scaled.DrawAt(_scale, _scaledRdram, _scaledHidden);
            scaled.CopyStateFrom(Processor);
            scaled.Rescale();
            scaled.ShadeOn(_gpu);
            return scaled;
        }

        // After a state is read, the scaled memory holds another machine's picture: emptied, and shown again only once something draws - see §11.
        private void ResetScaled()
        {
            if (_scale == 1) return;
            Array.Clear(_scaledRdram);
            Array.Clear(_scaledHidden);
            _gpu?.Clear();
            _scaledProcessor = NewScaled();
            foreach (Worker worker in _workers)
            {
                if (_gpu is null)
                {
                    worker.Scaled = NewScaled();
                    worker.Scaled.Configure(worker.Index, _workerCount);
                }
            }
        }

        // How many processors share the list, on threads of their own, each shading the rows that are its by count - see §2.8.
        public int Workers
        {
            get => _workerCount;
            set
            {
                int count = Math.Max(1, value);
                if (count == _workerCount) return;
                Join();
                StopWorkers();
                _workerCount = count;
                StartWorkers();
            }
        }

        // What each processor's thread did, and how often every one of them waited for the others - see Mars_Performance.md §35.
        public (long Words, long Ticks)[] WorkerLoads => Array.ConvertAll(_workers, w => (w.Words, w.Ticks));

        public long Barriers => _barrier.Passed;

        // Reads past a row's end made for another processor's row, by whichever processor made them - see §2.8.
        public long AliasedReads
        {
            get
            {
                long reads = Processor.AliasedReads;
                foreach (Worker worker in _workers) if (worker.Index > 0) reads += worker.Processor.AliasedReads;
                return reads;
            }
        }

        private void StartWorkers()
        {
            if (!_threaded || _workerCount == 1) return;

            long completed = Completed();
            _barrier.Reset(_workerCount);
            _workers = new Worker[_workerCount];
            for (int i = 0; i < _workerCount; i++)
            {
                Rdp.Rdp processor = i == 0 ? Processor : new Rdp.Rdp(_bus);
                if (i > 0) processor.CopyStateFrom(Processor);
                processor.Configure(i, _workerCount);
                Worker worker = new(i, processor) { Completed = completed };
                if (_scale > 1 && _gpu is null)
                {
                    worker.Scaled = NewScaled();
                    worker.Scaled.Configure(i, _workerCount);
                }
                worker.Thread = new Thread(() => RunWorker(worker)) { IsBackground = true, Name = $"Mars RDP {i}" };
                _workers[i] = worker;
            }

            foreach (Worker worker in _workers) worker.Thread!.Start();
        }

        // After a join, so every thread stands at the end; the count carries over to the pool path - see §2.8.
        private void StopWorkers()
        {
            if (_workers.Length == 0) return;

            Volatile.Write(ref _stopping, 1);
            foreach (Worker worker in _workers) worker.Wake.Set();
            foreach (Worker worker in _workers) worker.Thread!.Join();
            Volatile.Write(ref _stopping, 0);

            _completed = Math.Min(Completed(), _issued);
            _workers = Array.Empty<Worker>();
            Processor.Configure(0, 1);
        }

        // How far every processor has come: the least of them, since a word is done when all have run it - see §2.8.
        private long Completed()
        {
            if (_workers.Length == 0) return Volatile.Read(ref _completed);

            long least = long.MaxValue;
            foreach (Worker worker in _workers) least = Math.Min(least, Volatile.Read(ref worker.Completed));
            return least;
        }

        // The leader's scratch replaced by whichever processor wrote each field last in raster order - see §2.8.
        private void Assemble()
        {
            for (int i = 1; i < _workers.Length; i++) Processor.TakeScratchFrom(_workers[i].Processor);
            if (_workers.Length > 0 && _workers[0].Scaled is { } leader) for (int i = 1; i < _workers.Length; i++) if (_workers[i].Scaled is { } other) leader.TakeScratchFrom(other);
        }

        // The marks a writer tests, and the ones a reader tests, for the bus and the processor's direct paths - see §2.6.1.
        public long[] Marks => _marks;

        public long[] WriteMarks => _writeMarks;

        public long MarkFor(uint physical) => _marks[physical >> 12];

        // Words handed over that the thread has not run, which a snapshot carries - see §2.7.
        public long Pending => _issued - Completed();

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
            _colorExtent.Index = _depthExtent.Index = -1;
            _taking = true;

            while (!_freeze && _current < _end)
            {
                ulong word = _xbus ? ReadDataMemory(_current) : _bus.Read64(_current);
                _current += 8;
                tail = Publish(word, tail);
            }

            _taking = false;
            EndBatch(tail);
        }

        // Words from a test or a bench, handed over as a list of their own would be, without a fetch from memory - see §2.8.
        public void HandOver(ReadOnlySpan<ulong> words)
        {
            if (!_threaded) throw new InvalidOperationException("Words can only be handed over to the threaded interface.");

            long tail = _issued;
            _batchStart = tail;
            _drawn = false;
            _colorExtent.Index = _depthExtent.Index = -1;
            _taking = true;
            foreach (ulong word in words) tail = Publish(word, tail);
            _taking = false;
            EndBatch(tail);
        }

        private long Publish(ulong word, long tail)
        {
            if (tail - Completed() >= _ring.Length) MakeRoom(tail);
            _ring[(int)(tail & (_ring.Length - 1))] = word;
            tail++;

            bool sync = Shadow(word, tail);
            Volatile.Write(ref _issued, tail);
            Kick();

            if (sync) FullSync();
            return tail;
        }

        private void EndBatch(long tail)
        {
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
            if (_workers.Length > 0)
            {
                Interlocked.MemoryBarrier();
                foreach (Worker worker in _workers) if (Volatile.Read(ref worker.Sleeping) != 0) worker.Wake.Set();
                return;
            }

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
            while (tail - Completed() >= _ring.Length)
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
            bool straddled = _gatheredBeforeTheRing;
            _gatheredBeforeTheRing = false;

            switch (id)
            {
                case >= 0x08 and <= 0x0F:
                case Rdp.Rdp.TextureRectangle or Rdp.Rdp.TextureRectangleFlipped:
                case Rdp.Rdp.FillRectangle:
                    MarkDraw(id, first, afterThisWord, straddled);
                    return false;

                case Rdp.Rdp.SetOtherModes:
                    _shadowCycle = (int)(first >> 52) & 3;
                    _shadowDepth = ((first >> 4) & 3) != 0;
                    _shadowDepthUpdate = ((first >> 5) & 1) != 0;
                    return false;

                case Rdp.Rdp.SyncFull:
                    return true;

                case Rdp.Rdp.SetColorImage:
                    _drawn = false;
                    _colorExtent.Index = -1;
                    _colorBytes = (int)((first >> 51) & 3) switch { 1 => 1, 2 => 2, 3 => 4, _ => 1 };
                    _colorWidth = (int)((first >> 32) & 0x3FF) + 1;
                    _colorImage = (uint)first & 0x00FF_FFFF;
                    return false;

                case Rdp.Rdp.SetMaskImage:
                    _drawn = false;
                    _depthExtent.Index = -1;
                    _depthImage = (uint)first & 0x00FF_FFFF;
                    return false;

                case Rdp.Rdp.SetScissor:
                    _drawn = false;
                    _colorExtent.Index = _depthExtent.Index = -1;
                    _scissorTopRaw = (int)((first >> 32) & 0xFFF);
                    _scissorBottomRaw = (int)(first & 0xFFF);
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

        // A draw marks the rows the walker can shade, by its own limits, and the depth image only in a mode that reads or writes it - see Mars_Rdp.md §2.6.2.
        private void MarkDraw(uint id, ulong first, long start, bool straddled)
        {
            _drawn = true;
            int upper, lower;
            if (id is >= 0x08 and <= 0x0F)
            {
                int yh = SignExtend14(first), yl = SignExtend14(first >> 32);
                upper = (yh & 0x2000) != 0 ? _scissorTopRaw : (yh & 0x1000) != 0 ? yh : Math.Max(yh, _scissorTopRaw);
                lower = (yl & 0x2000) != 0 ? yl : (yl & 0x1000) != 0 ? _scissorBottomRaw : Math.Min(yl, _scissorBottomRaw);
            }
            else
            {
                // Fill and copy draw a rectangle's bottom row whole, which the other modes' sub-scanline test would not.
                int top = (int)(first & 0xFFF), bottom = (int)((first >> 32) & 0xFFF) | (_shadowCycle >= 2 ? 3 : 0);
                upper = Math.Max(top, _scissorTopRaw);
                lower = Math.Min(bottom, _scissorBottomRaw);
            }

            // A draw with no rows still records a box, an empty one, so a write it makes is one the verifier can fault.
            if (lower <= upper)
            {
                AppendBox(Empty, first, 0, -1, start);
                return;
            }

            int kind = straddled ? WholeRows : id is >= 0x08 and <= 0x0F ? Triangle : Rectangle;
            AppendBox(kind, first, upper, lower, start);

            // A span's right edge is bounded by the scissor, not the width, so it runs past its row; two pixels of slack for a read beside it - see §2.6.1.
            long width = _colorWidth, firstRow = upper >> 2, lastRow = (lower - 1) >> 2;
            long from = Math.Max(firstRow * width - 2, 0);
            long to = Math.Max((lastRow + 1) * width, lastRow * width + _scissorRight + 3);

            Grow(ref _colorExtent, _colorImage, _colorBytes, from, to, start);
            if (_shadowCycle < 2 && _shadowDepth) Grow(ref _depthExtent, _depthImage, 2, from, to, start);
        }

        private static int SignExtend14(ulong v) => ((int)(v & 0x3FFF) << 18) >> 18;

        // Written whole before its count moves, as a range is, so the verifier on a worker never reads one half-written.
        private void AppendBox(int kind, ulong first, int upper, int lower, long end)
        {
            // A draw that compares depth without updating it only reads the depth image, and a read never waits for another reader.
            bool depth = _shadowCycle < 2 && _shadowDepthUpdate;
            ref Box box = ref _boxes[(int)(_boxesAppended & (BoxCount - 1))];
            box.End = end;
            box.Kind = kind;
            box.Color = _colorImage;
            box.Depth = depth ? _depthImage : uint.MaxValue;
            box.Bytes = _colorBytes;
            box.Width = _colorWidth;
            box.ScissorRight = _scissorRight;
            box.RowFirst = upper >> 2;
            box.RowLast = (lower - 1) >> 2;
            box.First = first;
            if (kind == Triangle)
            {
                int length = Rdp.Rdp.Length(Rdp.Rdp.Id(first));
                box.Low = _ring[(int)((end - length + 1) & (_ring.Length - 1))];
                box.High = _ring[(int)((end - length + 2) & (_ring.Length - 1))];
                box.Middle = _ring[(int)((end - length + 3) & (_ring.Length - 1))];
                box.RowFirst = upper;
                box.RowLast = lower;
            }

            Volatile.Write(ref _boxesAppended, _boxesAppended + 1);
        }

        private static long Signed(ulong v, int bits) => ((long)(v & ((1UL << bits) - 1)) << (64 - bits)) >> (64 - bits);

        // A triangle's edges as the walker steps them, from its words: each edge's start and its step a sub-scanline, and the sub-scanlines it spans; null when unsure.
        private static (long Xh, long Xm, long Xl, long Sh, long Sm, long Sl, long Top, long Bottom, long Ym)? Edges(in Box box)
        {
            long xl = Signed(box.Low >> 32, 28), dl = Signed(box.Low, 30);
            long xh = Signed(box.High >> 32, 28), dh = Signed(box.High, 30);
            long xm = Signed(box.Middle >> 32, 28), dm = Signed(box.Middle, 30);
            int yh = SignExtend14(box.First), ym = SignExtend14(box.First >> 16);

            long top = yh & ~3, bottom = box.RowLast | 3, span = Math.Max(0, bottom - top);
            long sh = (dh >> 2) & ~1, sm = (dm >> 2) & ~1, sl = (dl >> 2) & ~1;
            if (Math.Abs(xh + sh * span) >= (1L << 31)) return null;
            return (xh, xm, xl, sh, sm, sl, top, bottom, ym);
        }

        // The columns a triangle's sub-scanlines can name from one sub-scanline to another, from its edges' two ends there, since an edge is a line.
        private static (long, long) Reach((long Xh, long Xm, long Xl, long Sh, long Sm, long Sl, long Top, long Bottom, long Ym) e, long from, long to)
        {
            long low = long.MaxValue, high = long.MinValue;
            void Edge(long x, long step, long first, long last)
            {
                long a = (x + step * first) & ~1, b = (x + step * last) & ~1;
                low = Math.Min(low, Math.Min(a, b));
                high = Math.Max(high, Math.Max(a, b));
            }

            Edge(e.Xh, e.Sh, from - e.Top, to - e.Top);
            if (e.Ym >= e.Top && e.Ym <= e.Bottom)
            {
                if (from <= e.Ym) Edge(e.Xm, e.Sm, from - e.Top, Math.Min(to, e.Ym) - e.Top);
                if (to >= e.Ym) Edge(e.Xl, e.Sl, Math.Max(from, e.Ym) - e.Ym, to - e.Ym);
            }
            else Edge(e.Xm, e.Sm, from - e.Top, to - e.Top);

            return (low, high);
        }

        // The rows and, for one row, the columns a box's draw can reach, a pixel of slack each way and the scissor's reach past the row kept; false for none.
        private static bool Row(in Box box, long row, out long left, out long right)
        {
            left = 0;
            right = -1;
            if (box.Kind == Empty) return false;

            if (box.Kind == Triangle)
            {
                long firstRow = box.RowFirst >> 2, lastRow = (box.RowLast - 1) >> 2;
                if (row < firstRow || row > lastRow) return false;

                if (Edges(box) is not { } e) { left = -3; right = box.ScissorRight + 3; return true; }
                (long low, long high) = Reach(e, Math.Max(row * 4, e.Top), Math.Min(row * 4 + 3, e.Bottom));
                if (low < 0 || high >= (1L << 27)) { left = -3; right = box.ScissorRight + 3; return true; }
                left = (low >> 16) - 4;
                right = Math.Min((high >> 16) + 5, box.ScissorRight) + 3;
                return true;
            }

            if (row < box.RowFirst || row > box.RowLast) return false;
            if (box.Kind == WholeRows) { left = -3; right = box.ScissorRight + 3; return true; }

            left = ((long)((box.First >> 12) & 0xFFF) >> 2) - 3;
            right = Math.Min(((long)((box.First >> 44) & 0xFFF) >> 2) + 2, box.ScissorRight) + 3;
            return true;
        }

        // Whether any byte from one address to another is a pixel the box's draw can reach, a span running on into the next row included.
        private static bool Holds(in Box box, long from, long to) =>
            Within(box, box.Color, box.Bytes, from, to) || (box.Depth != uint.MaxValue && Within(box, box.Depth, 2, from, to));

        private static bool Within(in Box box, uint image, int bytes, long from, long to)
        {
            for (long at = from; at < to; at += bytes)
            {
                if (at < image) continue;
                long pixel = (at - image) / bytes, row = pixel / box.Width, column = pixel - row * box.Width;
                for (long r = row - 1; r <= row + 1; r++)
                {
                    long c = column + (row - r) * box.Width;
                    if (Row(box, r, out long left, out long right) && c >= left && c <= right) return true;
                }
            }

            return false;
        }

        // The last draw still pending whose box holds the eight bytes about an address; zero when none does, the idle count when the ring cannot say.
        private long LastHolding(long physical, long completed)
        {
            long from = physical & ~7L, to = from + 8;
            long appended = _boxesAppended;
            for (long i = appended - 1; i >= 0; i--)
            {
                if (appended - i > BoxCount) return Idle;
                ref Box box = ref _boxes[(int)(i & (BoxCount - 1))];
                if (box.End <= completed) return 0;
                if (Holds(box, from, to)) return box.End;
            }

            return 0;
        }

        // An extent's idle range grown in place under the sequence the verifier reads it by; one that could not be listed stays whole-memory.
        private void Grow(ref Extent extent, uint image, int bytes, long from, long to, long start)
        {
            if (extent.Index == -2) return;
            if (extent.Index >= 0 && from >= extent.First && to <= extent.Last) return;
            if (extent.Index >= 0) { from = Math.Min(from, extent.First); to = Math.Max(to, extent.Last); }

            long address = image + from * bytes, count = (to - from) * bytes;
            if (extent.Index < 0)
            {
                MarkIdle(address, count, start);
                extent.Index = _idleRangeCount <= _idleRanges.Length ? _idleRangeCount - 1 : -2;
            }
            else
            {
                Mark(address, count, Idle, write: true);
                Volatile.Write(ref _idleSequence, _idleSequence + 1);
                _idleRanges[extent.Index] = (address, address + count, _idleRanges[extent.Index].Start);
                Volatile.Write(ref _idleSequence, _idleSequence + 1);
            }

            extent.First = from;
            extent.Last = to;
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
                        Processor.RunningWord = completed + 1;
                        Processor.Accept(_ring[(int)(completed & (_ring.Length - 1))]);
                        _scaledProcessor?.Accept(_ring[(int)(completed & (_ring.Length - 1))]);
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

        // A processor's thread: every word in order, each command as its kind asks, standing when asked to and sleeping when there is nothing - see §2.8.
        private void RunWorker(Worker w)
        {
            Rdp.Rdp p = w.Processor;
            Rdp.Rdp? s = w.Scaled ?? (w.Index == 0 ? _scaledProcessor : null);
            long completed = w.Completed;
            bool inCommand = false;
            long runStart = System.Diagnostics.Stopwatch.GetTimestamp(), runFrom = completed;

            try
            {
                while (true)
                {
                    if (Volatile.Read(ref _pauseRequested) != 0)
                    {
                        Stand(w, completed);
                        s = w.Scaled ?? (w.Index == 0 ? _scaledProcessor : null);
                    }
                    if (Volatile.Read(ref _stopping) != 0) return;

                    if (completed == Volatile.Read(ref _issued))
                    {
                        w.Words += completed - runFrom;
                        w.Ticks += System.Diagnostics.Stopwatch.GetTimestamp() - runStart;
                        Sleep(w, completed);
                        s = w.Scaled ?? (w.Index == 0 ? _scaledProcessor : null);
                        runStart = System.Diagnostics.Stopwatch.GetTimestamp();
                        runFrom = completed;
                        continue;
                    }

                    p.RunningWord = completed + 1;
                    ulong word = _ring[(int)(completed & (_ring.Length - 1))];
                    Rdp.Rdp.Step step = p.Gather(word);
                    if (s is not null)
                    {
                        s.Gather(word);
                        s.Follow(p);
                    }
                    inCommand = step == Rdp.Rdp.Step.More;
                    switch (step)
                    {
                        case Rdp.Rdp.Step.Ready:
                            p.Execute();
                            s?.Execute();
                            break;

                        case Rdp.Rdp.Step.Leader:
                            _barrier.Arrive(completed + 1);
                            if (w.Index == 0)
                            {
                                Assemble();
                                p.Execute();
                                s?.Execute();
                            }
                            _barrier.Arrive(completed + 1);
                            break;

                        case Rdp.Rdp.Step.All:
                            _barrier.Arrive(completed + 1);
                            p.Execute();
                            s?.Execute();
                            break;

                        case Rdp.Rdp.Step.AllJoined:
                            _barrier.Arrive(completed + 1);
                            p.Execute();
                            s?.Execute();
                            _barrier.Arrive(completed + 1);
                            break;
                    }

                    completed++;
                    Volatile.Write(ref w.Completed, completed);
                }
            }
            catch (Exception fault)
            {
                _fault ??= fault;
                Volatile.Write(ref _faulted, 1);
                Volatile.Write(ref w.Completed, long.MaxValue >> 1);
            }
        }

        // Spins a moment for more, then sleeps until kicked; the flag and the fence keep a kick from passing unseen - see §2.8.
        private void Sleep(Worker w, long completed)
        {
            for (int i = 0; i < 400; i++)
            {
                if (Volatile.Read(ref _issued) != completed || Volatile.Read(ref _pauseRequested) != 0 || Volatile.Read(ref _stopping) != 0) return;
                Thread.SpinWait(50);
            }

            Volatile.Write(ref w.Sleeping, 1);
            Interlocked.MemoryBarrier();
            w.Wake.Reset();
            if (Volatile.Read(ref _issued) == completed && Volatile.Read(ref _pauseRequested) == 0 && Volatile.Read(ref _stopping) == 0) w.Wake.Wait();
            Volatile.Write(ref w.Sleeping, 0);
        }

        // The pause point is the furthest word any thread has reached when asked, inside a command or not; a thread short of it runs on, one at it stands - see §2.8.
        private void Stand(Worker w, long completed)
        {
            while (true)
            {
                long at = Volatile.Read(ref _pauseAt);
                if (at >= completed || Interlocked.CompareExchange(ref _pauseAt, completed, at) == at) break;
            }

            if (Volatile.Read(ref _pauseAt) > completed) return;

            Volatile.Write(ref w.Standing, completed);
            SpinWait spin = default;
            while (Volatile.Read(ref _pauseRequested) != 0 && Volatile.Read(ref _pauseAt) == completed && Volatile.Read(ref _stopping) == 0) spin.SpinOnce(-1);
            Volatile.Write(ref w.Standing, -1);
        }

        // A waiter at a barrier is inside its word and cannot stand short of it, so a pause point short of it is raised to it - see §2.8.
        private void RaisePauseTo(long word)
        {
            if (Volatile.Read(ref _pauseRequested) == 0) return;
            long at = Volatile.Read(ref _pauseAt);
            if (at >= 0 && at < word && Interlocked.CompareExchange(ref _pauseAt, word, at) == at) Interlocked.Increment(ref PausesRaised);
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

            if (_workers.Length > 0)
            {
                Kick();
                while (true)
                {
                    Rethrow();
                    long at = Volatile.Read(ref _pauseAt);
                    bool standing = at >= 0;
                    foreach (Worker worker in _workers) standing &= Volatile.Read(ref worker.Standing) == at;
                    if (standing) break;
                    spin.SpinOnce(-1);
                }

                Assemble();
                return;
            }

            while (Volatile.Read(ref _draining) != 0 && Volatile.Read(ref _paused) == 0) spin.SpinOnce(-1);
            Rethrow();
        }

        public void Resume()
        {
            Volatile.Write(ref _pauseRequested, 0);
            Volatile.Write(ref _pauseAt, -1);
            if (Completed() < _issued || _workers.Length > 0) Kick();
        }

        // The words handed over and not yet run, for a state written while the thread stands - see §2.7.
        public void WritePending(System.IO.BinaryWriter w)
        {
            long completed = Completed(), issued = _issued;
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
            Processor.Configure(0, 1);
            try
            {
                for (int i = 0; i < count; i++)
                {
                    ulong word = r.ReadUInt64();
                    Processor.Accept(word);
                    _scaledProcessor?.Accept(word);
                }
            }
            finally
            {
                _replaying = false;
                Processor.Configure(0, _workers.Length > 0 ? _workerCount : 1);
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
            long completed = Completed();
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

            // A read inside one doubleword is written over only by draws, so it waits for the last pending draw whose box holds it, if one does - see Mars_Rdp.md §2.6.3.
            long goal = mark == Idle ? _issued : mark;
            if (!write && ((from ^ (to - 1)) & ~7L) == 0)
            {
                long holding = LastHolding(from, completed);
                if (holding != Idle && holding < goal)
                {
                    ReadsNarrowed++;
                    if (holding <= completed) { ReadsFreed++; return false; }
                    goal = holding;
                }
            }

            if (_taking && site == 0) site = 10;
            long started = System.Diagnostics.Stopwatch.GetTimestamp();
            WaitUntil(goal);
            if (mark != Idle && goal >= mark) Clear(page, mark);
            WaitsPerPage[page]++;
            long took = System.Diagnostics.Stopwatch.GetTimestamp() - started;
            WaitsPerSite[site]++; TicksPerSite[site] += took;
            if (WaitsLogged < WaitLog.Length && took > System.Diagnostics.Stopwatch.Frequency / 20000) WaitLog[WaitsLogged++] = (site, (uint)page << 12, mark == Idle, took, _issued - Completed(), _colorImage, _depthImage, _textureImage);
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
            bool waited = Completed() < _issued;
            WaitUntil(_issued);
            if (_workers.Length > 0) Assemble();
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
            if (Completed() < words) Kick();

            SpinWait spin = default;
            while (Completed() < words)
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

        public void Touched(uint physical, long word) => Verify(physical, word, write: false);

        public void Wrote(uint physical, long word) => Verify(physical, word, write: true);

        private void Verify(uint physical, long word, bool write)
        {
            if (physical >= (uint)_bus.Rdram.Length) return;
            long[] marks = write ? _writeMarks : _marks;

            if (Volatile.Read(ref marks[physical >> 12]) < word)
            {
                _fault ??= new InvalidOperationException($"The display processor {(write ? "wrote" : "read")} {physical:X6} in a page its interface did not mark before word {word}.");
                return;
            }

            if (!Covers(physical, word, write)) _fault ??= new InvalidOperationException($"The display processor {(write ? "wrote" : "read")} {physical:X6} outside every range its interface marked for word {word}. {Nearest(physical, word)}. {Ranges()}");

            // A read waits only for the draws whose boxes hold it, so every byte a draw writes must be in its own box - see Mars_Rdp.md §2.6.3.
            if (write && !BoxHolds(physical, word)) _fault ??= new InvalidOperationException($"The display processor wrote {physical:X6} outside the box its interface recorded for the draw ending at word {word}.");
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

        // The box of the draw ending at the word, found newest first; a word that ends no recorded draw, or one the ring has lost, is not the boxes' to judge.
        private bool BoxHolds(uint physical, long word)
        {
            long appended = Volatile.Read(ref _boxesAppended);
            for (long i = appended - 1; i >= Math.Max(appended - BoxCount, 0); i--)
            {
                Box box = _boxes[(int)(i & (BoxCount - 1))];
                if (Volatile.Read(ref _boxesAppended) - i > BoxCount) return true;
                if (box.End < word) return true;
                if (box.End == word) return Holds(box, physical, physical + 1);
            }

            return true;
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
            _gatheredBeforeTheRing = _shadowTaken > 0;
            (_colorImage, _colorWidth, _colorBytes, _depthImage, _textureImage, _textureWidth, _textureSize, _scissorTop, _scissorBottom, _scissorRight) = Processor.Images;
            _drawn = false;
            (_scissorTopRaw, _scissorBottomRaw, ulong modes) = Processor.Bounds;
            _shadowCycle = (int)(modes >> 52) & 3;
            _shadowDepth = ((modes >> 4) & 3) != 0;
            _shadowDepthUpdate = ((modes >> 5) & 1) != 0;
            _colorExtent.Index = _depthExtent.Index = -1;
            _issued = _completed = Completed();
            foreach (Worker worker in _workers)
            {
                worker.Completed = _issued;
                if (worker.Index > 0) worker.Processor.CopyStateFrom(Processor);
            }

            ResetScaled();
        }
    }
}
