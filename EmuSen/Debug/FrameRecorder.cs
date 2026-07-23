using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.IO.Compression;

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

        // Approximate SNES/NES-class refresh rate this project already
        // treats as "close enough" elsewhere (see Program.cs's
        // SaveEveryNFrames comment, "~5 seconds at 60fps") - real hardware
        // is closer to 60.0988Hz, but this is a debug-tool convenience
        // encode, not a timing-accurate export, so the same approximation
        // already used throughout the codebase is fine here too.
        private const double AssumedFps = 60.0;

        // Blocks synchronously while ffmpeg encodes (if available) - for a
        // long recording this means a visible freeze when stopping, same
        // general tradeoff the F4 debug prompt already has (the execution
        // loop can't pause mid-frame independent of this either way).
        // Acceptable for a debug tool; would need to move off the main
        // thread if this ever needs to not block real-time play.
        public void Stop()
        {
            _ledger?.Dispose();
            _ledger = null;

            string? dirToEncode = _sessionDir;
            int stride = _frameStride;
            _sessionDir = null;

            if (dirToEncode != null) TryEncodeVideo(dirToEncode, stride);
        }

        // Muxes the just-captured PNG sequence into a single lossless
        // video file via ffmpeg, if it's available on PATH - never
        // required (the PNG sequence + frames.log ledger stand on their
        // own either way, and are never deleted by this), just a
        // convenience so a recording doesn't have to be reviewed as a
        // folder of thousands of loose images.
        //
        // FFV1-in-Matroska specifically, not a lossy codec like VP9/H.264:
        // this exists to inspect exact pixel-level rendering bugs, and a
        // lossy codec's own compression artifacts would undermine that -
        // trading file size for fidelity is the wrong tradeoff for a
        // debugging tool. Both FFV1 and Matroska are open formats.
        //
        // HONESTY NOTE: verified against a real run on the project's own
        // dev machine (Fedora 44, ffmpeg 8.1.2) - the first attempt
        // failed with "No such file or directory" from a WorkingDirectory/
        // output-path double-nesting bug (see outputFile's comment
        // above), now fixed. The -pix_fmt rgb24 that run also flagged as
        // "incompatible... auto-selecting bgr0" has been removed in favor
        // of letting ffmpeg negotiate it, since the auto-select is exactly
        // what worked. Re-verify after this fix on a real recording.
        private void TryEncodeVideo(string sessionDir, int frameStride)
        {
            // Output filename only, NOT sessionDir/recording.mkv - the
            // process's WorkingDirectory below already puts ffmpeg inside
            // sessionDir, so a full relative path here would resolve
            // relative to that (a real bug caught on first real-world run:
            // ffmpeg tried to open sessionDir/sessionDir/recording.mkv,
            // which obviously doesn't exist, and failed outright).
            const string outputFile = "recording.mkv";
            string outputPath = Path.Combine(sessionDir, outputFile);
            double fps = AssumedFps / Math.Max(1, frameStride);

            var psi = new ProcessStartInfo
            {
                FileName = "ffmpeg",
                WorkingDirectory = sessionDir,
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true,
            };
            psi.ArgumentList.Add("-y");
            psi.ArgumentList.Add("-pattern_type");
            psi.ArgumentList.Add("glob");
            psi.ArgumentList.Add("-framerate");
            psi.ArgumentList.Add(fps.ToString(CultureInfo.InvariantCulture));
            psi.ArgumentList.Add("-i");
            psi.ArgumentList.Add("frame_*.png");
            psi.ArgumentList.Add("-c:v");
            psi.ArgumentList.Add("ffv1");
            // No forced -pix_fmt: Raylib's screenshots are RGBA, and
            // letting ffmpeg auto-negotiate a format FFV1 actually
            // supports (confirmed working: it auto-picked bgr0 on a real
            // run) is more robust than guessing one ourselves.
            psi.ArgumentList.Add(outputFile);

            try
            {
                using Process proc = Process.Start(psi)!;
                string stderr = proc.StandardError.ReadToEnd();
                proc.WaitForExit();

                if (proc.ExitCode == 0)
                {
                    Console.WriteLine($"[RECORD] Video encoded -> {outputPath}");
                    ZipAndCleanupFrames(sessionDir);
                }
                else
                {
                    Console.WriteLine($"[RECORD] ffmpeg exited with code {proc.ExitCode}; PNG sequence + frames.log are still in {sessionDir}");
                    Console.WriteLine(stderr);
                }
            }
            catch (Win32Exception)
            {
                // ffmpeg isn't on PATH - not an error condition for this
                // tool, just means the convenience encode step is
                // unavailable. The PNG sequence and ledger are already
                // complete and useful on their own.
                Console.WriteLine($"[RECORD] ffmpeg not found on PATH - leaving the PNG sequence + frames.log as-is in {sessionDir}. Install ffmpeg for automatic video output.");
            }
        }

        // Only called after a confirmed-successful encode (ExitCode == 0)
        // - at that point the loose PNGs are fully redundant (FFV1 is
        // lossless, so recording.mkv already has everything they do), and
        // a folder of potentially thousands of individual image files is
        // exactly the "hot mess" this whole video-encode step exists to
        // avoid. Zipped rather than deleted outright so the raw frames
        // stay recoverable (just compressed) instead of gone - frames.log
        // (the frame/timestamp ledger) is left alone either way, it's
        // small and still useful for cross-referencing without needing to
        // unzip anything.
        private static void ZipAndCleanupFrames(string sessionDir)
        {
            string[] pngFiles = Directory.GetFiles(sessionDir, "frame_*.png");
            if (pngFiles.Length == 0) return;

            string zipPath = Path.Combine(sessionDir, "frames.zip");
            try
            {
                using (var zipStream = new FileStream(zipPath, FileMode.Create))
                using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create))
                {
                    foreach (string png in pngFiles)
                    {
                        archive.CreateEntryFromFile(png, Path.GetFileName(png), CompressionLevel.Optimal);
                    }
                }

                foreach (string png in pngFiles) File.Delete(png);
                Console.WriteLine($"[RECORD] Archived {pngFiles.Length} frame PNGs -> {zipPath}");
            }
            catch (Exception ex)
            {
                // Non-critical: the video already succeeded, so this is
                // purely tidiness. Leave the loose PNGs in place (don't
                // delete anything a failed zip didn't actually capture)
                // rather than risk losing data over a cleanup step.
                Console.WriteLine($"[RECORD] Zipping frame PNGs failed ({ex.Message}); leaving them as loose files in {sessionDir}");
            }
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
