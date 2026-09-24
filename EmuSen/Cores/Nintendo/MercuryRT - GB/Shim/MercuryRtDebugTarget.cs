using System.Collections.Generic;
using EmuSen.Cauldron;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Debug;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // The debugger's view of MercuryRT: the C# target over the mirror for what it shows, the core's registries and its live program counter for what halts - see Mercury_Native.md §8.5.
    public sealed class MercuryRtDebugTarget : IDebugTarget
    {
        private readonly MercuryRtCore _core;
        private readonly MercuryDebugTarget _view;
        private readonly DebugCpu[] _cpus;

        public MercuryRtDebugTarget(MercuryRtCore core)
        {
            _core = core;
            _view = new MercuryDebugTarget(core.Mirror, new MercuryDebugHost(
                core.ReadSpace, core.WriteSpace, core.SyncMirror, core.ApplyCheats, () => core.TotalFrames, (_, _) => core.SyncMutes()));
            _cpus = new[]
            {
                new DebugCpu("cpu", "SM83", core.Breakpoints)
                {
                    Coverage = core.Coverage,
                    CallStack = core.CallStack,
                    CodeSpace = MercuryCore.SpaceCpuBus,
                    Registers = _view.CpuRegisters,
                    ProgramCounter = () => core.Pc,
                },
            };
            core.Listen();
        }

        public string CoreName => _core.CoreName;
        public long FrameCount => _core.TotalFrames;
        public int MaxSprites => _view.MaxSprites;

        public WatchRegistry Watches => _core.Watches;
        public FrameLogRegistry FrameLog => _core.FrameLog;
        public BreakpointRegistry Breakpoints => _core.Breakpoints;
        public CheatRegistry Cheats => _core.Cheats;
        public CoverageRegistry? Coverage => _core.Coverage;
        public CallStackRegistry? CallStack => _core.CallStack;
        public LabelRegistry? Labels => _core.Labels;
        public IReadOnlyList<DebugCpu> DebugCpus => _cpus;

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters => _view.CpuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => _view.VideoRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => _view.ApuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => _view.CoprocessorRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => _view.Sprites;
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => _view.Palettes;
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => _view.AudioChannels;
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => _view.HardwareLoad;

        public IReadOnlyList<InterruptVector> InterruptVectors => _view.InterruptVectors;
        public IReadOnlyList<DebugDmaChannel> DmaChannels => _view.DmaChannels;
        public string? NameDmaDestination(byte register) => _view.NameDmaDestination(register);
        public PhysicalAddress? ResolvePhysical(int cpuAddress) => _view.ResolvePhysical(cpuAddress);
        public int TilemapEntryStride => _view.TilemapEntryStride;

        public void RefreshProviders() => _view.RefreshProviders();
        public void ApplyCheats() => _core.ApplyCheats();
        public void SetChannelMuted(int index, bool muted) => _view.SetChannelMuted(index, muted);
        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces() => _view.GetMemorySpaces();
        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count) => _view.Disassemble(spaceName, address, count);
        public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr) => _view.ClassifyStaticReference(instr);
        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) => _view.DecodeTilemapEntry(space, address);
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) => _view.DecodeTilePixels(space, address, bpp);
        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => _view.RenderTileSheet();
        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => _view.RenderPaletteSwatch();
        public (short[] Samples, int SampleRate) GetAudioSamples() => _view.GetAudioSamples();
        public string GetSummaryText() => _view.GetSummaryText().Replace("(Mercury)", "(MercuryRT)");
    }
}
