using System;
using System.Collections.Generic;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using EmuSen.Common.Imaging;

namespace EmuSen.WiseMan.Fixtures
{
    // A captured window, as plain RGBA8888 - the same shape FrameHash and BmpFile already take.
    public readonly record struct RenderedFrame(byte[] Rgba, int Width, int Height)
    {
        public ulong Hash => FrameHash.Compute(Rgba);

        // Stops early once the caller has seen enough; a flat-image check does not need the true total.
        public int DistinctColours(int stopAt = int.MaxValue)
        {
            var seen = new HashSet<uint>();
            for (int i = 0; i + 3 < Rgba.Length; i += 4)
            {
                seen.Add((uint)(Rgba[i] | (Rgba[i + 1] << 8) | (Rgba[i + 2] << 16)));
                if (seen.Count >= stopAt) break;
            }

            return seen.Count;
        }

        public void SavePng(string path)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            EmuSen.Hotaru.Imaging.FrameImageWriter.SavePng(Rgba, Width, Height, path);
        }
    }

    // The one place a UI test dispatches, captures and asserts - see EmuSen_LunaP.md §13.
    public static class UiTest
    {
        // EMUSEN_UI_DUMP names a directory; every capture in the run lands in it as <name>.png.
        private const string DumpVariable = "EMUSEN_UI_DUMP";

        // Opt-in pixel-exact comparison, off unless both are set - see EmuSen_LunaP.md §13.2.
        private const string BaselineVariable = "EMUSEN_UI_BASELINE";
        private const string BaselineModeVariable = "EMUSEN_UI_BASELINE_MODE";

        public static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(UiTest).GetTypeInfo().Assembly);

        // Window and control construction is only valid on the one dispatcher this session owns.
        public static Task Run(Action body) => Session.Dispatch(body, default);

        public static RenderedFrame Capture(Window window)
        {
            using WriteableBitmap bitmap = window.CaptureRenderedFrame()
                ?? throw new InvalidOperationException($"{window.GetType().Name} captured no frame - it was probably never shown.");

            return Capture(bitmap);
        }

        public static RenderedFrame Capture(WriteableBitmap bitmap)
        {
            int width = bitmap.PixelSize.Width;
            int height = bitmap.PixelSize.Height;
            var rgba = new byte[width * height * 4];
            using (ILockedFramebuffer fb = bitmap.Lock()) Marshal.Copy(fb.Address, rgba, 0, rgba.Length);
            return new RenderedFrame(rgba, width, height);
        }

        // The always-on assertion: a window that failed to lay out, or whose controls have no template, renders as one flat colour.
        public static RenderedFrame AssertLaidOut(Window window, string name, int minColours = 8)
        {
            RenderedFrame frame = Capture(window);
            Dump(name, frame);

            int distinct = frame.DistinctColours(minColours + 1);
            Assert.True(distinct > minColours,
                $"{name} rendered {distinct} distinct colours - layout or templating probably failed. Set {DumpVariable} to a directory and look at it.");

            AssertMatchesBaseline(name, frame);
            return frame;
        }

        // Builds and renders twice. A window that fails this can never be compared against a baseline - see EmuSen_LunaP.md §13.2.
        public static void AssertStable(string name, Func<Window> build)
        {
            Window first = build();
            first.Show();
            ulong firstHash = Capture(first).Hash;
            first.Close();

            Window second = build();
            second.Show();
            ulong secondHash = Capture(second).Hash;
            second.Close();

            Assert.True(firstHash == secondHash,
                $"{name} rendered differently on two identical runs, so it shows something live (a clock, a pid, a counter). It is not a valid baseline target.");
        }

        public static void Dump(string name, RenderedFrame frame)
        {
            if (Environment.GetEnvironmentVariable(DumpVariable) is not { Length: > 0 } directory) return;

            frame.SavePng(Path.Combine(directory, name + ".png"));
        }

        // No-op unless a baseline directory and mode are both set; the caller's own assertions are what run in CI.
        public static void AssertMatchesBaseline(string name, RenderedFrame frame)
        {
            if (Environment.GetEnvironmentVariable(BaselineVariable) is not { Length: > 0 } directory) return;

            string mode = Environment.GetEnvironmentVariable(BaselineModeVariable) ?? "compare";
            string path = Path.Combine(directory, name + ".frame");

            if (string.Equals(mode, "write", StringComparison.OrdinalIgnoreCase))
            {
                WriteBaseline(path, frame);
                return;
            }

            Assert.True(File.Exists(path), $"No baseline for {name} at {path}. Run once with {BaselineModeVariable}=write first.");

            RenderedFrame baseline = ReadBaseline(path);
            Assert.True(baseline.Width == frame.Width && baseline.Height == frame.Height,
                $"{name} changed size: baseline {baseline.Width}x{baseline.Height}, now {frame.Width}x{frame.Height}.");

            Assert.True(baseline.Hash == frame.Hash, $"{name} rendered {DifferingPixels(baseline, frame)} pixels differently from its baseline.");
        }

        private static int DifferingPixels(RenderedFrame a, RenderedFrame b)
        {
            int differing = 0;
            for (int i = 0; i + 3 < a.Rgba.Length; i += 4)
            {
                if (a.Rgba[i] != b.Rgba[i] || a.Rgba[i + 1] != b.Rgba[i + 1] || a.Rgba[i + 2] != b.Rgba[i + 2]) differing++;
            }

            return differing;
        }

        // Dimensions then raw pixels, one file, so a half-written pair can never be mistaken for a match.
        private static void WriteBaseline(string path, RenderedFrame frame)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);

            using var stream = new BinaryWriter(File.Create(path));
            stream.Write(frame.Width);
            stream.Write(frame.Height);
            stream.Write(frame.Rgba);
        }

        private static RenderedFrame ReadBaseline(string path)
        {
            using var stream = new BinaryReader(File.OpenRead(path));
            int width = stream.ReadInt32();
            int height = stream.ReadInt32();
            return new RenderedFrame(stream.ReadBytes(width * height * 4), width, height);
        }
    }
}
