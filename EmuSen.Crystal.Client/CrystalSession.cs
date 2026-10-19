using System.Text;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Galaxia.Input;
using static EmuSen.Crystal.Client.CrystalRuntime;

namespace EmuSen.Crystal.Client
{
    // One open session: the frame call and its hand-over, input, speed, rewind, states, settings and the debugger's twins - see EmuSen_Backend.md §3.5 and §14.
    public sealed unsafe class CrystalSession : IDisposable
    {
        internal CrystalSession(CrystalRuntime runtime, uint id)
        {
            Runtime = runtime;
            Id = id;
        }

        public CrystalRuntime Runtime { get; }
        public uint Id { get; }
        public bool Closed { get; private set; }

        private nint Rt => Runtime.Handle;

        // One frame and the frame-end order, the frame's facts into info; allocates nothing.
        public void Frame(ref CrystalFrameInfo info)
        {
            info.size = (uint)sizeof(CrystalFrameInfo);
            fixed (CrystalFrameInfo* p = &info) Check(Contract.Crystal.Frame(Rt, Id, p));
        }

        // The picture into the buffer, the bytes it needs returned; nothing is written to a buffer too short.
        public long CopyPicture(Span<byte> into)
        {
            fixed (byte* p = into) return Check(Contract.Crystal.PictureCopy(Rt, Id, into.IsEmpty ? null : p, (nuint)into.Length));
        }

        public long PictureBytes => Check(Contract.Crystal.PictureCopy(Rt, Id, null, 0));

        // Since 1.1: the current machine's lend for the presenter to borrow, zero for none; asked again after a frame, since a machine made again is lent afresh.
        public nint PictureLend() => Contract.Crystal.PictureLend(Rt, Id);

        // Since 1.1: the raw encoding kept from the next frame; null keeps none.
        public void RawSelect(string? encoding) => WithText(encoding, (e, el) => Check(Contract.Crystal.RawSelect(Rt, Id, (byte*)e, el)));

        // Since 1.1: the last frame's raw picture into the buffer and its shape; the bytes it needs, or -1 for none kept.
        public long RawCopy(Span<byte> into, out CrystalRawInfo info)
        {
            var shape = new CrystalRawInfo { size = (uint)sizeof(CrystalRawInfo) };
            long bytes;
            fixed (byte* p = into) bytes = Contract.Crystal.RawCopy(Rt, Id, &shape, into.IsEmpty ? null : p, (nuint)into.Length);
            info = shape;
            if (bytes == CrystalSessionAbi.Absent) return -1;
            return Check(bytes);
        }

        // Stereo frames of interleaved samples into the buffer, their rate out; the frames written.
        public int DrainAudio(Span<short> into, out uint rate)
        {
            uint r = 0;
            long frames;
            fixed (short* p = into) frames = Check(Contract.Crystal.AudioDrain(Rt, Id, into.IsEmpty ? null : p, (nuint)(into.Length / 2), &r));
            rate = r;
            return (int)frames;
        }

        public int AudioFramesReady => (int)Check(Contract.Crystal.AudioDrain(Rt, Id, null, 0, null));

        public bool Wait(int timeoutMs) => Check(Contract.Crystal.Wait(Rt, Id, (uint)Math.Max(0, timeoutMs))) != 0;

        public string PhaseNames() => Text((o, n) => Contract.Crystal.PhaseNames(Rt, Id, o, n));

        public int Phases(Span<double> into)
        {
            fixed (double* p = into) return (int)Check(Contract.Crystal.Phases(Rt, Id, into.IsEmpty ? null : p, (nuint)into.Length));
        }

        public void Button(int port, PadButton button, bool pressed) => Check(Contract.Crystal.InputButton(Rt, Id, (uint)port, (uint)button, pressed ? 1u : 0u));

        public void Axis(int port, PadAxis axis, double value) => Check(Contract.Crystal.InputAxis(Rt, Id, (uint)port, (uint)axis, value));

        public void Connected(int port, bool connected) => Check(Contract.Crystal.InputConnected(Rt, Id, (uint)port, connected ? 1u : 0u));

        public void Speed(uint percent, uint turboPercent = 300, uint slowPercent = 25, bool drawNone = false) =>
            Check(Contract.Crystal.SpeedSet(Rt, Id, percent, turboPercent, slowPercent, drawNone ? CrystalAbi.SpeedDrawNone : 0));

        public void SpeedMode(uint mode) => Check(Contract.Crystal.SpeedMode(Rt, Id, mode));

