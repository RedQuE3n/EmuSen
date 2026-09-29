using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT behind the NES's ICore: the machine in Rust, the registries, saves and cheats' rules in C# - see Moon_Native.md §2.
    public sealed class MoonRtCore : NativeRtCore<MoonMachine>, ICore, IFrameProfiler, ICheatRegistryHost, IStateFormat, IFrameBufferPool, IDisposable
    {
        private readonly uint[] _buttons = new uint[2];
        private double _lastFrameMs;

        public MoonRtCore() : base(MoonMachine.FrameBytes) { }

        // The C# machine the debugger reads, refreshed from MoonRT's state; its registries are this core's - see Moon_Native.md §2.4.
        public MoonCore Mirror { get; } = new();

        protected override ICore MirrorCore => Mirror;
        protected override bool MirrorLoaded => Mirror.Bus is not null;
        protected override CheatRegistry MirrorCheats { get => Mirror.Cheats; set => Mirror.Cheats = value; }
        protected override IReadOnlyList<string> SpaceNames => MoonMachine.SpaceNames;
        protected override int PatchLow => 0x4020;
        protected override int PatchHigh => 0xFFFF;

        public static bool Available => MoonMachine.Complete;

        public override string CoreName => "NES";
        public override int ScreenWidth => Moon.Video.Ppu.ScreenWidth;
        public override int ScreenHeight => Moon.Video.Ppu.ScreenHeight;
        public override double FrameRateHz => Mirror.FrameRateHz;
        public override IReadOnlyList<PadButton> SupportedButtons => MoonCore.PadButtons;
        public override int StateVersion => MoonCore.StateVersion;

        // One phase: the frame is one call into Rust, which has no seam to time its parts through - see Moon_Native.md §2.1.
        public IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases => new[] { ("frame", _lastFrameMs) };

        public WatchRegistry Watches => Mirror.Watches;
        public override FrameLogRegistry FrameLog => Mirror.FrameLog;
        public override BreakpointRegistry Breakpoints => Mirror.Breakpoints;
        public CoverageRegistry Coverage => Mirror.Coverage;
        public LabelRegistry Labels => Mirror.Labels;

        public MoonMachine Machine => _machine ?? throw new InvalidOperationException("No ROM is loaded.");

        // MoonCore.LoadRom: the header parsed by the C# Cartridge so its exceptions are C#'s own, the battery save read as C# reads it.
        public override void LoadRom(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            Cartridge header = Cartridge.FromImage(image);
            BatterySave battery = BatterySave.Open(path, BatterySave.Nes, hasRam: header.HasBattery && header.PrgRam.Length > 0);
            byte[]? saved = battery.Read();

            var machine = new MoonMachine(image);
            Adopt(machine, battery, saved, header.PrgRam.Length);
            machine.SetButtons(0, _buttons[0]);
            machine.SetButtons(1, _buttons[1]);
            Mirror.LoadRom(path);
        }

        // The RESET button - see Moon_Core.md §6.
        public void Reset()
        {
            if (_machine is null) throw new InvalidOperationException("Reset() called before LoadRom().");
            _machine.Reset();
        }

        public override void SetButton(int port, PadButton button, bool pressed)
        {
            int bit = button switch
            {
                PadButton.A => 0,
                PadButton.B => 1,
                PadButton.Select => 2,
                PadButton.Start => 3,
                PadButton.Up => 4,
                PadButton.Down => 5,
                PadButton.Left => 6,
                PadButton.Right => 7,
                _ => -1,
            };
            if (bit < 0) return;
            int pad = port == 0 ? 0 : 1;
            _buttons[pad] = pressed ? _buttons[pad] | (1u << bit) : _buttons[pad] & ~(1u << bit);
            _machine?.SetButtons(pad, _buttons[pad]);
        }

        // MoonCore.RunFrame and EndFrame's host half, in C#'s order; breakpoints and coverage are stage 5's (Moon_Native.md §4).
        public override void RunFrame()
        {
            if (_machine is null) throw new InvalidOperationException("RunFrame() called before LoadRom().");
            BeforeFrame();
            long start = Stopwatch.GetTimestamp();
            _machine.RunFrame();
            _lastFrameMs = Stopwatch.GetElapsedTime(start).TotalMilliseconds;
            EndFrame();
        }

        // MoonCore.LoadState's two refusals with its messages.
        protected override void CheckState(byte[] state)
        {
            if (state.Length < 8) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            if (BitConverter.ToUInt32(state, 0) != MoonCore.StateMagic) throw new InvalidDataException("Not a Moon save state.");
            int version = BitConverter.ToInt32(state, 4);
            if (version != MoonCore.StateVersion) throw new InvalidDataException($"Save state version {version} is not {MoonCore.StateVersion}.");
        }

        // The debugger's view: the mirror loaded from MoonRT's state, its reads and writes sent to MoonRT - see Moon_Native.md §2.4.
        public MoonDebugTarget CreateDebugTarget() => new(Mirror, () => (_lastFrameMs, 0.0), new MoonDebugHost(
            ReadSpace, WriteSpace, SyncMirror, ApplyCheats, () => TotalFrames, (_, _) => SyncMutes()));

        protected override uint MuteMask()
        {
            uint mask = 0;
            for (int i = 0; i < 5; i++) if (Mirror.Apu!.IsChannelMuted(i)) mask |= 1u << i;
            return mask;
        }
    }
}
