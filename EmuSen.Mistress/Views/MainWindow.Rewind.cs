using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Threading;
using EmuSen.Crystal.Client;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // The rewind reel: the pictures the runtime keeps with its snapshots, and a moment chosen from them - see EmuSen_Settings_Reference.md §4.49 and EmuSen_Backend.md §14.13.
    public partial class MainWindow
    {
        // Why the reel cannot open now, or null when it can.
        private string? RewindUnavailable()
        {
            if (_session is not { } session) return "no game running";
            if (!session.RewindCapture) return session.RewindWithheld ?? "not kept for this engine";
            try
            {
                using JsonDocument reel = JsonDocument.Parse(session.Crystal.RewindReel());
                if (reel.RootElement.GetProperty("snapshot_bytes").GetInt64() == 0) return "nothing to go back to yet";
            }
            catch (CrystalException) { return "no game running"; }
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

        // Paused first, then the moments and their pictures read from the runtime off the UI thread - see §4.49.
        private void OpenRewindReel(bool resumeAfter)
        {
            if (_session is not { } session || RewindUnavailable() is string why)
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

            double hz = session.FrameRateHz;
            Task.Run(() =>
            {
                List<ReelMoment> reel;
                try { reel = ReelOf(session, hz); }
                catch (CrystalException) { reel = new List<ReelMoment>(); }
                Dispatcher.UIThread.Post(() => _ = ShowRewindReelAsync(reel, resumeAfter));
            });
        }

        // The held moments with pictures, oldest first, then the picture on show as the present.
        private static List<ReelMoment> ReelOf(GameSession session, double hz)
        {
            using JsonDocument reel = JsonDocument.Parse(session.Crystal.RewindReel());
            var moments = new List<(long, byte[], int, int)>();
            long now = 0;
            foreach (JsonElement m in reel.RootElement.GetProperty("moments").EnumerateArray())
            {
                long frame = m.GetProperty("frame").GetInt64();
                now = Math.Max(now, frame);
                if (m.GetProperty("picture").ValueKind != JsonValueKind.Object) continue;
                byte[] packed = session.Crystal.RewindPicture((ulong)frame);
                int width = (int)BitConverter.ToUInt32(packed, 0), height = (int)BitConverter.ToUInt32(packed, 4);
                moments.Add((frame, packed[8..], width, height));
            }
            using JsonDocument info = JsonDocument.Parse(session.Crystal.Info());
            now = Math.Max(now, info.RootElement.GetProperty("total_frames").GetInt64());
            return ReelMoment.From(moments, now, hz, CurrentPicture(session));
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
            Sheets.Hint = EmuSen.Mistress.Input.PadHints.Face(RewindReelWindow.PadHint);
            ReelMoment? chosen;
            try
            {
                chosen = await SheetLayer.ShowDialog<ReelMoment>(new RewindReelWindow(reel) { ShowHint = !Sheets.PresentsWindows }, this);
            }
            finally
            {
                Sheets.Hint = footer;
            }

            if (chosen is null || _session is not { } session)
            {
                if (resumeAfter && GameOnScreen) ResumeEmulation();
                return;
            }

            long frame = chosen.Frame;
            string label = chosen.Label;
            _ = Task.Run(() =>
            {
                bool rewound;
                try
                {
                    session.Crystal.RewindTo((ulong)frame);
                    _audioPlayer.RateControl.Reset();
                    OfferStill(session);
                    rewound = true;
                }
                catch (CrystalException) { rewound = false; }

                Dispatcher.UIThread.Post(() =>
                {
                    if (resumeAfter && GameOnScreen) ResumeEmulation();
                    StatusText.Text = rewound ? $"Rewound to {label}" : "Rewind: that moment is no longer held";
                    Notify(rewound ? "Rewound" : StatusText.Text);
                });
            });
        }

        // The picture on show, its repeated rows written out; null where none is drawn.
        private static (byte[] Rgba, int Width, int Height)? CurrentPicture(GameSession session)
        {
            CrystalSession crystal = session.Crystal;
            using JsonDocument shown = JsonDocument.Parse(crystal.Info());
            JsonElement picture = shown.RootElement.GetProperty("picture");
            int width = picture.GetProperty("width").GetInt32(), height = picture.GetProperty("height").GetInt32(), repeat = Math.Max(1, picture.GetProperty("row_repeat").GetInt32());
            var rgba = new byte[crystal.PictureBytes];
            crystal.CopyPicture(rgba);
            if (width <= 0 || height <= 0 || rgba.Length < width * height * 4) return null;
            if (repeat == 1) return (rgba, width, height);
            var tall = new byte[width * height * repeat * 4];
            int row = width * 4;
            for (int y = 0; y < height; y++)
                for (int r = 0; r < repeat; r++)
                    Buffer.BlockCopy(rgba, y * row, tall, (y * repeat + r) * row, row);
            return (tall, width, height * repeat);
        }

        // A paused machine's picture to the screen after it moved, as a frame would offer it.
        private void OfferStill(GameSession session)
        {
            if (CurrentPicture(session) is not { } still || _frames.Rent(still.Rgba.Length) is not { } pixels) return;
            still.Rgba.CopyTo(pixels, 0);
            _frames.Offer(pixels, still.Width, still.Height, 1);
        }
    }
}
