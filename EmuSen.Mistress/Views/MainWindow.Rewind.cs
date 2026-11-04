using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Media;
using Avalonia.Threading;
using EmuSen.Crystal.Client;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // The rewind reel as a page of the Pause Card: the pictures the runtime keeps with its snapshots, and a moment chosen from them - see EmuSen_Interface.md §18.5 and EmuSen_Backend.md §14.13.
    public partial class MainWindow
    {
        public const string GoBackRow = "Go back", MomentRow = "Moment", HoldRow = "Hold ◂ to scrub", ReelPage = "Rewind";
        public const double StrideSeconds = 5;

        // One moment the reel holds, newest first from the present: its frame and how far back it is.
        public sealed record ReelMark(long Frame, double SecondsBack, bool IsNow)
        {
            public string Label => IsNow ? "Now" : Ago(SecondsBack);

            // How a moment so many seconds back is named, as the Rewind row's pane names the newest ones.
            public static string Ago(double seconds) => MainWindow.Ago(seconds);
            public string Short => IsNow ? "Now" : SecondsBack < 10 ? $"{SecondsBack:0.#} s" : $"{Math.Floor(SecondsBack):0} s";
        }

        private List<ReelMark> _reel = new();
        private int _reelAt;
        private readonly Dictionary<long, IImage?> _reelPictures = new();

        public IReadOnlyList<ReelMark> ReelShown => _reel;
        public ReelMark? ReelFocused => _reelAt >= 0 && _reelAt < _reel.Count ? _reel[_reelAt] : null;

        // Why the reel cannot open now, or null when it can.
        private string? RewindUnavailable()
        {
            if (_session is not { } session) return "no game running";
            try
            {
                if (session.RewindWithheldNow() is string withheld) return withheld;
                using JsonDocument reel = JsonDocument.Parse(session.Crystal.RewindReel());
                if (reel.RootElement.GetProperty("snapshot_bytes").GetInt64() == 0) return "nothing to go back to yet";
            }
            catch (CrystalException) { return "no game running"; }
            return null;
        }

        private string RewindMenuText() => RewindUnavailable() is string why ? $"Rewind  ({why})" : "Rewind";

        // The reel as a page of the card, on its first stop after the present: the rows to go back, step and scrub - see EmuSen_Interface.md §18.5.
        private void OpenReelPage()
        {
            if (RewindUnavailable() is not null || _session is not { } session) return;
            try { _reel = ReelOf(session); }
            catch (CrystalException) { _reel = new(); }
            _reelPictures.Clear();
            if (_reel.Count < 2) return;
            _reelStops = StopsOf(_reel);
            _reelAt = _reelStops[1];
            EnterPadMenuPage(new PadMenuPage(ReelPage, ReelRows)
            {
                AcceptWord = "Go back", SideWord = "Moment", StartResumes = false, Sideways = StepReel, Pad = ReelPad,
            }, Math.Max(PadMenuList.SelectedIndex, 0), key: null);
        }

        // The menu bar's Rewind: the Pause Card opened on its reel page.
        private void OpenRewindReel()
        {
            if (!GameOnScreen) return;
            if (!_padMenuOpen) OpenPadMenu();
            if (_padMenuPages.Count != 1 || !_padMenuInGame) return;
            PadMenuList.SelectedIndex = Math.Max(0, _padMenuEntries.FindIndex(e => e.ShowsReel));
            OpenReelPage();
            if (_padMenuPages.Count == 1)
            {
                string status = $"Rewind: {RewindUnavailable() ?? "nothing to go back to yet"}";
                StatusText.Text = status;
                Notify(status);
            }
        }

        private List<PadMenuEntry> ReelRows() =>
        [
            new(() => ReelFocused is { IsNow: true } ? "Stay at Now" : $"{GoBackRow} to {ReelFocused?.Label}", GoBackToMoment, closes: false) { ShowsMoment = true },
            new(() => $"{MomentRow}      <  {ReelFocused?.Label}  >", () => { }, StepReel, closes: false) { Label = () => MomentRow, Value = () => ReelFocused?.Label, ShowsMoment = true },
            new(() => HoldRow, () => { }, closes: false) { Dimmed = true, ShowsMoment = true },
        ];

        public static readonly double[] StopSeconds = [1.5, 4, 8, 12];

        private List<int> _reelStops = new();

        // The strip's stops, as indexes of the reel: the present, the held moments nearest 1.5, 4, 8 and 12 seconds back, and the oldest.
        public IReadOnlyList<int> ReelStops => _reelStops;

        private static List<int> StopsOf(List<ReelMark> reel)
        {
            var stops = new List<int> { 0 };
            foreach (double seconds in StopSeconds)
            {
                if (seconds >= reel[^1].SecondsBack) break;
                int nearest = Enumerable.Range(1, reel.Count - 1).MinBy(i => Math.Abs(reel[i].SecondsBack - seconds));
                if (nearest > stops[^1]) stops.Add(nearest);
            }
            if (reel.Count - 1 > stops[^1]) stops.Add(reel.Count - 1);
            return stops;
        }

        // A press of Left or Right goes to the next stop that way, Right further back; held, it scrubs one held moment, half a second, at a time.
        private void StepReel(int by)
        {
            if (_reel.Count == 0) return;
            if (_padNavigator.IsRepeat(by > 0 ? UiButton.Right : UiButton.Left)) _reelAt = Math.Clamp(_reelAt + by, 0, _reel.Count - 1);
            else if (by > 0) _reelAt = _reelStops.FirstOrDefault(i => i > _reelAt, _reel.Count - 1);
            else _reelAt = _reelStops.LastOrDefault(i => i < _reelAt, 0);
        }

        // The shoulders stride five seconds and the triggers go to either end, as the reel always has.
        private bool ReelPad(UiButton button)
        {
            switch (button)
            {
                case UiButton.PageUp: Stride(1); break;
                case UiButton.PageDown: Stride(-1); break;
                case UiButton.First: _reelAt = _reel.Count - 1; break;
                case UiButton.Last: _reelAt = 0; break;
                default: return false;
            }
            RefreshPadMenuText(Math.Max(PadMenuList.SelectedIndex, 0));
            return true;
        }

        // The nearest moment at least five seconds further in that direction, else the end.
        private void Stride(int back)
        {
            if (_reel.Count == 0) return;
            double from = _reel[_reelAt].SecondsBack;
            int target = back > 0 ? _reel.Count - 1 : 0;
            for (int i = _reelAt + back; i >= 0 && i < _reel.Count; i += back)
                if (back > 0 ? _reel[i].SecondsBack >= from + StrideSeconds : _reel[i].SecondsBack <= from - StrideSeconds) { target = i; break; }
            _reelAt = target;
        }

        // A closes the card and returns the game to the moment, held paused until it is in place; the present is no rewind, so A there only closes the card.
        private void GoBackToMoment()
        {
            if (ReelFocused is not { IsNow: false } moment || _session is not { } session)
            {
                ClosePadMenu();
                return;
            }
            bool resume = _padMenuPaused;
            _padMenuPaused = false;
            ClosePadMenu();
            string label = moment.Label;
            long frame = moment.Frame;
            Task.Run(() =>
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
                    if (!ReferenceEquals(_session, session)) return;
                    if (resume && GameOnScreen) ResumeEmulation();
                    StatusText.Text = rewound ? $"Rewound to {label}" : "Rewind: that moment is no longer held";
                    Notify(rewound ? "Rewound" : StatusText.Text);
                });
            });
        }

        // The held moments with pictures, newest first, after the present.
        private static List<ReelMark> ReelOf(GameSession session)
        {
            double hz = session.FrameRateHz > 0 ? session.FrameRateHz : 60;
            using JsonDocument reel = JsonDocument.Parse(session.Crystal.RewindReel());
            using JsonDocument info = JsonDocument.Parse(session.Crystal.Info());
            long now = info.RootElement.GetProperty("total_frames").GetInt64();
            var held = new List<long>();
            foreach (JsonElement m in reel.RootElement.GetProperty("moments").EnumerateArray())
            {
                long frame = m.GetProperty("frame").GetInt64();
                now = Math.Max(now, frame);
                if (m.GetProperty("picture").ValueKind == JsonValueKind.Object) held.Add(frame);
            }
            var marks = new List<ReelMark> { new(now, 0, IsNow: true) };
            marks.AddRange(held.Where(f => f < now).OrderByDescending(f => f).Select(f => new ReelMark(f, (now - f) / hz, IsNow: false)));
            return marks;
        }

        // A moment's picture, read from the runtime once while the page is open.
        private IImage? ReelPicture(ReelMark mark)
        {
            long key = mark.Frame * 2 + (mark.IsNow ? 1 : 0);
            if (_reelPictures.TryGetValue(key, out IImage? known)) return known;
            IImage? picture = null;
            if (_session is { } session)
            {
                try
                {
                    picture = mark.IsNow
                        ? CurrentPicture(session) is { } now ? Picture(now.Rgba, now.Width, now.Height) : null
                        : Picture(session.Crystal.RewindPicture((ulong)mark.Frame));
                }
                catch (CrystalException) { picture = null; }
            }
            _reelPictures[key] = picture;
            return picture;
        }

        // The focused moment large, the stops in the strip, and what A does.
        private void ShowMomentPane()
        {
            if (ReelFocused is not { } focused)
            {
                _pauseCardPane.Clear();
                return;
            }
            // The stops, with a scrubbed moment between two of them standing in for the nearer.
            List<int> shown = _reelStops.ToList();
            if (!shown.Contains(_reelAt)) shown[Enumerable.Range(1, shown.Count - 1).MinBy(i => Math.Abs(shown[i] - _reelAt))] = _reelAt;
            shown.Sort();
            List<(IImage? Picture, string Label)> strip = shown.Select(i => (ReelPicture(_reel[i]), _reel[i].Short)).ToList();
            string sentence = focused.IsNow ? "Now, where the game is paused. A closes the card and plays on from here." : $"{focused.Label}. A goes back there; the moments after it are kept until you play on.";
            _pauseCardPane.ShowMoment(ReelPicture(focused), strip, shown.IndexOf(_reelAt), sentence);
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
