using System;
using System.Diagnostics;
using System.IO;
using System.Threading.Tasks;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Library;

using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // Resume where you left off, and the play records the library reads - see EmuSen_Settings_Reference.md §4.31 and §4.32.
    public partial class MainWindow
    {
        private readonly GameRecords _records = GameRecords.Load();

        // Runs while a game is on screen, stopped for the library and for pauses - see §4.32.
        private readonly Stopwatch _playClock = new();

        // Every way a game is started from outside it: firmware, then the resume question, then from big picture the launch screen (§4.71), then the load.
        private async Task StartGameAsync(string path, string displayName, BigPicture.Scene.SceneGame? bigPicture = null)
        {
            await PrepareGameAsync(path);
            if (await ChooseResumeAsync(path) is not { } choice) return;
            Task<GameSession>? loading = bigPicture is not null ? LoadUnderLaunchScreen(path) : null;
            if (bigPicture is not null && !await ShowLaunchScreenAsync(bigPicture))
            {
                LetGo(loading);
                return;
            }
            if (loading is not null)
            {
                // Never throws: a failed load is rethrown inside LoadGame, which reports it as any failed load.
                await Task.WhenAny(loading);
                // The window closed while the load outlasted the screen: nothing starts in it - see Diamond_Players.md §12.8.3.
                if (!LaunchScreenOpen)
                {
                    LetGo(loading);
                    return;
                }
            }
            LoadGame(path, displayName, choice == ResumeChoice.Resume ? ResumeStatePath(path) : null, reset: false, loading);
            if (bigPicture is null) return;
            bool failed = _currentRomPath != path;
            CloseLaunchScreen();
            if (failed) PadNotice.Show(StatusText.Text ?? $"Failed to load {displayName}");
        }

        // A session opened for a launch that will not happen is closed once its open ends.
        private static void LetGo(Task<GameSession>? loading) =>
            _ = loading?.ContinueWith(t => t.Result.Dispose(), TaskContinuationOptions.OnlyOnRanToCompletion);

        // The game opened on a worker as the launch screen opens, so the screen's time covers the disc's read; null opens it after, as before - see Diamond_Players.md §12.8.3.
        private Task<GameSession>? LoadUnderLaunchScreen(string path)
        {
            if (EarlyLoader is not { } open || LaunchScreen.DurationOf(_appSettings.BigPictureInterface.LaunchScreenDuration) <= TimeSpan.Zero || path == _currentRomPath) return null;
            CrystalOpen what = new() { Path = path };
            return Task.Run(() => open(what));
        }

        // What opens a game under the launch screen; a test gives one it can hold back, or none where its clock is stepped.
        internal Func<CrystalOpen, GameSession>? EarlyLoader { get; set; }

        // A packaged game's resume state is in its package, as the runtime names it - see EmuSen_GamePackages.md §3.5.
        private string ResumeStatePath(string romPath) => ResumeStatePath(romPath, EmuSen.Galaxia.Media.GamePackages.RootFor(romPath));

        private string ResumeStatePath(string romPath, string? package) => SaveLibrary.ResumeStatePathFor(romPath, _appSettings.StateDirectory, package);

        // Null cancels the launch; with no resume state there is nothing to ask.
        private async Task<ResumeChoice?> ChooseResumeAsync(string romPath)
        {
            string state = ResumeStatePath(romPath);
            if (!File.Exists(state)) return ResumeChoice.Restart;
            // Whether the state may load is the runtime's to say when it is loaded; only another copy of the game is asked about here - see EmuSen_Backend.md §14.12.
            StateRecord? record = StateRecordsOnApi.Read(Runtime, state);

            switch (_appSettings.ResumeOnLaunch)
            {
                case AppSettings.ResumeAlways: return await SameGameOrConfirmedAsync(record, romPath) ? ResumeChoice.Resume : ResumeChoice.Restart;
                case AppSettings.ResumeNever: return ResumeChoice.Restart;
            }

            if (_session is not null) PauseEmulation();
            var ask = new ResumeWindow(Path.GetFileNameWithoutExtension(romPath), SaveLibrary.PicturePathFor(state), File.GetLastWriteTime(state));
            ResumeChoice? choice = await SheetLayer.ShowDialog<ResumeChoice?>(ask, this);
            if (choice is { } chosen && ask.Remember)
            {
                _appSettings.ResumeOnLaunch = chosen == ResumeChoice.Resume ? AppSettings.ResumeAlways : AppSettings.ResumeNever;
                _appSettings.Save();
            }
            if (choice == ResumeChoice.Resume && !await SameGameOrConfirmedAsync(record, romPath)) return ResumeChoice.Restart;
            return choice;
        }

        // Only with the emulation thread stopped; the runtime writes the state, its picture and its record. A failure is reported, never thrown at a closing window.
        private void WriteResumeState()
        {
            if (_session is not { } session || _currentRomPath is null) return;
            try { session.Crystal.SaveResume(); }
            catch (CrystalException ex)
            {
                ErrorLog.Error("states", "Could not save where you left off", ex);
                StatusText.Text = $"Could not save where you left off: {ex.Message}";
            }
        }

        // The picture on show as a PNG, the rows the screen repeats written out, as a state's picture is - see EmuSen_Settings_Reference.md §4.34.
        private static void WritePicture(GameSession session, string picturePath)
        {
            if (CurrentPicture(session) is { } shown) EmuSen.Serenity.Imaging.FrameImageWriter.SavePng(shown.Rgba, shown.Width, shown.Height, picturePath);
        }

        // Null when the state loaded; otherwise why it did not, in the runtime's or the core's words, and the caller starts the game afresh.
        private string? TryResume(GameSession session, string path)
        {
            StateRecord? record = StateRecordsOnApi.Read(Runtime, path);
            try
            {
                session.Crystal.LoadState(path);
                return null;
            }
            catch (CrystalException ex)
            {
                ErrorLog.Error("states", "Could not resume", ex, path);
                return $"Could not resume, started from the beginning: {ex.Message}{(ex.Status == EmuSen.Crystal.Client.Contract.CrystalAbi.StateRefused ? "" : Provenance(record))}";
            }
        }

        private void RecordStart(string path)
        {
            _currentRomMd5 = null;
            _currentRomBytes = 0;
            _records.Started(path, DateTime.Now);
            IdentifyLater(path);
            _playClock.Restart();
        }

        // What the play clock read, for a test that cannot wait hours; null reads the clock.
        internal Func<TimeSpan>? PlayClockReading { get; set; }

        private void RecordPlayTime()
        {
            if (_currentRomPath is not string path || !_playClock.IsRunning && _playClock.Elapsed == TimeSpan.Zero) return;
            _playClock.Stop();
            if (PlayTime.Tracked(PlayClockReading?.Invoke() ?? _playClock.Elapsed, _appSettings.MaxPlayTimeTracking) is { } tracked) _records.Played(path, tracked);
            _playClock.Reset();
        }
    }
}
