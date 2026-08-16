using System;
using System.ComponentModel;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.IO.Compression;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // Continuous F3, core-agnostic, with pixel capture supplied by the caller - see EmuSen_Debugging_Tools_Reference_v5.md §3.8.
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

        // Every Nth frame; a long capture wants a stride rather than thinning afterwards - see §3.8.
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

        // A convenience encode, not a timing-accurate export - see §3.8.
        private const double AssumedFps = 60.0;

        // Stopping a long recording visibly freezes; acceptable for a debug tool - see §3.8.
        public void Stop()
        {
            _ledger?.Dispose();
            _ledger = null;

            string? dirToEncode = _sessionDir;
            int stride = _frameStride;
            _sessionDir = null;

            if (dirToEncode != null) TryEncodeVideo(dirToEncode, stride);
        }

        // Lossless either way; libx264rgb first, ffv1 when the build lacks it - see §3.8.
        private static readonly string[][] CodecAttempts =
        {
            new[] { "-c:v", "libx264rgb", "-crf", "0", "-preset", "slow" }, // RGB colorspace, mathematically lossless
            new[] { "-c:v", "ffv1" },
        };

        private void TryEncodeVideo(string sessionDir, int frameStride)
        {
            // Bare filename: WorkingDirectory already puts ffmpeg inside sessionDir - see §3.8.
            const string outputFile = "recording.mkv";
            string outputPath = Path.Combine(sessionDir, outputFile);
            double fps = AssumedFps / Math.Max(1, frameStride);

            for (int attempt = 0; attempt < CodecAttempts.Length; attempt++)
            {
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
                foreach (string arg in CodecAttempts[attempt]) psi.ArgumentList.Add(arg);
                psi.ArgumentList.Add(outputFile);

                try
                {
                    using Process proc = Process.Start(psi)!;
                    string stderr = proc.StandardError.ReadToEnd();
                    proc.WaitForExit();

                    if (proc.ExitCode == 0)
                    {
                        Console.WriteLine($"[RECORD] Video encoded ({CodecAttempts[attempt][1]}) -> {outputPath}");
                        ZipAndCleanupFrames(sessionDir);
                        return;
                    }

                    bool moreAttemptsLeft = attempt < CodecAttempts.Length - 1;
                    Console.WriteLine($"[RECORD] ffmpeg -c:v {CodecAttempts[attempt][1]} exited with code {proc.ExitCode}{(moreAttemptsLeft ? " - trying fallback codec" : "")}");
                    if (!moreAttemptsLeft)
                    {
                        Console.WriteLine(stderr);
                        Console.WriteLine($"[RECORD] PNG sequence + frames.log are still in {sessionDir}");
                    }
                }
                catch (Win32Exception)
                {
                    // ffmpeg absent is not an error here, and the fallback codec cannot help.
                    Console.WriteLine($"[RECORD] ffmpeg not found on PATH - leaving the PNG sequence + frames.log as-is in {sessionDir}. Install ffmpeg for automatic video output.");
                    return;
                }
            }
        }

        // The PNGs are redundant after a lossless encode, so they are zipped rather than deleted - see §3.8.
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
                // The video already succeeded, so a failed zip must not cost the frames.
                Console.WriteLine($"[RECORD] Zipping frame PNGs failed ({ex.Message}); leaving them as loose files in {sessionDir}");
            }
        }

        // Called unconditionally every frame; the no-op and the stride skip are both internal.
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
