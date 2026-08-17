using System;
using System.Collections.Generic;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
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

        public IReadOnlyList<PadButton> SupportedButtons =>
            _core?.SupportedButtons ?? Array.Empty<PadButton>();

        // Built by CoreFactory alongside the core, so this class names no concrete target.
        public IDebugTarget? DebugTarget { get; private set; }

        public ICheatCodeCodec? CheatAutoDetectCodec { get; private set; }
        public ICheatCodeCodec? CheatExplicitCodec { get; private set; }
        public ICpuTraceSwitch? CpuTraceSwitch { get; private set; }

        // Firmware <romPath> needs that isn't in the library yet - see EmuSen_Firmware.md §3.
        public static IReadOnlyList<FirmwareRequest> MissingFirmwareFor(string romPath) =>
            FirmwareLibrary.MissingFrom(CoreFactory.ForFirmwareProbe(romPath).GetFirmwareRequirements(romPath));

        // Set by the frontend before LoadRom so the debug target shares its registry.
        public CheatRegistry? Cheats { get; set; }

        public void LoadRom(string path)
        {
            var bundle = CoreFactory.Load(path, headless: true, Cheats);
            _core = bundle.Core;
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

        // Before the caller's log writer is disposed, or a still-open trace loop never reaches the log.
        public void FlushVerboseLogs()
        {
            (_core as ITraceFlushable)?.FlushVerboseTrace();
        }

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

        public byte[] GetFrameBufferRgba()
        {
            if (_core is null) throw new InvalidOperationException("GetFrameBufferRgba() called before LoadRom().");
            return _core.GetFrameBufferRgba();
        }

        // AudioSampleRate falls back to the same AudioSettings default CoreName above falls back to "SNES".
        public int AudioSampleRate => _core?.AudioSampleRate ?? EmuSen.Audio.AudioSettings.SampleRate;

        // Straight pass-through to ICore.DequeueAudioSamples() - see that method's own comment.
        public short[] DequeueAudioSamples(int maxFrames) => _core?.DequeueAudioSamples(maxFrames) ?? Array.Empty<short>();

        // Falls back to NTSC until a ROM is loaded - see Venus_CPU.md §8.5b.
        public double FrameRateHz => _core?.FrameRateHz ?? 60.0988;
    }
}
