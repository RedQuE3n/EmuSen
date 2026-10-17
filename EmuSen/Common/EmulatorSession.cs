using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia.Input;

namespace EmuSen.Common
{
    // Thin, mostly core-agnostic wrapper around ICore for a non-Raylib frontend (the Avalonia - see EmuSen_Multicore.md §2.
    public class EmulatorSession
    {
        private ICore? _core;

        public int ScreenWidth => _core?.ScreenWidth ?? 256;

        // Asks the core, or Moon's 240 lines get submitted as Venus's 224 - see EmuSen_Multicore.md §7.
        public int ScreenHeight => _core?.ScreenHeight ?? 224;

        // Only meaningful after LoadRom; ask CoreCatalog.ConsoleForRom before that - see EmuSen_Multicore.md §12.
        public string CoreName => _core?.CoreName ?? "SNES";

        public long TotalFrames => _core?.TotalFrames ?? 0;
        public bool IsRomLoaded => _core?.IsRomLoaded ?? false;

        // Routed to whichever core is loaded - see EmuSen_Input.md §2.
        public void SetButton(int port, PadButton button, bool pressed) => _core?.SetButton(port, button, pressed);

        public void SetAxis(int port, PadAxis axis, double value) => _core?.SetAxis(port, axis, value);

        public void SetControllerConnected(int port, bool connected) => _core?.SetControllerConnected(port, connected);

        // 0 with nothing loaded - see EmuSen_Input.md §8.1.
        public int ControllerPorts => ControllerPortsOf(_core);

        private static int ControllerPortsOf(ICore? core) => core is { IsRomLoaded: true } ? EmuSen.Cores.ControllerPorts.Of(core) : 0;

        public IReadOnlyList<PadAxis> SupportedAxes => _core?.SupportedAxes ?? Array.Empty<PadAxis>();

        public IReadOnlyList<PadButton> SupportedButtons =>
            _core?.SupportedButtons ?? Array.Empty<PadButton>();

        // Built by CoreFactory alongside the core, so this class names no concrete target.
        public IDebugTarget? DebugTarget { get; private set; }

        public ICheatCodeCodec? CheatAutoDetectCodec { get; private set; }
        public ICheatCodeCodec? CheatExplicitCodec { get; private set; }
        public ICpuTraceSwitch? CpuTraceSwitch { get; private set; }

        // Required firmware <romPath> needs on <engine>, a CoreCatalog.EngineFor name or null for the reference, not in the library yet - see EmuSen_Firmware.md §3.
        public static IReadOnlyList<FirmwareRequest> MissingFirmwareFor(string romPath, string? engine = null)
        {
            // A file that opens nowhere asks for nothing; the load after it says why - see Diamond_Players.md §12.
            try { return FirmwareLibrary.MissingFrom(CoreFactory.ForFirmwareProbe(romPath, engine) is var probe && probe is Cores.Native.CoreEngine v1 ? v1.RequiredFirmwareFor(romPath) : probe.GetFirmwareRequirements(romPath).Where(r => r.Required)); }
            catch (Exception e) when (e is NotSupportedException or System.IO.IOException or System.IO.InvalidDataException or UnauthorizedAccessException) { return Array.Empty<FirmwareRequest>(); }
        }

        // Set by the frontend before LoadRom so the debug target shares its registry.
        public CheatRegistry? Cheats { get; set; }

        // The engine the frontend's setting asks for, a CoreCatalog.EngineFor name; null is the console's default - see EmuSen_Settings_Reference.md §4.44.
        public string? Engine { get; set; }

        // Set by the frontend before LoadRom: the device a core may draw its picture on - see EmuSen_DevicePicture.md §17.
        public Func<IDeviceLend?>? DeviceLender { get; set; }

        // Set by the frontend before LoadRom: what each core this session makes is made with.
        public CoreOptions Options { get; set; } = CoreOptions.Default;

        // Why the engine asked for is not the one running, or null - see CoreFactory.EngineNotice.
        public string? EngineNotice { get; private set; }

        // The core's word that this game runs on a firmware replacement or without a chip - see ICore.FirmwareNotice.
        public string? FirmwareNotice => _core?.FirmwareNotice;

        public void LoadRom(string path) => LoadRom(path, loaded: null);

