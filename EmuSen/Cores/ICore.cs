using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Common.Firmware;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores
{
    // The core-agnostic execution contract - the missing counterpart to
    // Debug/IDebugTarget.cs. IDebugTarget already proved this pattern works:
    // a narrow interface any core implements, with everything built on top
    // of it (DianaOSInterpreter, WatchRegistry, FrameRecorder) written
    // once and reused unchanged for whatever core is plugged in. This is
    // the same idea applied to actually *running* a core, not just
    // inspecting it.
    //
    // Input joined this contract late, and console-specific blocks never did - see EmuSen_Input.md §1 and EmuSen_Multicore.md §5.
    public interface ICore
    {
        // Short display name ("SNES", "NES", etc) - same convention as
        // IDebugTarget.CoreName.
        string CoreName { get; }

        // Current frame's pixel dimensions. Width can vary frame-to-frame
        // for cores with a variable-width display mode (SNES pseudo-hi-res
        // is 512 wide instead of 256) - see VenusCore's own comment.
        int ScreenWidth { get; }
        int ScreenHeight { get; }

        // Hardware refresh rate, for frontend frame pacing - see Venus_CPU.md §8.5b.
        double FrameRateHz { get; }

        bool IsRomLoaded { get; }
        long TotalFrames { get; }

        // The buttons this console actually has, so a rebind UI can show a real pad - see EmuSen_Input.md §2.
        IReadOnlyList<PadButton> SupportedButtons => Array.Empty<PadButton>();

        // <port> is 0-based. A core with no input modelled can leave this alone.
        void SetButton(int port, PadButton button, bool pressed) { }

        void LoadRom(string path);

        // Firmware this ROM needs that the core cannot supply itself - a
        // coprocessor's mask ROM, a console BIOS, whatever the hardware had
        // that isn't in the cartridge. Answered WITHOUT loading, so a caller
        // can resolve anything missing (see FirmwareLibrary) before LoadRom
        // rather than discovering it afterwards. Defaults to none, which is
        // right for any core whose hardware needs nothing external.
        //
        // A missing image is never fatal: LoadRom still succeeds and the core
        // runs with that chip absent. See EmuSen_Firmware.md §1.
        IReadOnlyList<FirmwareRequest> GetFirmwareRequirements(string romPath) => Array.Empty<FirmwareRequest>();

        // Runs exactly one frame's worth of internal timing (however many
        // scanlines/cycles/whatever unit makes sense for this hardware),
        // then returns. Does not touch any window/presentation surface -
        // see GetFrameBufferRgba() for that. Must not be called before
        // LoadRom().
        void RunFrame();

        // Plain RGBA8888 bytes, ScreenWidth*ScreenHeight*4 long, ready to
        // copy into any presentation surface (a Raylib texture, an
        // Avalonia WriteableBitmap, whatever) without that surface needing
        // to know anything about this core's native pixel format.
        byte[] GetFrameBufferRgba();

        // Sample rate this core's audio is generated at, fixed for the
        // whole session - a caller opening a real output device needs this
        // once up front, before any samples exist to read a rate off of,
        // so it's its own property rather than folded into
        // DequeueAudioSamples' return the way IDebugTarget.GetAudioSamples
        // bundles (Samples, SampleRate) together for a one-shot snapshot.
        int AudioSampleRate { get; }

        // The audio counterpart to GetFrameBufferRgba() above: drains up
        // to <maxFrames> interleaved stereo frames (maxFrames*2 shorts,
        // L then R, 16-bit signed PCM) of already-synthesized audio out of
        // this core's internal buffer, so a caller can push real sound to
        // its own output device (a Raylib AudioStream, an SDL audio
        // device, whatever) without knowing anything about this core's
        // synthesis hardware. Destructive - removes exactly what it
        // returns - and never blocks: returns fewer frames than requested,
        // or an empty array, if less is currently buffered. Deliberately
        // NOT the same method as IDebugTarget.GetAudioSamples(), which is
        // a non-destructive ToArray() snapshot built for one-shot WAV
        // export (`audiodump`) - real-time playback needs to actually
        // drain the queue as it consumes it, or the buffer would just grow
        // until it hits its own cap and starts dropping the oldest
        // samples. A core with no audio output modeled can return an
        // empty array unconditionally.
        short[] DequeueAudioSamples(int maxFrames);

        void SaveState(string path);
        void LoadState(string path);

        // Same bytes as the path overloads above, without the filesystem - see
        // EmuSen_Rewind_And_FastForward.md §1.1. Neither closes the stream.
        void SaveState(Stream stream);
        void LoadState(Stream stream);

        // Fast-forward-only hint that this frame's pixels are discarded; a core
        // honoring it may leave render-derived bits stale - see §2.2.
        bool SkipRendering { get; set; }

        // Stopped mid-frame in front of a breakpoint, so the next call resumes it - see EmuSen_Multicore.md §5.
        bool IsHaltedAtBreakpoint => false;

        int HaltedAddress => 0;

        // Flushes battery-backed save data (SRAM or whatever this
        // hardware's equivalent is) to disk. A core with no such concept
        // can no-op this.
        void SaveSram();
    }
}
