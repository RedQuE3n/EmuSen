using System;
using System.Diagnostics;
using System.IO;
using System.Threading.Tasks;
using EmuSen.Common;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
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

        // Every way a game is started from outside it: firmware, then the resume question, then the load.
        private async Task StartGameAsync(string path, string displayName)
        {
            await PromptForMissingFirmwareAsync(path);
            if (await ChooseResumeAsync(path) is not { } choice) return;
            LoadGame(path, displayName, choice == ResumeChoice.Resume ? ResumeStatePath(path) : null, reset: false);
        }

        private string ResumeStatePath(string romPath) => SaveLibrary.ResumeStatePathFor(romPath, _appSettings.StateDirectory);

        // Null cancels the launch; with no resume state there is nothing to ask.
        private async Task<ResumeChoice?> ChooseResumeAsync(string romPath)
        {
            string state = ResumeStatePath(romPath);
            if (!File.Exists(state)) return ResumeChoice.Restart;
            StateRecord? record = StateRecord.Read(state);
            if (Refusal(record, romPath, null) is string refused)
            {
                StatusText.Text = $"{refused} Starting from the beginning.";
                return ResumeChoice.Restart;
            }

            switch (_appSettings.ResumeOnLaunch)
            {
                case AppSettings.ResumeAlways: return await SameGameOrConfirmedAsync(record, romPath) ? ResumeChoice.Resume : ResumeChoice.Restart;
                case AppSettings.ResumeNever: return ResumeChoice.Restart;
            }

            if (_session is { IsRomLoaded: true }) PauseEmulation();
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

        // Only with the emulation thread stopped; a failure is reported, never thrown at a closing window.
        private void WriteResumeState()
        {
            if (_session is not { IsRomLoaded: true } session || _currentRomPath is not string rom) return;
            string path = ResumeStatePath(rom);
            try
            {
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                session.SaveState(path);
                WriteStatePicture(session, path);
                WriteStateRecord(session, path, rom);
            }
            catch (Exception ex)
            {
                StatusText.Text = $"Could not save where you left off: {ex.Message}";
            }
        }

        // On the thread that owns the core, so the picture is the frame the state holds - see EmuSen_Galaxia.md §5.2.
        private static void WriteStatePicture(EmulatorSession session, string statePath) =>
            WritePicture(session, SaveLibrary.PicturePathFor(statePath));

        private static void WritePicture(EmulatorSession session, string picturePath)
        {
            byte[] rgba = session.GetFrameBufferRgba();
            int width = session.ScreenWidth, height = session.ScreenHeight, repeat = Math.Max(1, session.RowRepeat);
            if (rgba.Length < width * height * 4) return;
            if (repeat > 1)
            {
                var tall = new byte[width * height * repeat * 4];
                int row = width * 4;
                for (int y = 0; y < height; y++)
                    for (int r = 0; r < repeat; r++)
                        Buffer.BlockCopy(rgba, y * row, tall, (y * repeat + r) * row, row);
                rgba = tall;
                height *= repeat;
            }
            EmuSen.Common.Imaging.PngFile.Write(picturePath, rgba, width, height);
        }

        // Null when the state loaded; otherwise why it did not, and the caller starts the game afresh.
        private static string? TryResume(EmulatorSession session, string statePath, string romPath)
        {
            StateRecord? record = StateRecord.Read(statePath);
            if (Refusal(record, romPath, session) is string refused) return $"Could not resume, started from the beginning: {refused}";
            try
            {
                session.LoadState(statePath);
                return null;
            }
            catch (Exception ex)
            {
                return $"Could not resume, started from the beginning: {ex.Message}{Provenance(record)}";
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

        private void RecordPlayTime()
        {
            if (_currentRomPath is not string path || !_playClock.IsRunning && _playClock.Elapsed == TimeSpan.Zero) return;
            _playClock.Stop();
            _records.Played(path, _playClock.Elapsed);
            _playClock.Reset();
        }
    }
}
