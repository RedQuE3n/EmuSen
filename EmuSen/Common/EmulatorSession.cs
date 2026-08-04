using System;
using System.Collections.Generic;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Common
{
    // Thin, mostly core-agnostic wrapper around ICore for a non-Raylib
    // frontend (the Avalonia EmuSen.Mistress project) to drive a core on
    // its own schedule, independent of any particular windowing/UI
    // toolkit - call LoadRom() once and RunFrame() whenever the UI's own
    // render loop wants a new frame.
    //
    // This used to hold a full second copy of VenusCore's timing loop
    // (CPU/PPU/APU stepping, HDMA, NMI - see git history if curious just
    // how identical the two were). That's gone now; RunFrame()/SaveState()/
    // LoadState()/GetFrameBufferRgba() all delegate straight to an ICore.
    //
    // Core-agnostic now - LoadRom picks a core from the ROM's extension - see EmuSen_Multicore.md §2.
    public class EmulatorSession
    {
        private ICore? _core;

        public int ScreenWidth => _core?.ScreenWidth ?? 256;

        // Asks the core, or Moon's 240 lines get submitted as Venus's 224 - see EmuSen_Multicore.md §7.
        public int ScreenHeight => _core?.ScreenHeight ?? 224;

        // "SNES" before LoadRom() is called too - there's only one core to
        // report today, but this exists so a caller (e.g. a log directory
        // path) doesn't need its own core-specific fallback string.
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

        // Firmware <romPath> needs that isn't in the library yet. Answered
        // without loading, so an interactive frontend can offer to go and
        // find it BEFORE LoadRom - afterwards is too late, the core has
        // already come up with the chip absent. A frontend that can't ask
        // (Pharaoh, Tomoe) simply skips this and gets the missing-chip
        // behaviour. See EmuSen_Firmware.md §3.
        //
        // Constructs a throwaway core for the same reason LoadRom below
        // hardcodes one: there is exactly one core to construct today.
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

        // Periodic SRAM autosave is handled inside VenusCore.RunFrame()
        // itself now, not scheduled here - this class used to own that
        // timing separately before delegating to ICore.
        public void RunFrame()
        {
            if (_core is null) throw new InvalidOperationException("RunFrame() called before LoadRom().");
            _core.RunFrame();
        }

        public void SaveSram() => _core?.SaveSram();

        // Must be called before the caller's log writer is disposed, while
        // CpuVerboseLogging/Spc700VerboseLogging might still be on - see
        // DebugTools.RepeatCollapsingTrace<TKey>.Flush() for why a
        // still-in-progress loop/tail would otherwise never reach the log.
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

        // AudioSampleRate falls back to the same AudioSettings default
        // CoreName above falls back to "SNES" for - a caller opening its
        // output device before any ROM is loaded yet (or between loads)
        // still needs a sample rate to open it at.
        public int AudioSampleRate => _core?.AudioSampleRate ?? EmuSen.Audio.AudioSettings.SampleRate;

        // Straight pass-through to ICore.DequeueAudioSamples() - see that
        // method's own comment. Returns an empty array rather than
        // throwing when no ROM is loaded yet, unlike GetFrameBufferRgba()
        // above - an audio pump callable every tick regardless of session
        // state (the same "no ROM loaded is a normal condition, not an
        // error" convention Shell/Commands/DebugCommandHelpers.
        // RequireTarget's callers avoid on their own read paths) is
        // simpler for a caller than needing its own IsRomLoaded guard
        // around every single pump call.
        public short[] DequeueAudioSamples(int maxFrames) => _core?.DequeueAudioSamples(maxFrames) ?? Array.Empty<short>();

        // Falls back to NTSC until a ROM is loaded - see Venus_CPU.md §8.5b.
        public double FrameRateHz => _core?.FrameRateHz ?? 60.0988;
    }
}
