using System;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.Cores.Nintendo.MarsRT;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.WiseMan.Fixtures
{
    // One N64 engine under the debugger's claims: the core, its target and registries, and the few pokes the claims make at the machine - see Mars_Native.md §6.5.
    public abstract class MarsDebugRig : IDisposable
    {
        public abstract ICore Core { get; }
        public abstract IDebugTarget Target { get; }
        public abstract BreakpointRegistry Breakpoints { get; }
        public abstract CoverageRegistry Coverage { get; }
        public abstract CoverageRegistry RspCoverage { get; }
        public abstract CallStackRegistry CallStack { get; }
        public abstract WatchRegistry Watches { get; }
        public abstract FrameLogRegistry FrameLog { get; }
        public abstract bool IsHaltedAtBreakpoint { get; }
        public abstract int HaltedAddress { get; }
        public abstract long TotalFrames { get; }
        public abstract long Instructions { get; }
        public abstract bool Listening { get; }

        // The engine's name for CoreFactory, and the core it builds.
        public abstract string? Engine { get; }
        public abstract Type CoreType { get; }

        public void RunFrame() => Core.RunFrame();

        // LoadRom on the same image, with the VI programmed again so a frame stays short.
        public abstract void Reload();

        public abstract uint ReadRdram32(uint address);
        public abstract byte ReadRdram8(uint address);
        public abstract void WriteRdram8(uint address, byte value);
        public abstract void WriteBus32(uint physical, uint value);
        public abstract uint ReadBus32(uint physical);
        public abstract void SetStatus(ulong value);
        public abstract void RaiseVideoInterrupt();
        public abstract void StartRsp(uint pc);
        public abstract void StepRsp(int steps);
        public abstract void WriteImem(int offset, byte value);
        public abstract PhysicalAddress? ResolvePhysical(int address);

        // The registries a bundle's core owns, for the claim that the bundle hands out the core's own.
        public abstract (BreakpointRegistry Breakpoints, CoverageRegistry Coverage, WatchRegistry Watches) RegistriesOf(ICore core);

        public abstract void Dispose();

        // MarsDebugTests' VI: a field of 0x20 half-lines at 0x40 cycles each, so a frame is a few thousand instructions.
        protected void ProgramVideo()
        {
            WriteBus32(MemoryMap.ViBase + VideoInterface.VerticalSync, 0x20);
            WriteBus32(MemoryMap.ViBase + VideoInterface.HorizontalSync, 0x40);
        }

        public static MarsDebugRig Mars(string rom) => new MarsRig(rom);

        public static MarsDebugRig MarsRt(string rom) => new MarsRtRig(rom);

        private sealed class MarsRig : MarsDebugRig
        {
            private readonly string _rom;
            private readonly MarsCore _core;
            private readonly MarsDebugTarget _target;

            public MarsRig(string rom)
            {
                _rom = rom;
                _core = new MarsCore(batteryRamDisabled: true) { SkipRendering = true };
                _core.LoadRom(rom);
                ProgramVideo();
                _target = new MarsDebugTarget(_core);
            }

            public override ICore Core => _core;
            public override IDebugTarget Target => _target;
            public override BreakpointRegistry Breakpoints => _core.Breakpoints;
            public override CoverageRegistry Coverage => _core.Coverage;
            public override CoverageRegistry RspCoverage => _core.RspCoverage;
            public override CallStackRegistry CallStack => _core.CallStack;
            public override WatchRegistry Watches => _core.Watches;
            public override FrameLogRegistry FrameLog => _core.FrameLog;
            public override bool IsHaltedAtBreakpoint => _core.IsHaltedAtBreakpoint;
            public override int HaltedAddress => _core.HaltedAddress;
            public override long TotalFrames => _core.TotalFrames;
            public override long Instructions => _core.Cpu!.Instructions;
            public override bool Listening => _target.Listening;
            public override string? Engine => null;
            public override Type CoreType => typeof(MarsCore);

            public override void Reload()
            {
                _core.LoadRom(_rom);
                ProgramVideo();
            }

            public override uint ReadRdram32(uint address) => _core.Bus!.Read32(address);
            public override byte ReadRdram8(uint address) => _core.Bus!.Rdram[address];
            public override void WriteRdram8(uint address, byte value) => _core.Bus!.Rdram[address] = value;
            public override void WriteBus32(uint physical, uint value) => _core.Bus!.Write32(physical, value);
            public override uint ReadBus32(uint physical) => _core.Bus!.Read32(physical);
            public override void SetStatus(ulong value) => _core.Cpu!.Cop0[EmuSen.Cores.Nintendo.Mars.Cpu.Core.Cpu.StatusRegister] = value;

            public override void RaiseVideoInterrupt()
            {
                _core.Bus!.Mi.Mask = MiInterrupt.VideoInterface;
                _core.Bus.Mi.Raise(MiInterrupt.VideoInterface);
            }

            public override void StartRsp(uint pc) => _core.Bus!.Sp.Processor.Start(pc);

            public override void StepRsp(int steps)
            {
                for (int i = 0; i < steps; i++) _core.Bus!.Sp.Processor.Step();
            }

            public override void WriteImem(int offset, byte value) => _core.Bus!.SpImem[offset] = value;
            public override PhysicalAddress? ResolvePhysical(int address) => _target.ResolvePhysical(address);
            public override (BreakpointRegistry, CoverageRegistry, WatchRegistry) RegistriesOf(ICore core) => (((MarsCore)core).Breakpoints, ((MarsCore)core).Coverage, ((MarsCore)core).Watches);
            public override void Dispose() { }
        }

        private sealed class MarsRtRig : MarsDebugRig
        {
            private readonly string _rom;
            private readonly MarsRtCore _core;
            private readonly MarsRtDebugTarget _target;

            public MarsRtRig(string rom)
            {
                _rom = rom;
                _core = new MarsRtCore(batteryRamDisabled: true) { SkipRendering = true };
                _core.LoadRom(rom);
                ProgramVideo();
                _target = new MarsRtDebugTarget(_core);
            }

            public override ICore Core => _core;
            public override IDebugTarget Target => _target;
            public override BreakpointRegistry Breakpoints => _core.Breakpoints;
            public override CoverageRegistry Coverage => _core.Coverage;
            public override CoverageRegistry RspCoverage => _core.RspCoverage;
            public override CallStackRegistry CallStack => _core.CallStack;
            public override WatchRegistry Watches => _core.Watches;
            public override FrameLogRegistry FrameLog => _core.FrameLog;
            public override bool IsHaltedAtBreakpoint => _core.IsHaltedAtBreakpoint;
            public override int HaltedAddress => _core.HaltedAddress;
            public override long TotalFrames => _core.TotalFrames;
            public override long Instructions => _core.Instructions;
            public override bool Listening => _target.Listening;
            public override string? Engine => CoreCatalog.MarsRtEngine;
            public override Type CoreType => typeof(MarsRtCore);

            public override void Reload()
            {
                _core.LoadRom(_rom);
                ProgramVideo();
            }

            public override uint ReadRdram32(uint address) => _core.ReadBus32(address);
            public override byte ReadRdram8(uint address) => _core.Peek(MarsRtSpace.Rdram, address);
            public override void WriteRdram8(uint address, byte value) => _core.Poke(MarsRtSpace.Rdram, address, value);
            public override void WriteBus32(uint physical, uint value) => _core.WriteBus32(physical, value);
            public override uint ReadBus32(uint physical) => _core.ReadBus32(physical);
            public override void SetStatus(ulong value) => _core.SetCop0(12, value);
            public override void RaiseVideoInterrupt() => _core.RaiseInterrupt(MiInterrupt.VideoInterface);
            public override void StartRsp(uint pc) => _core.StepRsp(pc, 0);
            public override void StepRsp(int steps) => _core.StepRsp(null, steps);
            public override void WriteImem(int offset, byte value) => _core.Poke(MarsRtSpace.Imem, (uint)offset, value);
            public override PhysicalAddress? ResolvePhysical(int address) => _target.ResolvePhysical(address);
            public override (BreakpointRegistry, CoverageRegistry, WatchRegistry) RegistriesOf(ICore core) => (((MarsRtCore)core).Breakpoints, ((MarsRtCore)core).Coverage, ((MarsRtCore)core).Watches);
            public override void Dispose() => _core.Dispose();
        }
    }
}
