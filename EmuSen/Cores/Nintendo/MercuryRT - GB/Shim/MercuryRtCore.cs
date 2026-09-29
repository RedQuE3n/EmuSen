using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Debug;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT behind the Game Boy's ICore: the machine in Rust, the registries, saves and cheats' rules in C# - see Mercury_Native.md §8.3.
    public sealed partial class MercuryRtCore : NativeRtCore<MercuryMachine>, ICore, ICheatRegistryHost, IStateFormat, IFrameBufferPool, ICoreSettings, IDisposable
    {
        private static readonly PadButton[] MaskOrder = { PadButton.Right, PadButton.Left, PadButton.Up, PadButton.Down, PadButton.A, PadButton.B, PadButton.Select, PadButton.Start };

        private uint _buttons;

        // The C# machine the debugger reads, refreshed from MercuryRT's state; its registries are this core's - see Mercury_Native.md §8.3.
        public MercuryCore Mirror { get; } = new();

        protected override ICore MirrorCore => Mirror;
        protected override bool MirrorLoaded => Mirror.Bus is not null;
        protected override CheatRegistry MirrorCheats { get => Mirror.Cheats; set => Mirror.Cheats = value; }
        protected override IReadOnlyList<string> SpaceNames => MercuryMachine.SpaceNames;
        protected override int PatchLow => 0;
        protected override int PatchHigh => 0x7FFF;

        public static bool Available => MercuryMachine.Complete;

        // The console running, as MercuryCore names it; a state from the other console changes it - see Mercury_Model.md §5.
        public override string CoreName => _machine?.CgbHardware == true ? "GBC" : "GB";
        public override int ScreenWidth => MercuryCore.ScreenWidthPixels;
        public override int ScreenHeight => MercuryCore.ScreenHeightPixels;
        public override double FrameRateHz => MercuryCore.CpuClockHz / (double)MercuryCore.CyclesPerFrame;
        public override IReadOnlyList<PadButton> SupportedButtons => MercuryCore.PadButtons;
        public override int StateVersion => MercuryCore.StateVersion;

        private GbModel _model;
        private string? _romPath;

        // MercuryCore.Model: the console the next load builds, and a change before the first frame loads the game again at once.
        public GbModel Model
        {
            get => _model;
            set
            {
                if (value == _model) return;
                _model = value;
                if (_machine is not null && TotalFrames == 0 && _romPath is { Length: > 0 } path) LoadRom(path);
            }
        }

        public IReadOnlyList<CoreSetting> Settings => MercuryCore.ModelSettings;

        public string Get(string key) => key == MercuryCore.ModelKey ? MercuryCore.ModelName(Model) : throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));

        public void Set(string key, string value)
        {
            if (key != MercuryCore.ModelKey) throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));
            Model = MercuryCore.ParseModel(value);
        }

        public WatchRegistry Watches => Mirror.Watches;
        public override FrameLogRegistry FrameLog => Mirror.FrameLog;
        public override BreakpointRegistry Breakpoints => Mirror.Breakpoints;
        public CoverageRegistry Coverage => Mirror.Coverage;
        public LabelRegistry Labels => Mirror.Labels;
        public CallStackRegistry CallStack => Mirror.CallStack;

        public MercuryRtCore() : base(MercuryMachine.FrameBytes) => CallStack.FrameNumberProvider = () => _eventFrame;

        public MercuryMachine Machine => _machine ?? throw new InvalidOperationException("No ROM is loaded.");

        // MercuryCore.LoadRom: the header parsed by the C# Cartridge so its exceptions are C#'s own, the battery save read as C# reads it.
        public override void LoadRom(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            Cartridge header = Cartridge.FromImage(image);
            BatterySave battery = BatterySave.Open(path, BatterySave.GameBoyFolder(path), hasRam: header.HasBattery && header.Ram.Length > 0);
            byte[]? saved = battery.Read();

            var machine = new MercuryMachine(image, _model);
            Adopt(machine, battery, saved, header.Ram.Length);
            _romPath = path;
            machine.SetButtons(_buttons);
            Mirror.LoadRom(path);
            Mirror.Model = _model;
            IsHaltedAtBreakpoint = false;
        }

        public override void SetButton(int port, PadButton button, bool pressed)
        {
            if (port != 0) return;
            int bit = Array.IndexOf(MaskOrder, button);
            if (bit < 0) return;
            _buttons = pressed ? _buttons | (1u << bit) : _buttons & ~(1u << bit);
            _machine?.SetButtons(_buttons);
        }

        // MercuryCore.RunFrame and EndFrame; the observed loop when anything is armed, and a halt returns before the frame's end - see Mercury_Native.md §8.5.
        public override void RunFrame()
        {
            if (_machine is null) throw new InvalidOperationException("RunFrame() called before LoadRom().");
            BeforeFrame();
            bool resuming = IsHaltedAtBreakpoint;
            IsHaltedAtBreakpoint = false;
            if (Observed)
            {
                if (!RunObserved(_machine.Handle, resuming)) return;
            }
            else
            {
                _machine.RunFrame();
            }
            EndFrame();
        }

        // MercuryCore.LoadState's two refusals with its messages.
        protected override void CheckState(byte[] state)
        {
            if (state.Length < 4) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            if (BitConverter.ToUInt32(state, 0) != MercuryCore.StateMagic) throw new InvalidDataException("Not a Mercury save state.");
            if (state.Length < 8) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            int version = BitConverter.ToInt32(state, 4);
            if (version is < MercuryCore.OldestReadableVersion or > MercuryCore.StateVersion) throw new InvalidDataException($"Save state version {version} is not one this build reads ({MercuryCore.OldestReadableVersion} to {MercuryCore.StateVersion}).");
        }

        // A store to the bus while a debugger listens goes through the observed path, as C#'s bus reports it.
        public override void WriteSpace(string spaceName, int address, byte value)
        {
            if (_machine is not null && SpaceNumber(spaceName) == CpuBusSpace && Listening)
            {
                WriteObserved(_machine.Handle, address, value);
                return;
            }
            base.WriteSpace(spaceName, address, value);
        }

        // The debugger's view: the mirror loaded from MercuryRT's state, its reads and writes sent to MercuryRT, its halts this core's - see Mercury_Native.md §8.5.
        public MercuryRtDebugTarget CreateDebugTarget() => new(this);

        protected override uint MuteMask()
        {
            uint mask = 0;
            for (int i = 0; i < 4; i++) if (Mirror.Bus!.Apu.IsChannelMuted(i)) mask |= 1u << i;
            return mask;
        }
    }
}
