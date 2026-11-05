using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Threading;
using EmuSen.Endymion;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Media;
using EmuSen.LunaP.Theme;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.Views
{
    // Moonrise over big picture's start, held until the window's start-up work is done, skipped by any press - see EmuSen_Interface.md §22.
    public partial class MainWindow
    {
        private LunaSplash? _splash;
        private TimeSpan _splashFrom;
        private bool _splashPlayed, _splashFrameAsked, _splashSkipped, _bellsRung;
        private IReadOnlyList<double>? _bellTimes;
        private Task<byte[]>? _bells;
        private Task? _startupWork;
        private readonly TaskCompletionSource _artworkReady = new(TaskCreationOptions.RunContinuationsAsynchronously);

        // This window's switch, from StartupSplash.ShownByDefault; a test about the splash sets it.
        internal bool StartupSplashes { get; set; } = StartupSplash.ShownByDefault;

        // What the splash waits for beside its own length: the artwork index read; a test gives slower work.
        internal Task StartupWork
        {
            get => _startupWork ?? _artworkReady.Task;
            set => _startupWork = value;
        }

        internal LunaSplash? SplashShown => _splash;

        internal bool SplashOpen => _splash is not null;

        // Once a window, when it opens in big picture with the setting on, and not under a game started by --play.
        private void OpenStartupSplash()
        {
            if (_splashPlayed || !_bigScreen || !_appSettings.StartupSplash || !StartupSplashes || PlayArgument(Environment.GetCommandLineArgs()) is not null) return;
            if (Sheets.Parent is not Panel root) return;
            _splashPlayed = true;
            IReadOnlyList<SplashBody> bodies = StartupSplash.Bodies(RuntimeCatalog.Systems);
            _splash = new LunaSplash { Name = "StartupSplash", Bodies = bodies, Title = StartupSplash.Title, Caption = StartupSplash.Caption(bodies.Count) };
            root.Children.Add(_splash);
            _splashFrom = UiClock();
            if (LunaMotion.Reduced) _splash.Finish();
            StartupWork.ContinueWith(_ => Dispatcher.UIThread.Post(AdvanceSplash), CancellationToken.None, TaskContinuationOptions.ExecuteSynchronously, TaskScheduler.Default);
            ScheduleSplash();
        }

        // Steps the splash to the interface clock, rings the bells when due, and takes it down once it is finished and start-up is done.
        internal void AdvanceSplash()
        {
            if (_splash is not { } splash) return;
            TimeSpan elapsed = UiClock() - _splashFrom;
            if (!splash.AtEnd) splash.Time = elapsed.TotalSeconds;
            StepBells(splash, elapsed.TotalSeconds);
            if (SplashFinished(splash, elapsed) && StartupWork.IsCompleted)
            {
                CloseSplash();
                return;
            }
            ScheduleSplash();
        }

        // A frame each refresh while it moves or its bells are due; a finished splash held for start-up asks for none, its work's end steps it.
        private void ScheduleSplash()
        {
            if (_splash is not { } splash || _splashFrameAsked) return;
            if (SplashFinished(splash, UiClock() - _splashFrom) && !BellsPending(splash)) return;
            _splashFrameAsked = true;
            RequestAnimationFrame(_ => { _splashFrameAsked = false; AdvanceSplash(); });
        }

        // Any pad button, key or click: the finished frame at once, and the splash gone as soon as start-up is done.
        private void SkipSplash()
        {
            if (_splash is not { } splash) return;
            _splashSkipped = true;
            splash.Finish();
            AdvanceSplash();
        }

        // A key held the splash takes; true while it is open, so nothing behind it hears the press.
        private bool SplashKey(Avalonia.Input.Key key, bool pressed)
        {
            if (_splash is null) return false;
            if (pressed && key is not (Avalonia.Input.Key.Tab or Avalonia.Input.Key.LeftShift or Avalonia.Input.Key.RightShift)) SkipSplash();
            return true;
        }

        // The pad while the splash shows: a press skips it, and the press does nothing more once it is gone.
        private void SplashPad()
        {
            if (Enum.GetValues<UiButton>().Any(PadHeld)) SkipSplash();
            else AdvanceSplash();
            _padNavigator.Forget(PadHeld);
        }

        private void CloseSplash()
        {
            if (_splash is not { } splash) return;
            _splash = null;
            (splash.Parent as Panel)?.Children.Remove(splash);
        }

        // Its whole length shown, the still frame under reduced motion included, or skipped.
        private bool SplashFinished(LunaSplash splash, TimeSpan elapsed) => _splashSkipped || elapsed.TotalSeconds >= splash.Timeline.Length;

        private bool BellsPending(LunaSplash splash) =>
            !_bellsRung && !_splashSkipped && _appSettings.NavigationSounds && UiSoundGain > 0 && splash.Bodies.Count > 0;

        // One bell per planet as the sweep reaches it, made for this count and this screen's row, through the navigation sounds' stream and volume.
        private void StepBells(LunaSplash splash, double seconds)
        {
            if (!BellsPending(splash)) return;
            if (_bellTimes is null && seconds >= 1.0 && splash.Bounds.Height > 0)
            {
                _bellTimes = splash.StrikeTimes();
                SplashChime chime = splash.Timeline.Chime;
                IReadOnlyList<double> strikes = _bellTimes;
                if (UiSoundSink is null) _bells = Task.Run(() => LunaSounds.Render(chime, strikes));
            }
            if (_bellTimes is not { Count: > 0 } times || seconds < times[0]) return;
            if (UiSoundSink is { } sink)
            {
                _bellsRung = true;
                sink(StartupSplash.BellsKey);
                return;
            }
            if (_bells is not { IsCompletedSuccessfully: true } made) return;
            _bellsRung = true;
            UiSoundPlayer player = UiSounds();
            player.Volume = UiSoundGain;
            player.Remember(StartupSplash.BellsKey, made.Result);
            player.Play(StartupSplash.BellsKey);
        }
    }
}
