using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Common.Firmware;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores
{
    // The core-agnostic execution contract - see EmuSen_Multicore.md and EmuSen_Input.md §1.
    public interface ICore
    {
        // Short display name, matching IDebugTarget.CoreName.
        string CoreName { get; }

        // Width can vary by frame where a core has a hi-res mode - see Venus_CPU.md §12.
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

        // Answered without loading, so a caller can resolve it first - see EmuSen_Firmware.md §1.
        IReadOnlyList<FirmwareRequest> GetFirmwareRequirements(string romPath) => Array.Empty<FirmwareRequest>();

        // One frame of internal timing; touches no presentation surface. Not before LoadRom.
        void RunFrame();

        // Plain RGBA8888, so no surface needs to know a core's native pixel format.
        byte[] GetFrameBufferRgba();

        // Fixed for the session, so a device can be opened before any samples exist.
        int AudioSampleRate { get; }

        // Destructive and non-blocking, unlike GetAudioSamples' snapshot - see §3.1.
        short[] DequeueAudioSamples(int maxFrames);

        void SaveState(string path);
        void LoadState(string path);

        // The same bytes without the filesystem - see EmuSen_Rewind_And_FastForward.md §1.1.
        void SaveState(Stream stream);
        void LoadState(Stream stream);

        // A fast-forward hint; a core honouring it may leave render-derived bits stale - see §2.2.
        bool SkipRendering { get; set; }

        // Stopped mid-frame in front of a breakpoint, so the next call resumes it - see EmuSen_Multicore.md §5.
        bool IsHaltedAtBreakpoint => false;

        int HaltedAddress => 0;

        // Flushes battery-backed save data; a core without the concept no-ops.
        void SaveSram();
    }
}
