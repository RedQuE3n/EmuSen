using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // The rewind reel: pictures kept with the snapshots, and a moment chosen from them - see EmuSen_Settings_Reference.md §4.49.
    public partial class MainWindow
    {
        // Emulation thread only: the last picture made for a snapshot, and the frame serial it was made from.
        private RewindThumbnail? _lastThumbnail;
        private long? _lastThumbnailSerial;

        // Why the reel cannot open now, or null when it can.
        private string? RewindUnavailable()
        {
            if (_session?.Core is EmuSen.Cores.Nintendo.Mars.MarsCore) return "not kept for Mars (C#)";
            if (_rewind.SnapshotBytes == 0) return "nothing to go back to yet";
            return null;
        }

        private string RewindMenuText() => RewindUnavailable() is string why ? $"Rewind  ({why})" : "Rewind";

        // Stays in the menu when there is nothing to show; otherwise hands the menu's pause to the reel - see §4.49.
        private void RewindFromPadMenu()
        {
            if (RewindUnavailable() is not null) return;
            bool resume = _padMenuPaused;
            _padMenuPaused = false;
            ClosePadMenu();
            OpenRewindReel(resume);
        }

        // Paused first, then the moments read on the thread that owns the buffer - see §4.49.
        private void OpenRewindReel(bool resumeAfter)
        {
            if (_session is not { IsRomLoaded: true } || RewindUnavailable() is string why)
            {
                string status = $"Rewind: {RewindUnavailable() ?? "no game running"}";
                StatusText.Text = status;
                Notify(status);
                if (resumeAfter) ResumeEmulation();
                return;
            }

            if (!IsPaused)
            {
                PauseEmulation();
                resumeAfter = true;
            }

            double hz = _session.FrameRateHz;
            RequestOnEmulationThread(session =>
            {
                IReadOnlyList<RewindMoment> moments = _rewind.Moments();
                List<ReelMoment> reel = ReelMoment.From(moments, _rewind.Frame, hz, PresentPicture(session));
                Dispatcher.UIThread.Post(() => _ = ShowRewindReelAsync(reel, resumeAfter));
            });
        }

        private RewindThumbnail? PresentPicture(EmulatorSession session)
        {
            byte[] rgba = session.GetFrameBufferRgba();
            RewindThumbnail? picture = RewindThumbnail.From(rgba, session.ScreenWidth, session.ScreenHeight, session.RowRepeat, _rewind.ThumbnailWidth);
            (session.Core as IFrameBufferPool)?.ReturnFrameBuffer(rgba);
            return picture;
        }

        private async Task ShowRewindReelAsync(List<ReelMoment> reel, bool resumeAfter)
        {
            if (!reel.Any(m => !m.IsNow))
            {
                StatusText.Text = "Rewind: nothing to go back to yet";
                Notify(StatusText.Text);
                if (resumeAfter && GameOnScreen) ResumeEmulation();
                return;
            }

            // On a sheet the footer names the reel's buttons, not the ones every other sheet has.
            string? footer = Sheets.Hint;
            Sheets.Hint = RewindReelWindow.PadHint;
            ReelMoment? chosen;
            try
            {
                chosen = await SheetLayer.ShowDialog<ReelMoment>(new RewindReelWindow(reel) { ShowHint = !Sheets.PresentsWindows }, this);
            }
            finally
            {
                Sheets.Hint = footer;
            }

            if (chosen is null)
            {
                if (resumeAfter && GameOnScreen) ResumeEmulation();
                return;
            }

            long frame = chosen.Frame;
            string label = chosen.Label;
            RequestOnEmulationThread(session =>
            {
                bool rewound = session.Core is { } core && _rewind.RewindTo(core, frame);
                if (rewound)
                {
                    _lastThumbnail = null;
                    _debugTarget?.RefreshProviders();
                    session.DequeueAudioSamples(int.MaxValue);
                    _audioPlayer.RateControl.Reset();
                    SubmitFrame(session.GetFrameBufferRgba(), session.ScreenWidth, session.ScreenHeight, session.RowRepeat, _frames.ReleaseFor(session.Core));
                }

                Dispatcher.UIThread.Post(() =>
                {
                    if (resumeAfter && GameOnScreen) ResumeEmulation();
                    StatusText.Text = rewound ? $"Rewound to {label}" : "Rewind: that moment is no longer held";
                    Notify(rewound ? "Rewound" : StatusText.Text);
                });
            });
        }

        // A snapshot was just taken and this frame is being shown: its picture, downscaled - see §4.49.
        private void Picture(EmulatorSession session, byte[] frame, long? serial)
        {
            if (_rewind.AttachThumbnail(frame, session.ScreenWidth, session.ScreenHeight, session.RowRepeat) is not { } made) return;
            _lastThumbnail = made;
            _lastThumbnailSerial = serial;
        }

        // A snapshot on a frame whose picture was not offered again: the last picture if it is that one, else the frame fetched for it.
        private void PictureUnchanged(EmulatorSession session, long? serial)
        {
            if (serial is not null && serial == _lastThumbnailSerial && _lastThumbnail is { } same)
            {
                _rewind.AttachThumbnail(same);
                return;
            }

            byte[] frame = session.GetFrameBufferRgba();
            Picture(session, frame, serial);
            (session.Core as IFrameBufferPool)?.ReturnFrameBuffer(frame);
        }
    }
}
