using System;
using System.Linq;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Motion;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // A view that moves: rebuilt on each change of selection, its time-derived state pushed into the controls at each time value the host steps - see EmuSen_BigPicture.md §14.6.
    public sealed class SceneView
    {
        private Glide _position;
        private TimeSpan _selectedAt;
        private readonly SceneRepeat _repeat;

        public SceneView(SceneData data, string view, TimeSpan now, SceneMotion? motion = null)
        {
            Motion = motion ?? data.Motion;
            ViewName = view;
            Data = data with { Motion = Motion };
            Now = now;
            _selectedAt = now;
            _shownAt = now;
            _position = Glide.At(Index);
            _repeat = new SceneRepeat();
            Root = new Panel { Width = data.Screen.Width, Height = data.Screen.Height };
            Scene = Rebuild();
            Apply();
        }

        public SceneMotion Motion { get; }
        public string ViewName { get; }
        public SceneData Data { get; private set; }
        public TimeSpan Now { get; private set; }

        // The host shows this panel; the scene inside it is replaced on each change of selection.
        public Panel Root { get; }
        public SceneBuilder Scene { get; private set; }

        public ResolvedView View => Data.System.Theme.View(ViewName);

        // The primary element's position now, in items, not wrapped.
        public double Position => _position.ValueAt(Now);

        public TimeSpan SelectedAt => _selectedAt;

        // Where the primary element is going, in items, not wrapped.
        public double Target => _position.To;

        private bool IsSystemView => ViewName == "system";
        private int Count => IsSystemView ? Data.Systems.Count : Data.System.Games.Count;
        private int Index => IsSystemView ? Data.SystemIndex : Data.GameIndex;
        private ResolvedElement? Primary => View.Primary;

        // Whether the primary element slides between items; ES-DE's itemTransitions, and only its carousel and grid have one.
        private bool Slides => Primary is { Type: "carousel" } p && p.String("itemTransitions") != "instant";

        // One step of the selection, as a tap gives it: a carousel and a list both wrap at their ends.
        public void Step(int delta, TimeSpan now) => Move(delta, now, held: false);

        // A move of the selection; a held list stops at its end, where a carousel wraps; false when nothing moved.
        private bool Move(int delta, TimeSpan now, bool held)
        {
            Now = now;
            int count = Count;
            if (count == 0 || delta == 0) return false;
            int target = Index + delta;
            bool wraps = Primary?.Type == "carousel" || !held;
            if (wraps) target = ((target % count) + count) % count;
            else target = Math.Clamp(target, 0, count - 1);
            if (target == Index) return false;
            double to = _position.To + (wraps ? delta : target - Index);
            Data = IsSystemView ? Data with { SystemIndex = target } : Data with { GameIndex = target };
            _position = Slides ? _position.Toward(to, now, Motion.CarouselStep, Motion.CarouselEasing) : Glide.At(to);
            _selectedAt = now;
            Scene = Rebuild();
            Apply();
            return true;
        }

        // A direction held from a time on; the repeats it gives are stepped by Advance.
        public void Press(int direction, TimeSpan now)
        {
            Step(direction, now);
            _repeat.Press(direction, now, RepeatRule);
        }

        public void Release(TimeSpan now)
        {
            Advance(now);
            _repeat.Release();
            FadeMetadataIn(now);
        }

        private void FadeMetadataIn(TimeSpan now)
        {
            if (_metadata.To < 1) _metadata = _metadata.Toward(1, now, Motion.MetadataFadeIn);
        }

        private SceneRepeatRule RepeatRule => Primary?.Type == "carousel" ? (Primary.Bool("fastScrolling") == true ? Motion.CarouselFastRepeat : Motion.CarouselRepeat) : Motion.ListRepeat;

        // Moves the view's clock on: key repeats that fall due, then every time-derived state at the new time.
        public void Advance(TimeSpan now)
        {
            foreach ((int delta, TimeSpan at) in _repeat.Due(now))
            {
                if (IsSystemView) Move(delta, at, held: true);
                else if (Move(delta, at, held: true)) { if (_metadata.To > 0) _metadata = _metadata.Toward(0, at, Motion.MetadataFadeOut); }
                else FadeMetadataIn(at);
            }

            Now = now;
            Apply();
        }

        // Whether anything is still moving, so a host can stop asking for frames.
        public bool IsMoving => !_position.IsSettledAt(Now) || _repeat.Held || !_metadata.IsSettledAt(Now) || (Now < _selectedAt + Motion.ScrollFadeIn && Scene.Entries.Any(e => FadesIn(e.Element))) || Scrolls;

        // A text that may scroll keeps asking for frames: a conservative answer, since its loop never ends.
        private bool Scrolls => Scene.Entries.Any(e => e.Control is FontText { ScrollDirection: not TextScrollDirection.None, Scroll.Speed: > 0 } or TextRowList { Marquee.Speed: > 0 });

        private readonly System.Collections.Generic.Dictionary<Control, double> _opacity = new();

        private SceneBuilder Rebuild()
        {
            SceneBuilder scene = SceneBuilder.Build(View, Data);
            Root.Children.Clear();
            Root.Children.Add(scene.Canvas);
            _opacity.Clear();
            foreach (SceneEntry e in scene.Entries)
                if (e.Control is { } c) _opacity[c] = c.Opacity;
            return scene;
        }

        private Glide _metadata = Glide.At(1);
        private readonly TimeSpan _shownAt;

        // A game's metadata and media, which ES-DE fades out while the list scrolls fast: its fields but the system's names, dates, ratings, badges, media, and whatever the theme marks.
        private bool IsGameMetadata(ResolvedElement e) => !IsSystemView && (e.Bool("metadataElement") == true || e.Type switch
        {
            "text" => e.String("metadata") is { } m && m is not ("systemName" or "systemFullname" or "sourceSystemName" or "sourceSystemFullname"),
            "datetime" or "rating" or "badges" or "video" => true,
            "image" => e.List("imageType").Count > 0,
            _ => false,
        });

        // An element fading in on a change of game (scrollFadeIn), in the gamelist only.
        private bool FadesIn(ResolvedElement e) => !IsSystemView && e.Bool("scrollFadeIn") == true;

        private void Apply()
        {
            double fadeIn = Motion.ScrollFadeIn > TimeSpan.Zero && _selectedAt > _shownAt ? new Glide(Motion.ScrollFadeInFrom, 1, _selectedAt, Motion.ScrollFadeIn).ValueAt(Now) : 1;
            double metadata = _metadata.ValueAt(Now);
            foreach (SceneEntry entry in Scene.Entries)
            {
                if (entry.Control is { } control && _opacity.TryGetValue(control, out double own))
                    control.Opacity = own * (FadesIn(entry.Element) ? fadeIn : 1) * (IsGameMetadata(entry.Element) ? metadata : 1);
                if (entry.Control is ImageCarousel carousel) carousel.Position = _position.ValueAt(Now);
                else if (entry.Control is TextRowList list) list.MarqueeTime = Now - _selectedAt;
                else if (entry.Control is FontText { ScrollDirection: not TextScrollDirection.None } text) text.ScrollTime = Now - _selectedAt;
            }
        }
    }
}
