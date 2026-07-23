using System;
using System.IO;

namespace EmuSen.Debug
{
    // Continuous version of the F3 screenshot utility: instead of one PNG
    // + one companion .txt for a single instant, captures a run of frames
    // into a session folder plus one ledger file mapping every captured
    // frame back to exactly what the emulator was doing - same
    // cross-reference purpose F3's companion .txt already served (so a
    // frame can be matched against a console.log line without
    // line-proximity guessing), just spread across a whole sequence
    // instead of one moment.
    //
    // Core-agnostic on purpose, same as the rest of this toolchain: only
    // touches IDebugTarget (CoreName/FrameCount), never anything console-
    // specific. Actually grabbing a frame's pixels isn't something this
    // shared class can do itself (that's Raylib today, on the console
    // frontend - a different API on any future frontend), so it's
    // supplied by the caller as a callback per capture instead. A future
    // NES core/frontend combination reuses this unchanged.
    public class FrameRecorder
    {
        private readonly IDebugTarget _target;
        private string? _sessionDir;
        private StreamWriter? _ledger;
        private int _frameStride;
        private long _framesSeen;

        public bool IsRecording => _sessionDir != null;
        public string? SessionDir => _sessionDir;

        public FrameRecorder(IDebugTarget target)
        {
            _target = target;
        }

        // frameStride: capture every Nth rendered frame instead of every
        // single one. Default 1 (every frame) is fine for a short,
        // targeted capture around a specific moment - the same use case
        // F3 already covers one frame at a time - but a long session at
        // 60fps/frame produces a lot of PNGs fast, so a caller chasing
        // something slower (e.g. a multi-second animation) should pass a
        // higher stride rather than thin the result out after the fact.
        public string Start(string baseDir, int frameStride = 1)
        {
            if (IsRecording) throw new InvalidOperationException("Already recording - call Stop() first.");

            _frameStride = Math.Max(1, frameStride);
            _framesSeen = 0;

            string sessionName = $"{_target.CoreName}_{DateTime.Now:yyyyMMdd_HHmmss}";
            _sessionDir = Path.Combine(baseDir, sessionName);
            Directory.CreateDirectory(_sessionDir);

            _ledger = new StreamWriter(Path.Combine(_sessionDir, "frames.log"), append: false) { AutoFlush = true };
            _ledger.WriteLine("Frame\tWallClock\tCore\tImageFile");

            return _sessionDir;
        }

        public void Stop()
        {
            _ledger?.Dispose();
            _ledger = null;
            _sessionDir = null;
        }

        // Called once per rendered frame by whatever owns the frame loop,
        // unconditionally - a cheap no-op when not recording, and the
        // stride skip when recording, both happen internally so the
        // caller doesn't need its own gating logic beyond "call this
        // every frame."
        public void CaptureFrame(Action<string> captureImage)
        {
            if (!IsRecording) return;
            if (_framesSeen++ % _frameStride != 0) return;

            string imageName = $"frame_{_target.FrameCount:D8}.png";
            string imagePath = Path.Combine(_sessionDir!, imageName);
            captureImage(imagePath);

            string wallClock = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss.fff");
            _ledger!.WriteLine($"{_target.FrameCount}\t{wallClock}\t{_target.CoreName}\t{imageName}");
        }
    }
}
