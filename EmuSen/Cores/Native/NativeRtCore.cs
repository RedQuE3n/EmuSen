using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Native
{
    // Where a NativeDebugBridge plugs in: when armed, it runs the frame instead of advance, and false is a halt at <haltedAt> - see EmuSen_NativeCores.md §4.4.
    public interface INativeDebugBridge
    {
        bool Armed { get; }
        bool RunFrame(bool resuming, out int haltedAt);
    }

    // The ICore body over the common native interface; a console supplies its data and its differences - see EmuSen_NativeCores.md §4.3, §4.5.
    public abstract class NativeRtCore<TMachine> : ICore, ICheatRegistryHost, IStateFormat, IFrameBufferPool, IFrameProfiler, IEngineFeatures, IDisposable where TMachine : NativeMachine
    {
        protected TMachine? _machine;
        private readonly byte[] _frame;
        private readonly FrameBufferLending _lending = new();
        private bool _skipRendering;
        private int _patchVersion = -1;
        private BatterySave _battery = BatterySave.None;
        private int _audioLimitSent = -1;
        private object? _audioLimitMachine;
        private readonly uint[] _buttons;
        private byte[] _stateBuffer = Array.Empty<byte>();
        private double _lastFrameMs;

        protected NativeRtCore(int frameBytes, int ports)
        {
            _frame = new byte[frameBytes];
            _buttons = new uint[ports];
        }

        // What the console's C# part decides before the library sees the image: the oracle's refusals, the battery file and the create-time settings.
        protected abstract (BatterySave Battery, string Settings) Prepare(string path, byte[] image);

        protected abstract TMachine CreateMachine(byte[] image, string settings, IReadOnlyList<(uint Which, byte[] Data)> files);

        // After a machine is adopted: the mirror's load, and whatever else the console keeps per game.
        protected virtual void Loaded(string path) { }

        // The C# core the debugger reads, refreshed from the machine's state; its registries are this core's.
        protected abstract ICore MirrorCore { get; }
        protected abstract bool MirrorLoaded { get; }
        protected abstract CheatRegistry MirrorCheats { get; set; }

        // The C# core's space names, numbered as the core's ABI numbers them, and the one whose reads are the CPU's.
        protected abstract IReadOnlyList<string> SpaceNames { get; }
        protected virtual int CpuBusSpace => -1;

        // The addresses ResolveRomPatches' list is kept to, inclusive.
        protected abstract long PatchLow { get; }
        protected abstract long PatchHigh { get; }

        // The pad bit a button is, or -1; and the machine's port for a frontend's, or -1 for one the console ignores.
        protected abstract int ButtonBit(PadButton button);
        protected virtual int PortFor(int port) => Math.Clamp(port, 0, _buttons.Length - 1);

        protected abstract uint MuteMask();

        // The C# core's refusals, with its messages, before the bytes reach the machine.
        protected abstract void CheckState(byte[] state);

        // Stage 5's bridge; null runs every frame through advance.
        protected INativeDebugBridge? DebugBridge { get; set; }

        public abstract string CoreName { get; }
        public abstract int ScreenWidth { get; }
        public abstract int ScreenHeight { get; }
        public abstract double FrameRateHz { get; }
        public abstract IReadOnlyList<PadButton> SupportedButtons { get; }
        public abstract int StateVersion { get; }
        public abstract FrameLogRegistry FrameLog { get; }
        public abstract BreakpointRegistry Breakpoints { get; }

        // A rewind capture is kept on every engine of this host until one withholds it - see EmuSen_NativeCores.md §5.2.
        public virtual EngineFeatures Features => EngineFeatures.All;

        public int AudioSampleRate => _machine?.AudioSampleRate ?? 44100;
        public bool IsRomLoaded => _machine != null;
        public long TotalFrames => _machine?.TotalFrames ?? 0;

        // One phase unless the core reports its own: the frame is one call, timed here.
        public IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases => new[] { ("frame", _lastFrameMs) };

        public double LastFrameMilliseconds => _lastFrameMs;

        public TMachine Machine => _machine ?? throw new InvalidOperationException("No ROM is loaded.");

        // Halted in front of a breakpoint, with the frame left open for the next RunFrame to resume, as the C# cores keep it.
        public bool IsHaltedAtBreakpoint { get; private set; }
        public int HaltedAddress { get; private set; }

        // A console's RESET, which the C# cores let end a halt.
        protected void ClearHalt() => IsHaltedAtBreakpoint = false;

        public CheatRegistry Cheats
        {
            get => MirrorCheats;
            set
            {
                MirrorCheats = value;
                _patchVersion = -1;
            }
        }

        public bool SkipRendering
        {
            get => _skipRendering;
            set
            {
                _skipRendering = value;
                _machine?.SetOptions(value);
            }
        }

        // The console's rules first, then the machine with its battery file, then the old one freed and the per-game state reset.
        public void LoadRom(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            var (battery, settings) = Prepare(path, image);
            byte[]? saved = battery.Read();
            var files = saved is null ? Array.Empty<(uint, byte[])>() : new[] { (0u, saved) };

            TMachine machine = CreateMachine(image, settings, files);
            machine.SetOptions(_skipRendering);
            for (int port = 0; port < _buttons.Length; port++) machine.SetButtons(port, _buttons[port]);

            _machine?.Dispose();
            _machine = machine;
            _battery = battery;
            _patchVersion = -1;
            IsHaltedAtBreakpoint = false;
            Loaded(path);
        }

        public void SetButton(int port, PadButton button, bool pressed)
        {
            int bit = ButtonBit(button);
            if (bit < 0) return;
            int pad = PortFor(port);
            if (pad < 0) return;
            _buttons[pad] = pressed ? _buttons[pad] | (1u << bit) : _buttons[pad] & ~(1u << bit);
            _machine?.SetButtons(pad, _buttons[pad]);
        }

        public void RunFrame()
        {
            if (_machine is null) throw new InvalidOperationException("RunFrame() called before LoadRom().");
            RefreshRomPatches();
            SyncAudioLimit();
            bool resuming = IsHaltedAtBreakpoint;
            IsHaltedAtBreakpoint = false;
            long start = Stopwatch.GetTimestamp();
            if (DebugBridge is { Armed: true } bridge)
            {
                bool ended = bridge.RunFrame(resuming, out int haltedAt);
                _lastFrameMs = Stopwatch.GetElapsedTime(start).TotalMilliseconds;
                if (!ended)
                {
                    IsHaltedAtBreakpoint = true;
                    HaltedAddress = haltedAt;
                    return;
                }
            }
            else
            {
                _machine.Advance();
                _lastFrameMs = Stopwatch.GetElapsedTime(start).TotalMilliseconds;
            }
            EndFrame();
        }

        // Mercury's frame-end order, which Moon's is too: the frame log, the cheats, the frame's notice, the periodic battery save.
        protected virtual void EndFrame()
        {
            FrameLog.RecordFrame(TotalFrames, ReadForFrameLog);
            ApplyCheats();
            Breakpoints.NoteFrame(TotalFrames);
            if (BatterySave.IsFlushFrame(TotalFrames)) SaveSram();
        }

        public void ApplyCheats() => Cheats.ApplyAll(ReadSpace, WriteSpace);

        // A copy in an array no one else holds - see EmuSen_Multicore.md §16.
        public byte[] GetFrameBufferRgba()
        {
            _machine?.CopyFrame(_frame);
            byte[] buffer = _lending.Lend(_frame.Length);
            _frame.AsSpan().CopyTo(buffer);
            return buffer;
        }

        public void ReturnFrameBuffer(byte[] buffer) => _lending.Return(buffer);

        public FrameBufferLending FrameBuffers => _lending;

        public short[] DequeueAudioSamples(int maxFrames) => _machine?.DrainAudio(maxFrames) ?? Array.Empty<short>();

        // The save opened at load, which no state can change; the core says how long its battery RAM is, and empty where there is none.
        public void SaveSram()
        {
            if (_machine is null || _battery.Path is null) return;
            var (data, _) = _machine.Battery(0);
            if (data.Length == 0) return;
            _battery.Write(data);
            _machine.BatterySaved(0);
        }

        public void SaveState(string path)
        {
            using var stream = File.Create(path);
            SaveState(stream);
        }

        public void LoadState(string path)
        {
            using var stream = File.OpenRead(path);
            LoadState(stream);
        }

        // Written through one array reused while the state's size stands, as a session's does - see EmuSen_NativeCores.md §3.9.
        public void SaveState(Stream stream)
        {
            if (_machine is null) throw new InvalidOperationException("SaveState() called before LoadRom().");
            int size = _machine.StateSize;
            if (_stateBuffer.Length != size) _stateBuffer = new byte[size];
            _machine.Save(_stateBuffer);
            stream.Write(_stateBuffer);
        }

        // The C# core's refusals with its messages; a truncated state is refused whole, where C# stops part-way.
        public void LoadState(Stream stream)
        {
            if (_machine is null) throw new InvalidOperationException("LoadState() called before LoadRom().");
            using var copy = new MemoryStream();
            stream.CopyTo(copy);
            byte[] state = copy.ToArray();
            CheckState(state);
            _machine.Load(state);
        }

        protected int SpaceNumber(string spaceName)
        {
            IReadOnlyList<string> names = SpaceNames;
            for (int i = 0; i < names.Count; i++) if (names[i] == spaceName) return i;
            return -1;
        }

        // A read of the CPU's bus first brings the ROM patches up to the registry, so it answers as the C# core would (§9 Q11).
        public byte ReadSpace(string spaceName, int address)
        {
            int space = SpaceNumber(spaceName);
            if (_machine is null || space < 0) return 0;
            if (space == CpuBusSpace) RefreshRomPatches();
            Span<byte> one = stackalloc byte[1];
            _machine.ReadSpace(space, address, one);
            return one[0];
        }

        public virtual void WriteSpace(string spaceName, int address, byte value)
        {
            int space = SpaceNumber(spaceName);
            if (_machine is null || space < 0) return;
            _machine.WriteSpace(space, address, stackalloc byte[] { value });
        }

        public int SpaceSize(string spaceName)
        {
            int space = SpaceNumber(spaceName);
            return _machine is null || space < 0 ? 0 : _machine.SpaceSize(space);
        }

        private long ReadForFrameLog(string spaceName, int address, int width)
        {
            long value = 0;
            for (int i = 0; i < width; i++) value |= (long)ReadSpace(spaceName, address + i) << (8 * i);
            return value;
        }

        public void SyncMirror()
        {
            if (_machine is null || !MirrorLoaded) return;
            MirrorCore.LoadState(new MemoryStream(_machine.Save()));
        }

        public void SyncMutes()
        {
            if (_machine is null || !MirrorLoaded) return;
            _machine.SetMutes(MuteMask());
        }

        // AudioSettings.AudioBufferMaxSamples, sent when it differs from what this machine was last told.
        private void SyncAudioLimit()
        {
            int limit = EmuSen.Audio.AudioSettings.AudioBufferMaxSamples;
            if (_machine is null || (_audioLimitSent == limit && ReferenceEquals(_audioLimitMachine, _machine))) return;
            _machine.SetAudioLimit(limit);
            _audioLimitSent = limit;
            _audioLimitMachine = _machine;
        }

        // CheatRegistry.ResolveRomPatches within the console's range, sent as triples when the registry moves - see EmuSen_NativeCores.md §3.12.
        private void RefreshRomPatches()
        {
            CheatRegistry cheats = Cheats;
            int version = cheats.Version;
            if (version == _patchVersion || _machine is null) return;
            _patchVersion = version;
            _machine.SetRomPatches(RomPatchTriples(cheats, PatchLow, PatchHigh));
        }

        // (address, value, compare) for each byte ResolveRomPatches lists in [lo, hi], in its order; uint.MaxValue is no compare.
        public static uint[] RomPatchTriples(CheatRegistry cheats, long lo, long hi)
        {
            var words = new List<uint>();
            foreach (var patch in cheats.ResolveRomPatches(hi + 1))
            {
                if (patch.Address < lo) continue;
                words.Add(patch.Address);
                words.Add(patch.Value);
                words.Add(patch.Compare is byte compare ? compare : uint.MaxValue);
            }
            return words.ToArray();
        }

        public virtual void Dispose()
        {
            _machine?.Dispose();
            _machine = null;
            _lending.Close();
        }
    }
}
