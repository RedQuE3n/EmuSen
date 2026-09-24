using System;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Debug;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.WiseMan.Fixtures
{
    // One Game Boy engine under the debugger's claims: the core, its target and registries, and the pokes the claims make - see Mercury_Native.md §8.5.
    public abstract class MercuryDebugRig : IDisposable
    {
        protected MercuryDebugRig(string rom) => Rom = rom;

        public string Rom { get; }
        public abstract ICore Core { get; }
        public abstract IDebugTarget Target { get; }
        public abstract Type TargetType { get; }
        public abstract Type CoreType { get; }

        // The engine's name for CoreFactory; null is the C# core.
        public abstract string? Engine { get; }

        public abstract int Pc { get; }
        public abstract byte ReadSpace(string space, int address);
        public abstract void WriteSpace(string space, int address, byte value);

        public BreakpointRegistry Breakpoints => Target.Breakpoints;
        public CoverageRegistry Coverage => Target.Coverage!;
        public CallStackRegistry? CallStack => Target.CallStack;
        public WatchRegistry Watches => Target.Watches;
        public CheatRegistry Cheats => Target.Cheats;
        public bool IsHaltedAtBreakpoint => Core.IsHaltedAtBreakpoint;
        public int HaltedAddress => Core.HaltedAddress;
        public long TotalFrames => Core.TotalFrames;

        public void RunFrame() => Core.RunFrame();

        public void Reload() => Core.LoadRom(Rom);

        public byte[] State()
        {
            using var stream = new MemoryStream();
            Core.SaveState(stream);
            return stream.ToArray();
        }

        public abstract void Dispose();

        public static MercuryDebugRig Mercury(string rom) => new MercuryRig(rom);

        public static MercuryDebugRig MercuryRt(string rom) => new MercuryRtRig(rom);

        private sealed class MercuryRig : MercuryDebugRig
        {
            private readonly MercuryCore _core = new();
            private readonly MercuryDebugTarget _target;

            public MercuryRig(string rom) : base(rom)
            {
                _core.LoadRom(rom);
                _target = new MercuryDebugTarget(_core);
            }

            public override ICore Core => _core;
            public override IDebugTarget Target => _target;
            public override Type TargetType => typeof(MercuryDebugTarget);
            public override Type CoreType => typeof(MercuryCore);
            public override string? Engine => null;
            public override int Pc => _core.Cpu!.PC;
            public override byte ReadSpace(string space, int address) => _core.ReadSpace(space, address);
            public override void WriteSpace(string space, int address, byte value) => _core.WriteSpace(space, address, value);
            public override void Dispose() { }
        }

        private sealed class MercuryRtRig : MercuryDebugRig
        {
            private readonly MercuryRtCore _core = new();
            private readonly MercuryRtDebugTarget _target;

            public MercuryRtRig(string rom) : base(rom)
            {
                _core.LoadRom(rom);
                _target = _core.CreateDebugTarget();
            }

            public override ICore Core => _core;
            public override IDebugTarget Target => _target;
            public override Type TargetType => _target.GetType();
            public override Type CoreType => typeof(MercuryRtCore);
            public override string? Engine => CoreCatalog.MercuryRtEngine;
            public override int Pc => _core.Pc;
            public override byte ReadSpace(string space, int address) => _core.ReadSpace(space, address);
            public override void WriteSpace(string space, int address, byte value) => _core.WriteSpace(space, address, value);
            public override void Dispose() => _core.Dispose();
        }
    }
}
