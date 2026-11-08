using System.Text;
using EmuSen.Crystal.Client.Contract;
using EmuSen.DianaSh.DianaSh.Bin.Native;
using EmuSen.Galaxia.Input;
using static EmuSen.Crystal.Client.CrystalRuntime;

namespace EmuSen.Crystal.Client
{
    // One open session: the frame call and its hand-over, input, speed, rewind, states, settings, and the debugger through the session's table - see EmuSen_Backend.md §3.5 and §14.
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

        // Since 1.4: the current machine's ring for the presenter to borrow, zero for none; asked again after a frame, since one whose maxima grew is replaced.
        public nint PictureRing() => Contract.Crystal.PictureRing(Rt, Id);

        // Since 1.4: one frame advance, the session held paused after it - see EmuSen_Backend.md §33.3.
        public void FrameAdvance() => Check(Contract.Crystal.FrameAdvance(Rt, Id));

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

        // A port's motors, strong and weak, for the player's pad each frame; zero where the console or the switch says none - see EmuSen_Input.md §9.
        public (ushort Strong, ushort Weak) Rumble(int port)
        {
            ushort strong, weak;
            Check(Contract.Crystal.InputRumble(Rt, Id, (uint)port, &strong, &weak));
            return (strong, weak);
        }

        // The player's rumble switch turned while the game runs; the console's own half of the gate stays as it opened.
        public void RumbleSwitch(bool on) => Check(Contract.Crystal.InputRumbleSwitch(Rt, Id, on ? 1u : 0u));

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

        public void SaveState(string? path = null) => WithText(path, (p, l) => Table.Check(T->state_save(T->ctx, Id, (byte*)p, l)));

        public void LoadState(string? path = null) => WithText(path, (p, l) => Table.Check(T->state_load(T->ctx, Id, (byte*)p, l)));

        public string DefaultStatePath() => Table.Text((o, n) => T->state_default_path(T->ctx, Id, o, n));

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

        // The session's emusen_session_v1 table, for a shell's host to hand on and for the calls below: alive as long as the runtime.
        public nint ServiceTable(bool readOnly = false)
        {
            nint table = Contract.Crystal.SessionService(Rt, Id, readOnly ? CrystalAbi.ServiceReadOnly : 0);
            return table == 0 ? throw Failure(Rt == 0 ? CrystalSessionAbi.Null : CrystalSessionAbi.NoSession) : table;
        }

        private SessionTable? _table;

        // The full table, asked once while the runtime lives; a closed session's answers NO_SESSION in its own words.
        private SessionTable Table => Rt != 0 && _table is { } made ? made : _table = new SessionTable(ServiceTable());

        private EmuSenSessionV1* T => Table.T;

        public long Run(uint op, uint processor = 0, long argument = 0) => Table.Check(T->run_control(T->ctx, Id, op, processor, argument));

        public string HaltState(uint processor = 0) => Table.Text((o, n) => T->halt_state(T->ctx, Id, processor, o, n));

        public string ServiceInfo() => Table.Text((o, n) => T->info(T->ctx, Id, o, n));

        public string Summary() => Table.Text((o, n) => T->summary(T->ctx, Id, o, n));

        public string Telemetry(uint kind) => Table.Text((o, n) => T->telemetry(T->ctx, Id, kind, o, n));

        public string Spaces() => Table.Text((o, n) => T->spaces(T->ctx, Id, o, n));

        // Bytes of a memory space from an address, read as the space reads any address.
        public void ReadSpace(string space, uint address, Span<byte> into)
        {
            byte[] name = Encoding.UTF8.GetBytes(space);
            SessionTable t = Table;
            fixed (byte* s = name)
            fixed (byte* p = into)
                t.Check(t.T->space_read(t.T->ctx, Id, s, (nuint)name.Length, address, p, (nuint)into.Length));
        }

        public long WatchAdd(string space, uint address, uint length, uint kind) =>
            WithText(space, (s, sl) => Table.Check(T->watch_add(T->ctx, Id, (byte*)s, sl, address, length, kind)));

        public string WatchList() => Table.Text((o, n) => T->watch_list(T->ctx, Id, o, n));

        public string Sprites() => Table.Text((o, n) => T->sprites(T->ctx, Id, o, n));

        public byte[] TileSheet() => Table.Bytes((o, n) => T->tile_sheet(T->ctx, Id, o, n));

        public byte[] PaletteSwatch() => Table.Bytes((o, n) => T->palette_swatch(T->ctx, Id, o, n));

        public byte[] AudioPeek() => Table.Bytes((o, n) => T->audio_peek(T->ctx, Id, o, n));

        // An op that answers a record, read with the copy-out protocol's two calls; a record-answering change is kept by the provider between them.
        public string CheatOp(uint op, string argsJson) =>
            WithText(argsJson, (a, al) => Table.Text((o, n) => T->cheat_op(T->ctx, Id, op, (byte*)a, al, o, n)));

        // An op that changes the list and answers a value, not a record, called once, since two calls would make the change twice - see EmuSen_Backend.md §14.13.
        public long CheatChange(uint op, string argsJson) =>
            WithText(argsJson, (a, al) => Table.Check(T->cheat_op(T->ctx, Id, op, (byte*)a, al, null, 0)));

        public string CheatCodecs() => Table.Text((o, n) => T->cheat_codecs(T->ctx, Id, o, n));

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