        // <loaded> is CoreFactory.Loaded's core of <path> on <Engine>, made off this thread; null loads it here.
        public void LoadRom(string path, ICore? loaded)
        {
            var bundle = loaded is null ? CoreFactory.Load(path, headless: true, Cheats, engine: Engine, lender: DeviceLender, options: Options)
                : CoreFactory.Bundle(loaded, Cheats) with { Notice = CoreFactory.EngineNotice(path, Engine, loaded) };
            _core = bundle.Core;
            EngineNotice = bundle.Notice;
            DebugTarget = bundle.DebugTarget;
            CheatAutoDetectCodec = bundle.CheatAutoDetectCodec;
            CheatExplicitCodec = bundle.CheatExplicitCodec;
            CpuTraceSwitch = bundle.CpuTraceSwitch;
        }

        // The console started with nothing in it, on Engine where it can - see Diamond_Shell.md §1.5.
        public void StartConsole(string console)
        {
            var bundle = CoreFactory.LoadConsole(console, Cheats, Engine, DeviceLender, Options);
            _core = bundle.Core;
            EngineNotice = null;
            DebugTarget = bundle.DebugTarget;
            CheatAutoDetectCodec = bundle.CheatAutoDetectCodec;
            CheatExplicitCodec = bundle.CheatExplicitCodec;
            CpuTraceSwitch = bundle.CpuTraceSwitch;
        }

        // The periodic SRAM autosave lives inside RunFrame now, not scheduled here.
        public void RunFrame()
        {
            if (_core is null) throw new InvalidOperationException("RunFrame() called before LoadRom().");
            _core.RunFrame();
        }

        public void SaveSram() => _core?.SaveSram();

        // Lets go of the game's save media, as the frontend leaves the game, so its cards can be managed - see EmuSen_SaveMedia.md §3.5.
        public void ReleaseMedia() => (_core as Cores.Native.CoreEngine)?.ReleaseMedia();

        // Why a slot of the game runs on the game's own file, or null - see CoreEngine.MediaNotice.
        public string? MediaNotice => (_core as Cores.Native.CoreEngine)?.MediaNotice;

        public void SaveState(string path)
        {
            if (_core is null) throw new InvalidOperationException("SaveState() called before LoadRom().");
            _core.SaveState(path);
        }

        public void LoadState(string path)
        {
            if (_core is null) throw new InvalidOperationException("LoadState() called before LoadRom().");
            _core.LoadState(path);
        }

        // For a caller handing the core straight to RewindBuffer; null before LoadRom(), like DebugTarget.
        public EmuSen.Cores.ICore? Core => _core;

        // Fast-forward frame skipping - see ICore.SkipRendering.
        public bool SkipRendering
        {
            get => _core?.SkipRendering ?? false;
            set { if (_core is not null) _core.SkipRendering = value; }
        }

        // Null for a core that cannot tell, which a frontend must read as a new picture every frame - see EmuSen_Multicore.md §14.
        public long? FrameSerial => (_core as EmuSen.Cores.IFrameSerial)?.FrameSerial;

        // How many times the frontend shows each row of the frame; one for a core that repeats its own - see EmuSen_Multicore.md §15.
        public int RowRepeat => (_core as EmuSen.Cores.IRepeatedRows)?.RowRepeat ?? 1;

        // The screen's shape the core reports for the frame on show; 0 when it reports none - see EmuSen_Serenity.md §2.9.
        public double DisplayAspect => _core?.DisplayAspect ?? 0;

        public int DisplayLines => _core?.DisplayLines ?? 0;

        public CoreRawPicture? RawPicture => _core?.RawPicture;

        // What the frame control's filter reads, passed to the core before each frame - see EmuSen_CoreAPI.md §6.30.
        public string? RawEncoding
        {
            get => _core?.RawEncoding;
            set { if (_core is { } core) core.RawEncoding = value; }
        }

        public byte[] GetFrameBufferRgba()
        {
            if (_core is null) throw new InvalidOperationException("GetFrameBufferRgba() called before LoadRom().");
            return _core.GetFrameBufferRgba();
        }

        // With no core there are no samples, and the rate is audio.json's default.
        public int AudioSampleRate => _core?.AudioSampleRate ?? Galaxia.Models.AudioConfig.DefaultSampleRate;

        // Straight pass-through to ICore.DequeueAudioSamples() - see that method's own comment.
        public short[] DequeueAudioSamples(int maxFrames) => _core?.DequeueAudioSamples(maxFrames) ?? Array.Empty<short>();

        // Falls back to NTSC until a ROM is loaded - see Venus_CPU.md §8.5b.
        public double FrameRateHz => _core?.FrameRateHz ?? 60.0988;
    }
}
