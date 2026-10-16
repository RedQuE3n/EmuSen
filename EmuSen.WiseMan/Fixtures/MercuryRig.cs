using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia.Input;
using Sm83 = EmuSen.Cores.Nintendo.Mercury.Cpu.Core.Cpu;

namespace EmuSen.WiseMan.Fixtures
{
    // The two Game Boy engines a machine-level test can run on.
    public enum MercuryEngine
    {
        Mercury,
        MercuryRt,
    }

    // One Game Boy engine under WiseMan's machine tests: reads through a C# view of its state, setup through the bus or a state round trip, time through frames, steps and ticks - see Mercury_Native.md §12.1.
    public abstract class MercuryRig : IDisposable
    {
        private readonly string _rom;
        private readonly bool _temporary;

        protected MercuryRig(string rom, bool temporary)
        {
            _rom = rom;
            _temporary = temporary;
            Rom = File.ReadAllBytes(rom);
        }

        protected string RomPath => _rom;

        // The cartridge image, whose header picks the layout listing a state follows.
        public byte[] Rom { get; }

        // An image written to a temporary file the rig deletes.
        public static MercuryRig Create(MercuryEngine engine, byte[] rom, GbModel model = GbModel.Auto) => Open(engine, SyntheticGbRom.WriteTemp(rom), model, temporary: true);

        // A ROM file in place, which the rig leaves alone.
        public static MercuryRig Open(MercuryEngine engine, string path, GbModel model = GbModel.Auto, bool temporary = false) =>
            engine == MercuryEngine.Mercury ? new CsharpRig(path, temporary, model) : new RustRig(path, temporary, model);

        public abstract MercuryEngine Engine { get; }

        // The engine as a frontend holds it.
        public abstract ICore Core { get; }

        // The machine as C# objects: the core itself on C#, a view of MercuryRT's state on Rust, which a field edit reaches through a state round trip.
        public abstract MercuryCore View { get; }

        public MemoryBus Bus => View.Bus!;
        public Sm83 Cpu => View.Cpu!;
        public Cartridge Cart => View.Cart!;

        // MemoryBus.Tick: the machine without its CPU.
        public abstract void Tick(int cycles);

        // The CPU's read and write, through the bus's decode and blocking.
        public abstract byte Read(int address);
        public abstract void Write(int address, byte value);

        // MercuryCore.ReadSpace: a named space as the debugger reads it.
        public abstract byte ReadSpace(string space, int address);

        // One instruction with the frame loop's acknowledge and stall; its T-cycles; an illegal opcode throws NotSupportedException.
        public abstract int Step();

        // MemoryBus.Stop: the speed switch KEY1 armed.
        public abstract void Stop();

        public abstract IReadOnlyList<byte> SerialLog { get; }

        public string SerialText => string.Concat(SerialLog.Select(b => (char)b));

        public abstract void SetChannelMuted(int channel, bool muted);
        public abstract bool IsChannelMuted(int channel);

        public abstract BreakpointRegistry Breakpoints { get; }

        // The engine's debugger, as a frontend makes it.
        public abstract IDebugTarget CreateDebugTarget();
        public bool IsHaltedAtBreakpoint => Core.IsHaltedAtBreakpoint;
        public int HaltedAddress => Core.HaltedAddress;

        // The engine before any ROM is loaded.
        public static ICore Unloaded(MercuryEngine engine) => engine == MercuryEngine.Mercury ? new MercuryCore() : new MercuryRtCore();

        public virtual void RunFrame() => Core.RunFrame();

        // LoadRom again on the same image, which resets the bus.
        public void Reload() => Core.LoadRom(RomPath);
        public long TotalFrames => Core.TotalFrames;
        public string CoreName => Core.CoreName;
        public int AudioSampleRate => Core.AudioSampleRate;
        public byte[] GetFrameBufferRgba() => Core.GetFrameBufferRgba();
        public short[] DequeueAudioSamples(int maxFrames) => Core.DequeueAudioSamples(maxFrames);
        public void SetButton(int port, PadButton button, bool pressed) => Core.SetButton(port, button, pressed);
        public bool SkipRendering { set => Core.SkipRendering = value; }
        public virtual void SaveState(Stream stream) => Core.SaveState(stream);
        public abstract void SaveSram();
        public virtual void LoadState(Stream stream) => Core.LoadState(stream);

        public byte[] State()
        {
            using var stream = new MemoryStream();
            SaveState(stream);
            return stream.ToArray();
        }