        public void Reset() => Check(Contract.Crystal.Reset(Rt, Id));

        public string Info() => Text((o, n) => Contract.Crystal.SessionInfo(Rt, Id, o, n));

        public void Rewind(bool enabled, uint intervalFrames = 0, ulong budgetBytes = 0, uint thumbnailWidth = 0) =>
            Check(Contract.Crystal.RewindSet(Rt, Id, enabled ? 1u : 0u, intervalFrames, budgetBytes, thumbnailWidth));

        public long RewindStep(uint count, ref CrystalFrameInfo info)
        {
            info.size = (uint)sizeof(CrystalFrameInfo);
            fixed (CrystalFrameInfo* p = &info) return Check(Contract.Crystal.RewindStep(Rt, Id, count, p));
        }

        public string RewindReel() => Text((o, n) => Contract.Crystal.RewindReel(Rt, Id, o, n));

        public byte[] RewindPicture(ulong frame) => Bytes((o, n) => Contract.Crystal.RewindPicture(Rt, Id, frame, o, n));

        public void RewindTo(ulong frame) => Check(Contract.Crystal.RewindTo(Rt, Id, frame));

        public void SaveState(string? path = null) => WithText(path, (p, l) => Check(Contract.Crystal.StateSave(Rt, Id, (byte*)p, l)));

        public void LoadState(string? path = null) => WithText(path, (p, l) => Check(Contract.Crystal.StateLoad(Rt, Id, (byte*)p, l)));

        public string DefaultStatePath() => Text((o, n) => Contract.Crystal.StateDefaultPath(Rt, Id, o, n));

        public string StateSlots() => Text((o, n) => Contract.Crystal.StateSlots(Rt, Id, o, n));

        public void SaveSlot(uint slot) => Check(Contract.Crystal.StateSlotSave(Rt, Id, slot));

        public void LoadSlot(uint slot) => Check(Contract.Crystal.StateSlotLoad(Rt, Id, slot));

        public void SaveResume() => Check(Contract.Crystal.ResumeSave(Rt, Id));

        public void LoadResume() => Check(Contract.Crystal.ResumeLoad(Rt, Id));

        public string? Setting(string key, bool game = true) =>
            WithText(key, (k, kl) => Optional(() => Text((o, n) => Contract.Crystal.SettingsGet(Rt, Id, game ? CrystalAbi.ScopeGame : CrystalAbi.ScopeConsole, null, 0, (byte*)k, kl, o, n))));

        public void SetSetting(string key, string? value, bool game) =>
            WithText(key, (k, kl) => WithText(value, (v, vl) => Check(Contract.Crystal.SettingsSet(Rt, Id, game ? CrystalAbi.ScopeGame : CrystalAbi.ScopeConsole, null, 0, (byte*)k, kl, (byte*)v, vl))));

        public string SettingsSchema(string? engine = null) =>
            WithText(engine, (e, el) => Text((o, n) => Contract.Crystal.SettingsSchema(Rt, Id, null, 0, (byte*)e, el, o, n)));

        public string SaveMedia() => Text((o, n) => Contract.Crystal.SaveMedia(Rt, Id, o, n));

        public void InsertDisc(string path) => WithText(path, (p, l) => Check(Contract.Crystal.DiscInsert(Rt, Id, (byte*)p, l)));

        public void EjectDisc() => Check(Contract.Crystal.DiscEject(Rt, Id));

        public void ChangeDisc(uint index) => Check(Contract.Crystal.DiscChange(Rt, Id, index));

        public void Lid(bool open) => Check(Contract.Crystal.DiscLid(Rt, Id, open ? 1u : 0u));

        public string StartRecording(string baseDir, uint stride) =>
            WithText(baseDir, (b, bl) => Once((o, n) => Contract.Crystal.RecordStart(Rt, Id, (byte*)b, bl, stride, o, n)));

        public string StopRecording() => Once((o, n) => Contract.Crystal.RecordStop(Rt, Id, o, n));

        // The session's emusen_session_v1 table, for a shell's host to hand on: the twins' functions, alive as long as the session.
        public nint ServiceTable(bool readOnly = false)
        {
            nint table = Contract.Crystal.SessionService(Rt, Id, readOnly ? CrystalAbi.ServiceReadOnly : 0);
            return table == 0 ? throw Failure(CrystalSessionAbi.NoSession) : table;
        }

        public long Run(uint op, uint processor = 0, long argument = 0) => Check(Contract.Crystal.RunControl(Rt, Id, op, processor, argument));

