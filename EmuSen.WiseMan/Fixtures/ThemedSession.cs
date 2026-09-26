using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Text;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Views;
using SDL3;

namespace EmuSen.WiseMan.Fixtures
{
    // A big-screen Mistress window whose library is a theme's view, a pad with no device, a clock the test moves and the sounds recorded - see EmuSen_BigPicture.md §15.
    public sealed class ThemedSession : IDisposable
    {
        public static readonly string[] SnesGames = ["Aurora Drift (Synthetic)", "Brass Lantern (Synthetic)", "Cobalt Harbor (Synthetic)", "Dune Relay (Synthetic)", "Ember Circuit (Synthetic)"];
        public static readonly string[] NesGames = ["Fable of Tiles (Synthetic)", "Granite Choir (Synthetic)"];
        public static readonly string[] GbGames = ["Hollow Comet (Synthetic)"];
        public static readonly string[] SoundNames = ["systembrowse", "quicksysselect", "select", "back", "scroll", "favorite", "launch"];

        private static readonly BindingFlags Hidden = BindingFlags.Instance | BindingFlags.NonPublic;

        public ThemedSession(double width = 1280, double height = 800, Action<AppSettings>? settings = null, string? themeDirectory = null, string? extraGamelist = null)
        {
            Root = Path.Combine(Path.GetTempPath(), "EmuSenThemedSession", Guid.NewGuid().ToString("N"));
            RomDirectory = Path.Combine(Root, "Roms");
            Directory.CreateDirectory(RomDirectory);
            ConfigStore.OverrideDirectory = Path.Combine(Root, "Config");
            DataStore.OverrideDirectory = Path.Combine(Root, "Home");
            foreach (string g in SnesGames) File.WriteAllBytes(Path.Combine(RomDirectory, g + ".sfc"), SyntheticRom.BuildBlank());
            foreach (string g in NesGames) File.WriteAllBytes(Path.Combine(RomDirectory, g + ".nes"), new byte[64]);
            foreach (string g in GbGames) File.WriteAllBytes(Path.Combine(RomDirectory, g + ".gb"), new byte[0x200]);

            Theme = new SyntheticTheme();
            if (themeDirectory is null) Write(Theme, extraGamelist);
            var app = new AppSettings
            {
                RomDirectory = RomDirectory, BigScreen = true, BigPictureTheme = themeDirectory ?? Theme.Root, LibraryStyle = AppSettings.LibraryStyleTheme,
                ResumeOnLaunch = AppSettings.ResumeNever, LibraryView = AppSettings.LibraryList,
            };
            settings?.Invoke(app);
            app.Save();

            Window = new MainWindow { Width = width, Height = height };
            Set("UiClock", (Func<TimeSpan>)(() => Now));
            Set("UiSoundSink", (Action<string>)(path => Sounds.Add(Path.GetFileNameWithoutExtension(path))));
            Window.Show();
            Pad = new PadDriver(Window);
            Settle();
        }

        public string Root { get; }
        public string RomDirectory { get; }
        public SyntheticTheme Theme { get; }
        public MainWindow Window { get; }
        public PadDriver Pad { get; }
        public TimeSpan Now { get; set; }
        public List<string> Sounds { get; } = new();

        public ThemedLibrary Themed => (ThemedLibrary)Get("Themed")!;
        public bool Shown => (bool)Get("ThemedLibraryShown")!;
        public TimeSpan? WakeAt => (TimeSpan?)Get("ThemedWakeAt");
        public int FramesDrawn => (int)Get("ThemedFramesDrawn")!;

        public string? System => Themed.SelectedSystem?.System.Name;
        public string? Game => Themed.SelectedGame?.Name;
        public string View => Themed.ViewName;

        private object? Get(string property) => typeof(MainWindow).GetProperty(property, Hidden)!.GetValue(Window);
        private void Set(string property, object value) => typeof(MainWindow).GetProperty(property, Hidden)!.SetValue(Window, value);

        public void Frame() => typeof(MainWindow).GetMethod("ThemedFrame", Hidden)!.Invoke(Window, null);

        // Lets layout and posted work run, and the view's clock catch up with the test's.
        public void Settle()
        {
            Dispatcher.UIThread.RunJobs();
            Window.UpdateLayout();
            Dispatcher.UIThread.RunJobs();
            if (Shown) Frame();
        }

        // The pad polled every 16 ms while the clock runs on, as the window's own timer polls it.
        public void Run(double ms, double step = 16)
        {
            TimeSpan end = Now + TimeSpan.FromMilliseconds(ms);
            while (Now < end)
            {
                Now += TimeSpan.FromMilliseconds(Math.Min(step, (end - Now).TotalMilliseconds));
                Pad.Tick();
                if (Shown) Frame();
            }
        }

