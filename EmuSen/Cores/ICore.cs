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
    // Deliberately does NOT try to abstract input. Different consoles have
    // wildly different controller shapes (SNES's 12 digital buttons vs.
    // N64's analog stick vs. a Game Boy's 8 buttons), and the Avalonia
    // frontend already has a substantial amount of code (ControllerKeyMap,
    // GamepadBindingMap, the rebind UI, persisted JSON bindings) built
    // specifically around SnesButton. Forcing that through a generic
    // interface now would be a much bigger, riskier piece of work than
    // this contract's actual purpose - eliminating the duplicated
    // frame-timing loop that used to exist separately in both the console
    // frontend's Program.cs (now EmuSen.Hotaru/Program.cs) and
    // Common/EmulatorSession.cs. A core's
    // concrete implementation (e.g. VenusCore) is free to expose its own
    // real input surface beyond this interface; callers that need it
    // (a frontend wiring up keyboard/gamepad input) already know which
    // concrete core they're driving.
    //
    // Same reasoning for debug-toolchain access: Program.cs's F1-F9
    // hotkeys and the F4 debug prompt need real SNES-specific objects
    // (Cpu, MemoryBus, Renderer) to build a SnesDebugTarget from - those
    // stay on the concrete core type too, not on this interface.
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

        bool IsRomLoaded { get; }
        long TotalFrames { get; }

        void LoadRom(string path);

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

        // Flushes battery-backed save data (SRAM or whatever this
        // hardware's equivalent is) to disk. A core with no such concept
        // can no-op this.
        void SaveSram();
    }
}
