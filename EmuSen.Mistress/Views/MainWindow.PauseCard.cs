using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text.Json;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // The in-game menu as the Pause Card on LunaP's Card form: its header, help bar and context pane, the volume row and the Guide chords - see EmuSen_BigPicture.md §48.
    public partial class MainWindow
    {
        private readonly PauseCardPane _pauseCardPane = new();
        private string? _paneShowing;
        private readonly Dictionary<string, (DateTime When, Bitmap? Picture)> _slotPictures = new(StringComparer.Ordinal);

        // The card shows over a game in a big-screen session; the library's menu keeps its centred look.
        internal bool PauseCardShown => _bigScreen && _padMenuInGame;

        private void SetUpPauseCard()
        {
            PadCard.Title = "Pause";
            PadCard.Detail = _pauseCardPane;
        }

        // The header names the page with the game's cover before it; a submenu's subtitle is the way back, a question's its sentence.
        private void ShowPauseCard(PadMenuPage page, bool nested)
        {
            MenuPanel card = PadCard.CardPanel;
            string? under = page.Question ? page.Detail : nested ? "‹ " + string.Join(" › ", _padMenuPages.SkipLast(1).Select(p => PageTitle(p))) : null;
            card.SubtitleLetterCase = LunaP.Media.LetterCase.None;
            card.HintFamily = HelpFamily;
            PadCard.Header = new AdaptiveHeader(PageTitle(page), under is null ? null : FitSubtitle(card, under), RunningGameCover());
            PadCard.SwapsAcceptAndBack = _appSettings.SwapPadButtons;
            var actions = new List<LunaAction>
            {
                new("Close Menu", ClosePadMenu) { PadButton = PadGlyphButton.Start },
                new("Screenshot", TakeScreenshot) { PadButton = PadHints.Glyph(PadGlyphButton.West) },
            };
            if (_padMenuEntries.Any(e => e.Adjust is not null)) actions.Add(new LunaAction("Change", () => { }) { PadButton = PadGlyphButton.DPadLeftRight });
            PadCard.SetActions(actions);
        }

        private Bitmap? RunningGameCover() =>
            _currentRomPath is string path && CoverPathFor(new EmuSen.Mistress.Library.RomEntry(path)) is string cover ? _covers.Get(cover) : null;

        // The pane follows the chosen row: the slots on Save and Load, the reel on Rewind, the paused frame on the library's Back row; read again only when the row or the slot changes.
        private void UpdatePauseCardPane()
        {
            int at = PadMenuList.SelectedIndex;
            PadMenuEntry? entry = _padMenuOpen && at >= 0 && at < _padMenuEntries.Count ? _padMenuEntries[at] : null;
            string key = entry is null ? "none" : entry.ShowsSlot ? $"slots {_stateSlot}" : entry.ShowsReel ? "reel" : entry.ShowsFrame ? "frame" : "none";
            if (key == _paneShowing) return;
            _paneShowing = key;
            if (!PauseCardShown) ShowLibraryFramePane(key == "frame");
            if (PauseCardShown && key.StartsWith("slots", StringComparison.Ordinal)) _pauseCardPane.ShowSlots(SlotStrip(), _stateSlot);
            else if (PauseCardShown && key == "reel") ShowReelPane();
            else _pauseCardPane.Clear();
        }

        private List<(string Name, string When, IImage? Picture)> SlotStrip()
        {
            var slots = new List<(string, string, IImage?)>();
            for (int slot = 1; slot <= StateSlots; slot++)
            {
                string? state = StatePathForSlot(slot);
                bool saved = state is not null && File.Exists(state);
                string when = saved ? File.GetLastWriteTime(state!).ToString("ddd d MMM, HH:mm", CultureInfo.CurrentCulture) : "Empty";
                slots.Add(($"Slot {slot}", when, saved ? SlotPictureOf(SaveLibrary.PicturePathFor(state!)) : null));
            }
            return slots;
        }

        // A slot's picture, read again only when its file changes.
        private Bitmap? SlotPictureOf(string path)
        {
            DateTime when = File.Exists(path) ? File.GetLastWriteTimeUtc(path) : default;
            if (_slotPictures.TryGetValue(path, out var known) && known.When == when) return known.Picture;
            Bitmap? picture = null;
            try { if (when != default) picture = new Bitmap(path); }
            catch (Exception) { picture = null; }
            _slotPictures[path] = (when, picture);
            return picture;
        }

        // The reel's three newest pictures and how many seconds back it reaches, or why it cannot open.
        private void ShowReelPane()
        {
            if (RewindUnavailable() is string why || _session is not { } session)
            {
                _pauseCardPane.ShowReel([], $"Rewind: {RewindUnavailable() ?? "no game running"}");
                return;
            }
            var pictures = new List<IImage>();
            double seconds = 0;
            try
            {
                using JsonDocument reel = JsonDocument.Parse(session.Crystal.RewindReel());
                List<JsonElement> moments = reel.RootElement.GetProperty("moments").EnumerateArray().ToList();
                if (moments.Count > 0 && session.FrameRateHz > 0)
                    seconds = (moments[^1].GetProperty("frame").GetInt64() - moments[0].GetProperty("frame").GetInt64()) / session.FrameRateHz;
                foreach (JsonElement m in moments.Where(m => m.GetProperty("picture").ValueKind == JsonValueKind.Object).TakeLast(3))
                    if (Picture(session.Crystal.RewindPicture((ulong)m.GetProperty("frame").GetInt64())) is { } p) pictures.Add(p);
            }
            catch (CrystalException) { }
            _pauseCardPane.ShowReel(pictures, seconds >= 1 ? $"{Math.Floor(seconds):0} seconds to go back through" : "Under a second to go back through");
        }

        // A packed picture as the runtime hands it, width and height then RGBA rows.
        private static WriteableBitmap? Picture(byte[] packed)
        {
            if (packed.Length < 8) return null;
            int width = (int)BitConverter.ToUInt32(packed, 0), height = (int)BitConverter.ToUInt32(packed, 4);
            return width > 0 && height > 0 && packed.Length >= 8 + width * height * 4 ? Picture(packed.AsSpan(8), width, height) : null;
        }

        private static WriteableBitmap Picture(ReadOnlySpan<byte> rgba, int width, int height)
        {
            var bitmap = new WriteableBitmap(new PixelSize(width, height), new Vector(96, 96), PixelFormat.Rgba8888, AlphaFormat.Opaque);
            using ILockedFramebuffer target = bitmap.Lock();
            for (int y = 0; y < height; y++)
                Marshal.Copy(rgba.Slice(y * width * 4, width * 4).ToArray(), 0, target.Address + y * target.RowBytes, width * 4);
            return bitmap;
        }

        // The library's menu shows the paused frame beside its Back row, where the screen leaves room for it.
        private void ShowLibraryFramePane(bool show)
        {
            if (!show || _session is not { } session)
            {
                PadMenuBig.Pane = null;
                return;
            }
            Bitmap? frame = null;
            try { if (CurrentPicture(session) is { } still) frame = Picture(still.Rgba, still.Width, still.Height); }
            catch (CrystalException) { frame = null; }
            var pane = new PauseCardPane();
            double u = MenuPanel.GetScale(this) is > 0 and var s ? s : 1;
            MenuPanel.SetScale(pane, u);
            pane.ShowPicture(frame, "Paused", 140 * u);
            PadMenuBig.Pane = pane;
        }

        // On the desktop the menu owns the volume; under Steam's Game Mode the device's quick menu does, and the row says so (§48.5).
        private PadMenuEntry VolumeEntry()
        {
            if (_bigScreenForced)
                return new(() => $"{VolumeRow}      Quick Access Menu", () => Notify("The volume is set in the device's quick access menu"), closes: false) { Label = () => VolumeRow, Value = () => "Quick Access Menu" };
            return new(() => $"{VolumeRow}      <  {VolumePercent()}  >", () => { }, StepVolume, closes: false) { Label = () => VolumeRow, Value = VolumePercent };
        }

        private string VolumePercent() => $"{Math.Round(Math.Clamp(_appSettings.Volume, 0, 1) * 100):0}%";

        private void StepVolume(int by)
        {
            double next = Math.Clamp(Math.Round(_appSettings.Volume * 10 + by) / 10, 0, 1);
            if (_hudVolume is not null) _hudVolume.Value = next;
            else SetVolume(next);
        }

        private readonly GuideChordReader _guideChords = new();

        // Guide-held chords where they do not clash with Steam's own: Auto turns them off in Steam's Game Mode (§48.7).
        internal bool GuideChordsOn => _appSettings.GuideChords switch
        {
            AppSettings.GuideChordsOn => true,
            AppSettings.GuideChordsOff => false,
            _ => !_bigScreenForced,
        };

        // Over a running game with the chords on: Guide alone opens the menu as it is let go, and with Up, Down or R1 saves, loads or takes a screenshot instead; true while Guide is the chord's.
        private bool FeedGuideChords()
        {
            if (!GuideChordsOn) return false;
            bool guide = PadHeld(UiButton.Guide);
            GuideChord chord = _guideChords.Feed(guide, PadHeld(UiButton.Up), PadHeld(UiButton.Down), PadHeld(UiButton.PageDown));
            if (guide) _padQuiet = true;
            switch (chord)
            {
                case GuideChord.Save: SaveState(); break;
                case GuideChord.Load: LoadState(); break;
                case GuideChord.Screenshot: TakeScreenshot(); break;
                case GuideChord.Menu: OpenPadMenu(); return true;
            }
            return guide;
        }
    }
}