        // The window's own loop, modelled: the pad polled every 16 ms and a frame drawn only when the view asked for one by then; returns the frames drawn.
        public int Loop(double ms, double step = 16)
        {
            int frames = 0;
            TimeSpan end = Now + TimeSpan.FromMilliseconds(ms);
            while (Now < end)
            {
                Now += TimeSpan.FromMilliseconds(Math.Min(step, (end - Now).TotalMilliseconds));
                Pad.Tick();
                if (WakeAt is { } due && due <= Now)
                {
                    Frame();
                    frames++;
                }
            }
            return frames;
        }

        // A button held for a while, polled as the window polls it, then let go.
        public void Hold(SDL.GamepadButton button, double ms)
        {
            Pad.Pad.Press(button);
            Pad.Tick();
            Run(ms);
            Pad.Pad.Release(button);
            Pad.Tick();
        }

        // The whole window redrawn and read back.
        public RenderedFrame Capture()
        {
            Settle();
            Window.CaptureRenderedFrame()?.Dispose();
            foreach (Visual v in Window.GetSelfAndVisualDescendants()) v.InvalidateVisual();
            using WriteableBitmap bitmap = Window.CaptureRenderedFrame() ?? throw new InvalidOperationException("no frame");
            return UiTest.Capture(bitmap);
        }

        // A theme written for these tests: a carousel of one picture per system, a list, a cover, a help bar in each view, and all seven sounds.
        public static void Write(SyntheticTheme theme, string? extraGamelist = null)
        {
            var art = new (string System, Color Left, Color Right)[] { ("nes", Colors.Firebrick, Colors.Gold), ("gb", Colors.SeaGreen, Colors.Khaki), ("snes", Colors.SlateBlue, Colors.Orchid) };
            foreach ((string system, Color left, Color right) in art)
            {
                string path = theme.PathOf($"art/{system}.png");
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.Copy(SceneAssets.Halves($"session-{system}", 120, 240, left, right), path, overwrite: true);
            }

            foreach (string name in SoundNames) Wav(theme.PathOf($"sounds/{name}.wav"), 220 + 110 * Array.IndexOf(SoundNames, name));

            const string help = "<helpsystem name=\"help\"><pos>0.03 0.95</pos><origin>0 1</origin><entries>all</entries><fontSize>0.035</fontSize>" +
                                "<textColor>FFFFFF</textColor><iconColor>FFFFFF</iconColor><backgroundColor>202020FF</backgroundColor></helpsystem>";
            string sounds = string.Concat(SoundNames.Select(n => $"<sound name=\"{n}\"><path>./sounds/{n}.wav</path></sound>"));
            theme.Capabilities("").Theme(
                "<view name=\"system\">" +
                "<carousel name=\"systemcarousel\"><pos>0 0.15</pos><size>1 0.6</size><maxItemCount>3</maxItemCount><itemScale>1.2</itemScale>" +
                "<staticImage>./art/${system.theme}.png</staticImage></carousel>" + help + "</view>" +
                "<view name=\"gamelist\">" +
                "<textlist name=\"gamelist\"><pos>0.05 0.08</pos><size>0.45 0.8</size><fontSize>0.045</fontSize><primaryColor>FFFFFF</primaryColor>" +
                "<selectedColor>FFFF00</selectedColor><selectedBackgroundColor>3050A0FF</selectedBackgroundColor></textlist>" +
                "<image name=\"cover\"><pos>0.55 0.08</pos><maxSize>0.4 0.7</maxSize><imageType>cover</imageType></image>" +
                (extraGamelist ?? "") + help + "</view>" +
                "<view name=\"all\">" + sounds + "</view>");
        }

        // A short tone as 16-bit mono PCM, so SDL has a real file to decode.
        public static void Wav(string path, double hz, int rate = 22050, double seconds = 0.05)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            int n = (int)(rate * seconds);
            using var w = new BinaryWriter(File.Create(path));
            w.Write(Encoding.ASCII.GetBytes("RIFF")); w.Write(36 + n * 2); w.Write(Encoding.ASCII.GetBytes("WAVEfmt "));
            w.Write(16); w.Write((short)1); w.Write((short)1); w.Write(rate); w.Write(rate * 2); w.Write((short)2); w.Write((short)16);
            w.Write(Encoding.ASCII.GetBytes("data")); w.Write(n * 2);
            for (int i = 0; i < n; i++) w.Write((short)(Math.Sin(2 * Math.PI * hz * i / rate) * 8000));
        }

        public void Dispose()
        {
            Window.Close();
            Theme.Dispose();
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(Root)) Directory.Delete(Root, recursive: true); } catch { }
        }
    }
}