        // A field of the engine's state by its listing path, read through the committed layout with no C# machine - see Mercury_Native.md §14.5.
        public long Field(string path)
        {
            byte[] state = State();
            return Listed(state).Get(state, path);
        }

        public bool Flag(string path) => Field(path) != 0;

        public byte FieldByte(string path, int index)
        {
            byte[] state = State();
            return Listed(state).Bytes(state, path)[index];
        }

        public byte[] FieldBytes(string path)
        {
            byte[] state = State();
            return Listed(state).Bytes(state, path).ToArray();
        }

        // A field edit as a state round trip: the engine's state, the field rewritten through the listing, loaded back.
        public void SetField(string path, long value)
        {
            byte[] state = State();
            Listed(state).Set(state, path, value);
            LoadState(new MemoryStream(state));
        }

        public void SetFieldByte(string path, int index, byte value)
        {
            byte[] state = State();
            Listed(state).Bytes(state, path)[index] = value;
            LoadState(new MemoryStream(state));
        }

        // The listing for this state; while C# Mercury is on main its own classes read the state too, and the two readings must agree.
        protected MercuryStateLayout Listed(byte[] state)
        {
            MercuryStateLayout layout = Listing(state);
            Agree(state, layout);
            return layout;
        }

        protected virtual MercuryStateLayout Listing(byte[] state) => MercuryStateLayout.For(Rom, state);

        protected abstract void Agree(byte[] state, MercuryStateLayout layout);

        public virtual void Dispose()
        {
            if (_temporary) try { File.Delete(_rom); } catch (IOException) { }
        }

        private sealed class CsharpRig : MercuryRig
        {
            private readonly MercuryCore _core;

            public CsharpRig(string rom, bool temporary, GbModel model) : base(rom, temporary)
            {
                _core = new MercuryCore { Model = model };
                _core.LoadRom(RomPath);
            }

            public override MercuryEngine Engine => MercuryEngine.Mercury;
            public override ICore Core => _core;
            public override MercuryCore View => _core;
            public override void Tick(int cycles) => _core.Bus!.Tick(cycles);
            public override byte Read(int address) => _core.Bus!.Read((ushort)address);
            public override void Write(int address, byte value) => _core.Bus!.Write((ushort)address, value);
            public override byte ReadSpace(string space, int address) => _core.ReadSpace(space, address);
            public override void Stop() => _core.Bus!.Stop();
            public override IReadOnlyList<byte> SerialLog => _core.Bus!.SerialLog.ToArray();
            public override void SetChannelMuted(int channel, bool muted) => _core.Bus!.Apu.SetChannelMuted(channel, muted);
            public override bool IsChannelMuted(int channel) => _core.Bus!.Apu.IsChannelMuted(channel);
            public override BreakpointRegistry Breakpoints => _core.Breakpoints;
            public override IDebugTarget CreateDebugTarget() => new EmuSen.Cores.Nintendo.Mercury.Debug.MercuryDebugTarget(_core);
            public override void SaveSram() => _core.SaveSram();

            // The live core is the state it has just saved.
            protected override void Agree(byte[] state, MercuryStateLayout layout) => MercuryStateAgreement.Check(_core, state, layout);

            public override int Step()
            {
                MemoryBus bus = _core.Bus!;
                int cycles = _core.Cpu!.Step(bus.InterruptEnable, bus.InterruptFlags, out int serviced);
                if (serviced >= 0) bus.InterruptFlags &= (byte)~(1 << serviced);
                int stall = bus.TakePendingStall();
                if (stall > 0) bus.Tick(stall);
                return cycles + stall;
            }
        }

        // MercuryRT through its shim; the view is pulled from its state when read after the machine moved, and pushed back before the machine runs when it was touched.
        private sealed unsafe class RustRig : MercuryRig
        {
            private static readonly delegate* unmanaged<nint, int> StepOf = (delegate* unmanaged<nint, int>)MercuryNative.Engine.Export("mercuryrt_core_step");
            private static readonly delegate* unmanaged<nint, int, int> TickOf = (delegate* unmanaged<nint, int, int>)MercuryNative.Engine.Export("mercuryrt_core_tick");
            private static readonly delegate* unmanaged<nint, int> StopOf = (delegate* unmanaged<nint, int>)MercuryNative.Engine.Export("mercuryrt_core_stop");
            private static readonly delegate* unmanaged<nint, byte*, nuint, long> SerialOf = (delegate* unmanaged<nint, byte*, nuint, long>)MercuryNative.Engine.Export("mercuryrt_core_serial");