        public string HaltState(uint processor = 0) => Text((o, n) => Contract.Crystal.HaltState(Rt, Id, processor, o, n));

        public string ServiceInfo() => Text((o, n) => Contract.Crystal.Info(Rt, Id, o, n));

        public string Summary() => Text((o, n) => Contract.Crystal.Summary(Rt, Id, o, n));

        public string Telemetry(uint kind) => Text((o, n) => Contract.Crystal.Telemetry(Rt, Id, kind, o, n));

        public string Spaces() => Text((o, n) => Contract.Crystal.Spaces(Rt, Id, o, n));

        // Bytes of a memory space from an address, read as the space reads any address.
        public void ReadSpace(string space, uint address, Span<byte> into)
        {
            byte[] name = Encoding.UTF8.GetBytes(space);
            fixed (byte* s = name)
            fixed (byte* p = into)
                Check(Contract.Crystal.SpaceRead(Rt, Id, s, (nuint)name.Length, address, p, (nuint)into.Length));
        }

        public long WatchAdd(string space, uint address, uint length, uint kind) =>
            WithText(space, (s, sl) => Check(Contract.Crystal.WatchAdd(Rt, Id, (byte*)s, sl, address, length, kind)));

        public string WatchList() => Text((o, n) => Contract.Crystal.WatchList(Rt, Id, o, n));

        public string Sprites() => Text((o, n) => Contract.Crystal.Sprites(Rt, Id, o, n));

        public byte[] TileSheet() => Bytes((o, n) => Contract.Crystal.TileSheet(Rt, Id, o, n));

        public byte[] PaletteSwatch() => Bytes((o, n) => Contract.Crystal.PaletteSwatch(Rt, Id, o, n));

        public byte[] AudioPeek() => Bytes((o, n) => Contract.Crystal.AudioPeek(Rt, Id, o, n));

        // An op that answers a record, read with the copy-out protocol's two calls; a record-answering change is kept by the provider between them.
        public string CheatOp(uint op, string argsJson) =>
            WithText(argsJson, (a, al) => Text((o, n) => Contract.Crystal.CheatOp(Rt, Id, op, (byte*)a, al, o, n)));

        // An op that changes the list and answers a value, not a record, called once, since two calls would make the change twice - see EmuSen_Backend.md §14.13.
        public long CheatChange(uint op, string argsJson) =>
            WithText(argsJson, (a, al) => Check(Contract.Crystal.CheatOp(Rt, Id, op, (byte*)a, al, null, 0)));

        public string CheatCodecs() => Text((o, n) => Contract.Crystal.CheatCodecs(Rt, Id, o, n));

        public void Close()
        {
            if (Closed) return;
            Closed = true;
            Check(Contract.Crystal.Close(Rt, Id));
        }

        public void Dispose()
        {
            if (Closed || Runtime.Handle == 0) return;
            Close();
        }
    }

    // A subscription's events, drained by the client, or read from a channel a background reader fills - see EmuSen_Backend.md §3.5.
    public sealed unsafe class CrystalSubscription : IDisposable
    {
        private System.Threading.Channels.Channel<string>? _channel;
        private CancellationTokenSource? _stop;
        private Task? _reader;

        internal CrystalSubscription(nint handle) => Handle = handle;

        public nint Handle { get; private set; }

        // The waiting events as a JSON array, removed once taken.
        public string Drain() => Text((o, n) => Contract.Crystal.SubscriptionDrain(Handle, o, n));

        public bool Wait(int timeoutMs) => Check(Contract.Crystal.SubscriptionWait(Handle, (uint)Math.Max(0, timeoutMs))) != 0;

        // Each drain's array as it arrives, filled by a reader that waits on the subscription.
        public System.Threading.Channels.ChannelReader<string> Reader
        {
            get
            {
                if (_channel is not null) return _channel.Reader;
                _channel = System.Threading.Channels.Channel.CreateUnbounded<string>();
                _stop = new CancellationTokenSource();
                CancellationToken stop = _stop.Token;
                _reader = Task.Run(() =>
                {
                    while (!stop.IsCancellationRequested)
                        if (Wait(100)) _channel.Writer.TryWrite(Drain());
                    _channel.Writer.TryComplete();
                });
                return _channel.Reader;
            }
        }

        public void Dispose()
        {
            if (Handle == 0) return;
            _stop?.Cancel();
            _reader?.Wait();
            Contract.Crystal.Unsubscribe(Handle);
            Handle = 0;
        }
    }
}
