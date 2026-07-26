using System;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Memory;

namespace EmuSen.Common
{
    // Thin, mostly core-agnostic wrapper around ICore for a non-Raylib
    // frontend (the Avalonia EmuSen.Mistress9 project) to drive a core on
    // its own schedule, independent of any particular windowing/UI
    // toolkit - call LoadRom() once and RunFrame() whenever the UI's own
    // render loop wants a new frame.
    //
    // This used to hold a full second copy of VenusCore's timing loop
    // (CPU/PPU/APU stepping, HDMA, NMI - see git history if curious just
    // how identical the two were). That's gone now; RunFrame()/SaveState()/
    // LoadState()/GetFrameBufferRgba() all delegate straight to an ICore.
    //
    // Not fully core-agnostic yet: it still constructs a VenusCore
    // directly in LoadRom() (there's only one core to construct today)
    // and exposes a Venus-specific Bus property for the one thing ICore
    // deliberately doesn't abstract - input (see ICore.cs's own comment
    // on why). A real multi-core frontend would take an ICore factory or
    // similar in its constructor instead of hardcoding VenusCore; not
    // worth building until there's a second core to actually need it
    // against.
    public class EmulatorSession
    {
        private VenusCore? _core;

        public int ScreenWidth => _core?.ScreenWidth ?? 256;
        public const int ScreenHeight = 224;

        // "SNES" before LoadRom() is called too - there's only one core to
        // report today, but this exists so a caller (e.g. a log directory
        // path) doesn't need its own core-specific fallback string.
        public string CoreName => _core?.CoreName ?? "SNES";

        public long TotalFrames => _core?.TotalFrames ?? 0;
        public bool IsRomLoaded => _core?.IsRomLoaded ?? false;

        // Venus-specific escape hatch for input - see this class's own
        // header comment and ICore.cs's comment on why input isn't part
        // of the generic interface.
        public MemoryBus Bus => _core?.Bus ?? throw new InvalidOperationException("LoadRom() hasn't been called yet.");

        // Temporary profiling pass-through - see VenusCore's own comment on
        // these. Not promoted onto ICore for the same reason input/debug
        // access aren't: this is Venus-specific instrumentation, not a
        // general execution-contract concern.
        public double LastFrameCpuSpc700Ms => _core?.LastFrameCpuSpc700Ms ?? 0;
        public double LastFramePpuMs => _core?.LastFramePpuMs ?? 0;
        public double LastFrameObjEvalMs => _core?.LastFrameObjEvalMs ?? 0;
        public double LastFrameBlendMs => _core?.LastFrameBlendMs ?? 0;
        public double LastFrameMainCompositeMs => _core?.LastFrameMainCompositeMs ?? 0;
        public double LastFrameSubCompositeMs => _core?.LastFrameSubCompositeMs ?? 0;
        public double LastFrameHdmaMs => _core?.LastFrameHdmaMs ?? 0;

        public void LoadRom(string path)
        {
            _core = new VenusCore(headless: true);
            _core.LoadRom(path);
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
            _core?.Cpu?.FlushVerboseTrace();
            _core?.Spc700?.FlushVerboseTrace();
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
    }
}
