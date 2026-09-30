using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Native
{
    // MercuryRT's ICore body over its per-core ABI, until it moves to NativeRtCore at the common interface's step 4 - see EmuSen_NativeCores.md §7.
    public abstract class LegacyNativeRtCore<TMachine> : ICore, ICheatRegistryHost, IStateFormat, IFrameBufferPool, IDisposable where TMachine : LegacyNativeMachine
    {
        protected TMachine? _machine;
        private readonly byte[] _frame;
        private readonly FrameBufferLending _lending = new();
        private bool _skipRendering;
        private int _patchVersion = -1;
        private BatterySave _battery = BatterySave.None;
        private int _batteryLength;
        private int _audioLimitSent = -1;
        private object? _audioLimitMachine;

        protected LegacyNativeRtCore(int frameBytes) => _frame = new byte[frameBytes];

        // The C# core the debugger reads, refreshed from the machine's state; its registries are this core's.
        protected abstract ICore MirrorCore { get; }

        protected abstract bool MirrorLoaded { get; }

        protected abstract CheatRegistry MirrorCheats { get; set; }

        // The C# core's space names, numbered as the C ABI numbers them.
        protected abstract IReadOnlyList<string> SpaceNames { get; }

        // The battery RAM's space number, the same on both 8-bit machines.
        protected virtual int BatterySpace => 2;

        // The addresses CheatRegistry.TryPatchRom is asked about, inclusive.
        protected abstract int PatchLow { get; }
        protected abstract int PatchHigh { get; }

        // The mirror's mutes as a mask, bit n for channel n.
        protected abstract uint MuteMask();

        // The C# core's refusals, with its messages, before the bytes reach the machine.
        protected abstract void CheckState(byte[] state);

        public abstract string CoreName { get; }
        public abstract int ScreenWidth { get; }
        public abstract int ScreenHeight { get; }
        public abstract double FrameRateHz { get; }
        public abstract IReadOnlyList<PadButton> SupportedButtons { get; }
        public abstract int StateVersion { get; }
        public abstract FrameLogRegistry FrameLog { get; }
        public abstract BreakpointRegistry Breakpoints { get; }

        public virtual int AudioSampleRate => 44100;
        public bool IsRomLoaded => _machine != null;
        public long TotalFrames => _machine?.TotalFrames ?? 0;

        public abstract void LoadRom(string path);
        public abstract void RunFrame();
        public abstract void SetButton(int port, PadButton button, bool pressed);

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

        // The new machine, its battery save written in and the old one freed; the registries' per-game state is the subclass's.
        protected void Adopt(TMachine machine, BatterySave battery, byte[]? saved, int batteryLength)
        {
            if (saved is not null) machine.WriteSpace(BatterySpace, 0, saved.AsSpan(0, Math.Min(saved.Length, batteryLength)));
            machine.SetOptions(_skipRendering);

            _machine?.Dispose();
            _machine = machine;
            _battery = battery;
            _batteryLength = batteryLength;
            _patchVersion = -1;
        }

        // Before the machine runs: the ROM patches when the registry moved, and the audio limit when the setting did.
        protected void BeforeFrame()
        {
            RefreshRomPatches();
            SyncAudioLimit();
        }

        // Mercury's frame-end order, which Moon's is too: the frame log, the cheats, the frame's notice, the periodic battery save.
        protected void EndFrame()
        {
            FrameLog.RecordFrame(TotalFrames, ReadForFrameLog);
            ApplyCheats();
            Breakpoints.NoteFrame(TotalFrames);
            if (BatterySave.IsFlushFrame(TotalFrames)) SaveSram();
        }

        public void ApplyCheats() => Cheats.ApplyAll(ReadSpace, WriteSpace);

        // A copy in an array no one else holds, as MarsRT's - see EmuSen_Multicore.md §16.
        public byte[] GetFrameBufferRgba()
        {
            _machine?.CopyFrame(_frame);
            byte[] buffer = _lending.Lend(_frame.Length);
            _frame.AsSpan().CopyTo(buffer);
            return buffer;
        }

        public void ReturnFrameBuffer(byte[] buffer) => _lending.Return(buffer);

        // What the lending has done, for the tests.
        public FrameBufferLending FrameBuffers => _lending;

        public short[] DequeueAudioSamples(int maxFrames) => _machine?.DrainAudio(maxFrames) ?? Array.Empty<short>();

        // The save opened at load, which no state can change.
        public void SaveSram()
        {
            if (_machine is null || _battery.Path is null) return;
            var ram = new byte[_batteryLength];
            _machine.ReadSpace(BatterySpace, 0, ram);
            _battery.Write(ram);
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

        public void SaveState(Stream stream)
        {
            if (_machine is null) throw new InvalidOperationException("SaveState() called before LoadRom().");
            stream.Write(_machine.Save());
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

        public byte ReadSpace(string spaceName, int address)
        {
            int space = SpaceNumber(spaceName);
            if (_machine is null || space < 0) return 0;
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

        // CheatRegistry.TryPatchRom flattened to a table per patched address, rebuilt when the registry changes - see Moon_Native.md §2.4.
        private void RefreshRomPatches()
        {
            CheatRegistry cheats = Cheats;
            int version = cheats.Version;
            if (version == _patchVersion || _machine is null) return;
            _patchVersion = version;
            var (addresses, tables) = RomPatchTable.Build(cheats, PatchLow, PatchHigh);
            _machine.SetRomPatches(addresses, tables);
        }

        public virtual void Dispose()
        {
            _machine?.Dispose();
            _machine = null;
            _lending.Close();
        }
    }
}