            private readonly MercuryRtCore _core;
            private readonly MercuryCore _view;
            private byte[] _pulled = Array.Empty<byte>();
            private bool _stale = true;
            private bool _touched;

            public RustRig(string rom, bool temporary, GbModel model) : base(rom, temporary)
            {
                Assert.True(MercuryRtCore.Available, MercuryNative.Engine.Report);
                Assert.True(StepOf != null && TickOf != null && StopOf != null && SerialOf != null, "libmercuryrt lacks the rig's extensions");
                _core = new MercuryRtCore { Model = model };
                _core.LoadRom(RomPath);
                _view = new MercuryCore { Model = model };
                _view.LoadRom(RomPath);
            }

            public override MercuryEngine Engine => MercuryEngine.MercuryRt;
            public override ICore Core
            {
                get
                {
                    Push();
                    _stale = true;
                    return _core;
                }
            }

            public override MercuryCore View
            {
                get
                {
                    if (_stale)
                    {
                        _pulled = _core.Machine.Save();
                        _view.LoadState(new MemoryStream(_pulled, writable: false));
                        _stale = false;
                        MercuryStateAgreement.Check(_view, _pulled, Listing(_pulled));
                    }
                    _touched = true;
                    return _view;
                }
            }

            // A view that was read may have been edited: its state goes back into the machine when it differs from what was pulled.
            private void Push()
            {
                if (!_touched) return;
                _touched = false;
                using var stream = new MemoryStream();
                _view.SaveState(stream);
                byte[] state = stream.ToArray();
                if (state.AsSpan().SequenceEqual(_pulled)) return;
                _core.Machine.Core.Load(state);
                _pulled = state;
            }

            // The listing for this state, written from MercuryRT's own state_layout when recording and none is committed.
            protected override MercuryStateLayout Listing(byte[] state)
            {
                try { return base.Listing(state); }
                catch (InvalidDataException) when (Environment.GetEnvironmentVariable(MercuryStateLayout.RecordVariable) == "1")
                {
                    MercuryStateLayout.Record(Rom[0x147], _core.Machine.Core.Layout());
                    return base.Listing(state);
                }
            }

            // A state read through the listing is read into the view as well, which is then current.
            protected override void Agree(byte[] state, MercuryStateLayout layout)
            {
                _view.LoadState(new MemoryStream(state, writable: false));
                _pulled = state;
                _stale = false;
                MercuryStateAgreement.Check(_view, state, layout);
            }

            private T Run<T>(Func<nint, T> call)
            {
                Push();
                _stale = true;
                lock (_core.Machine.Core.Gate) return call(_core.Machine.Core.Handle);
            }

            public override void Tick(int cycles) => Run(h => TickOf(h, cycles));
            public override void Stop() => Run(h => StopOf(h));
            public override byte Read(int address) => _core.ReadSpace(MercuryCore.SpaceCpuBus, Pushed(address));
            public override void Write(int address, byte value) => _core.WriteSpace(MercuryCore.SpaceCpuBus, Pushed(address), value);

            public override byte ReadSpace(string space, int address)
            {
                Push();
                return _core.ReadSpace(space, address);
            }

            private int Pushed(int address)
            {
                Push();
                _stale = true;
                return address & 0xFFFF;
            }

            public override int Step()
            {
                int cycles = Run(h => StepOf(h));
                if (cycles == -20) throw new NotSupportedException("MercuryRT stopped on an opcode no SM83 has.");
                return cycles;
            }

            public override IReadOnlyList<byte> SerialLog => Run(h =>
            {
                long n = SerialOf(h, null, 0);
                var log = new byte[n];
                fixed (byte* data = log) SerialOf(h, data, (nuint)log.Length);
                return log;
            });

            // As a frontend mutes a channel: on the shim's mirror, then sent down.
            public override void SetChannelMuted(int channel, bool muted)
            {
                _core.Mirror.Bus!.Apu.SetChannelMuted(channel, muted);
                _core.SyncMutes();
            }

            public override bool IsChannelMuted(int channel) => _core.Mirror.Bus!.Apu.IsChannelMuted(channel);
            public override BreakpointRegistry Breakpoints => _core.Breakpoints;

            public override void SaveSram() => _core.SaveSram();

            public override IDebugTarget CreateDebugTarget()
            {
                Push();
                _stale = true;
                return _core.CreateDebugTarget();
            }

            public override void Dispose()
            {
                _core.Dispose();
                base.Dispose();
            }
        }
    }
}
