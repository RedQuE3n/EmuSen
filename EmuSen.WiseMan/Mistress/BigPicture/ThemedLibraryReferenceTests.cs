using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Endymion;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Runs only when asked, since it writes pictures outside the repository for a person to look at.
    public sealed class ThemedPngFactAttribute : FactAttribute
    {
        public ThemedPngFactAttribute()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_BIGPICTURE_PNG") != "1")
                Skip = "Writes stage (e)'s pictures to ~/.cache/emusen/bigpicture/png/stage-e/; set EMUSEN_BIGPICTURE_PNG=1 - see EmuSen_BigPicture.md §15";
        }
    }

    // Art Book Next as the themed library, read in place and never copied; and the stage's pictures of the whole window - see EmuSen_BigPicture.md §15.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedLibraryReferenceTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedLibraryReferenceTests).GetTypeInfo().Assembly);

        public static readonly string MediaRoot = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "media", "downloaded_media");
        public static readonly string PngFolder = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "bigpicture", "png", "stage-e");

        private readonly ITestOutputHelper _out;

        public ThemedLibraryReferenceTests(ITestOutputHelper output) => _out = output;

        private static ThemedSession ArtBookNext(double w = 1280, double h = 800)
        {
            SyntheticLibrary.WriteMedia(MediaRoot);
            return new ThemedSession(w, h, a => a.EsdeMediaDirectory = MediaRoot, themeDirectory: ArtBookNextFactAttribute.Folder);
        }

        // P41 and the first half of P44: Art Book Next shown for the library's three systems, its first build, and its seven sounds decoded.
        [ArtBookNextFact]
        public Task Art_Book_Next_is_the_library_of_a_themed_session_and_is_steered_by_the_pad() => Session.Dispatch(() =>
        {
            var clock = Stopwatch.StartNew();
            using ThemedSession s = ArtBookNext();
            TimeSpan window = clock.Elapsed;
            Assert.True(s.Shown, s.Themed.Error);
            _out.WriteLine($"window built and shown: {window.TotalMilliseconds:F0} ms; last Show: theme {s.Themed.LoadTime.TotalMilliseconds:F1} ms, stage {s.Themed.BuildTime.TotalMilliseconds:F1} ms");
            clock.Restart();
            s.Capture();
            _out.WriteLine($"first capture of the window: {clock.Elapsed.TotalMilliseconds:F0} ms");
            Assert.Equal(new[] { "nes", "gb", "snes" }, s.Themed.Stage!.Current.Data.Systems.Select(x => x.System.Name));

            var files = s.Themed.SoundFiles.ToList();
            Assert.Equal(7, files.Count);
            clock.Restart();
            foreach (string f in files) Assert.NotNull(UiSoundPlayer.Decode(f));
            _out.WriteLine($"seven sounds decoded: {clock.Elapsed.TotalMilliseconds:F1} ms");

            s.Pad.Right(2);
            Assert.Equal("snes", s.System);
            s.Pad.A();
            Assert.Equal("gamelist", s.View);
            s.Pad.Down();
            Assert.Equal(ThemedSession.SnesGames[1], s.Game);
            Assert.Equal(new[] { "systembrowse", "systembrowse", "select", "scroll" }, s.Sounds);
            s.Settle();
            _out.WriteLine($"gamelist settled: next frame wanted at {s.WakeAt?.TotalMilliseconds.ToString() ?? "never"} (now {s.Now.TotalMilliseconds})");
            Assert.Equal(0, s.Loop(2900));
        }, default);

        // P41 by itself: the theme read for the five systems, the stage built, and the first frame of it laid out and drawn; five runs, the first cold in this process.
        [ArtBookNextFact]
        public Task Art_Book_Next_s_first_build_for_five_systems() => Session.Dispatch(() =>
        {
            SyntheticLibrary.WriteMedia(MediaRoot);
            var shelves = SyntheticLibrary.Systems.Select(x => new EmuSen.Mistress.BigPicture.ThemedShelf(x.System, SyntheticLibrary.Games(x.System, x.Extension))).ToList();
            var rows = new List<(double Load, double Build, double Frame)>();
            for (int run = 0; run < 5; run++)
            {
                var library = new EmuSen.Mistress.BigPicture.ThemedLibrary(() => TimeSpan.Zero) { Status = new DeviceStatus() };
                var clock = Stopwatch.StartNew();
                Assert.True(library.Show(ArtBookNextFactAttribute.Folder, new Size(1280, 800), shelves, new EmuSen.Mistress.BigPicture.Scene.EsdeMediaFolder(MediaRoot)), library.Error);
                double show = clock.Elapsed.TotalMilliseconds;
                var window = new Window { Width = 1280, Height = 800, Content = library.Root, Background = Avalonia.Media.Brushes.Black };
                clock.Restart();
                window.Show();
                using (var bitmap = window.CaptureRenderedFrame()) { }
                double frame = clock.Elapsed.TotalMilliseconds;
                window.Close();
                rows.Add((library.LoadTime.TotalMilliseconds, library.BuildTime.TotalMilliseconds, frame));
                Assert.Equal(5, library.Stage!.Current.Data.Systems.Count);
                _out.WriteLine($"run {run}: theme {library.LoadTime.TotalMilliseconds:F1} ms, media {library.MediaTime.TotalMilliseconds:F1} ms, stage {library.BuildTime.TotalMilliseconds:F1} ms, Show {show:F1} ms, first frame {frame:F1} ms, systems {library.Stage!.Current.Data.Systems.Count}");
            }
        }, default);

        // The PNGs of §15: both sizes, system view, gamelist, the pad menu over the view, the help bar in each family; synthetic theme always, Art Book Next when cloned.
        [ThemedPngFact]
        public Task Stage_e_pictures() => Session.Dispatch(() =>
        {
            Directory.CreateDirectory(PngFolder);
            foreach ((double w, double h) in new[] { (1280.0, 800.0), (1920.0, 1200.0) })
            {
                using (var s = new ThemedSession(w, h)) Pictures(s, $"synthetic-{w}x{h}");
                if (File.Exists(Path.Combine(ArtBookNextFactAttribute.Folder, "capabilities.xml")))
                    using (ThemedSession s = ArtBookNext(w, h)) Pictures(s, $"artbooknext-{w}x{h}");
            }
        }, default);

        private void Pictures(ThemedSession s, string name)
        {
            Assert.True(s.Shown, s.Themed.Error);
            s.Pad.Right(2);
            s.Run(500);
            Save(s.Capture(), $"{name}-system");
            s.Pad.A();
            s.Pad.Down();
            s.Run(500);
            Save(s.Capture(), $"{name}-gamelist");
            s.Pad.Start();
            Save(s.Capture(), $"{name}-padmenu");
            s.Pad.B();

            var bars = new List<RenderedFrame>();
            foreach ((string type, string pad) in new[] { ("Unknown", "Some pad"), ("XboxOne", "Xbox"), ("PS5", "DualSense"), ("NintendoSwitchPro", "Pro Controller") })
            {
                s.Pad.Pad.Type = Enum.Parse<SDL.GamepadType>(type);
                s.Pad.Pad.Name = pad;
                s.Pad.Tick();
                RenderedFrame frame = s.Capture();
                HintBar bar = s.Themed.Stage!.Current.Scene.Entries.Select(e => e.Control).OfType<HintBar>().First();
                var box = new Rect(bar.TranslatePoint(default, s.Window)!.Value, bar.Bounds.Size).Inflate(8);
                bars.Add(Crop(frame, box));
            }
            Save(Stack(bars), $"{name}-helpbar-families");
            s.Pad.Pad.Type = SDL.GamepadType.Unknown;
        }

        private void Save(RenderedFrame frame, string name)
        {
            string path = Path.Combine(PngFolder, name + ".png");
            frame.SavePng(path);
            _out.WriteLine(path);
        }

        private static RenderedFrame Crop(RenderedFrame f, Rect box)
        {
            int x0 = Math.Max(0, (int)box.X), y0 = Math.Max(0, (int)box.Y), x1 = Math.Min(f.Width, (int)Math.Ceiling(box.Right)), y1 = Math.Min(f.Height, (int)Math.Ceiling(box.Bottom));
            int w = x1 - x0, h = y1 - y0;
            var rgba = new byte[w * h * 4];
            for (int y = 0; y < h; y++) Array.Copy(f.Rgba, ((y0 + y) * f.Width + x0) * 4, rgba, y * w * 4, w * 4);
            return new RenderedFrame(rgba, w, h);
        }

        // Frames one above the other, left-aligned, on grey, 6 px apart.
        private static RenderedFrame Stack(IReadOnlyList<RenderedFrame> frames)
        {
            int w = frames.Max(f => f.Width), h = frames.Sum(f => f.Height) + 6 * (frames.Count - 1);
            var rgba = new byte[w * h * 4];
            for (int i = 0; i < rgba.Length; i += 4) { rgba[i] = rgba[i + 1] = rgba[i + 2] = 96; rgba[i + 3] = 255; }
            int top = 0;
            foreach (RenderedFrame f in frames)
            {
                for (int y = 0; y < f.Height; y++) Array.Copy(f.Rgba, y * f.Width * 4, rgba, ((top + y) * w) * 4, f.Width * 4);
                top += f.Height + 6;
            }
            return new RenderedFrame(rgba, w, h);
        }
    }
}
