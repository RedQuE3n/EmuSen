using System;
using Avalonia;
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
            _changed = false;
            _position = Glide.At(Index);
            _repeat = new SceneRepeat();
            Root = new Panel { Width = data.Screen.Width, Height = data.Screen.Height };
            Scene = Rebuild();
            _scroll = Glide.At(Grid()?.ScrollFor(Index) ?? 0);
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
        private ResolvedElement? Primary => View.Primary;

        // Whether the primary element slides between items; ES-DE's itemTransitions, and only its carousel and grid have one.
        private bool Slides => Primary is { Type: "carousel" } p && p.String("itemTransitions") != "instant";

        // One step of the selection, as a tap gives it: a carousel and a list both wrap at their ends.
        public void Step(int delta, TimeSpan now) => Move(delta, now, held: false);

        // A move by several items that stops at a list's ends, as a page or a jump to the first or last does; false when nothing moved.
        public bool Jump(int delta, TimeSpan now) => Move(delta, now, held: true);

        // Raised for every move of the selection, the repeats of a held direction included, with its signed size and whether it was held.
        public event Action<int, bool>? Stepped;

        // How many items the view lists, and which is selected.
        public int Count => IsSystemView ? Data.Systems.Count : Data.System.Games.Count;
        public int Index => IsSystemView ? Data.SystemIndex : Data.GameIndex;

        // Redraws the help bar for another pad without rebuilding the view, so nothing else changes (§15).
        public void SetFamily(PadFamily family)
        {
            Data = Data with { Family = family };
            foreach (SceneEntry e in Scene.Entries)
                if (e.Control is HintBar bar) bar.PadFamily = family;
        }

        // When the view next looks different from now: now while anything moves, a later time for a text still in its pause, null when it is still for good (§15).
        public TimeSpan? NextChange(TimeSpan now)
        {
            if (!_position.IsSettledAt(now) || _repeat.Held || !_metadata.IsSettledAt(now) || !_scroll.IsSettledAt(now) || !_focus.IsSettledAt(now)) return now;
            if (_changed && now < _selectedAt + Motion.ScrollFadeIn && Scene.Entries.Any(e => e.Control is not null && FadesIn(e.Element))) return now;
            TimeSpan? next = null;
            foreach (SceneEntry e in Scene.Entries)
            {
                TimeSpan? at = e.Control switch
                {
                    FontText { ScrollDirection: not TextScrollDirection.None, IsEffectivelyVisible: true } t => t.IsMeasureValid ? t.NextScrollChange(now - _selectedAt) : now - _selectedAt,
                    TextRowList { Marquee.Speed: > 0, IsEffectivelyVisible: true } l => l.IsMeasureValid && l.Bounds.Width > 0 ? l.NextMarqueeChange(now - _selectedAt) : now - _selectedAt,
                    _ => null,
                };
                if (at is { } due && (next is null || _selectedAt + due < next)) next = _selectedAt + due;
            }
            return next;
        }

        // A move of the selection; a held list stops at its end, where a carousel wraps; false when nothing moved.
        private bool Move(int delta, TimeSpan now, bool held)
        {
            if (Primary?.Type == "grid") return MoveGrid(delta, now, held, _vertical);
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
            _changed = true;
            if (held && Hidden && Scene.Entries.FirstOrDefault(e => e.Control is TextRowList)?.Control is TextRowList list)
            {
                list.SelectedIndex = target;
                _stale = true;
            }
            else
            {
                Scene = Rebuild();
                _stale = false;
            }

            Apply();
            Stepped?.Invoke(delta, held);
            return true;
        }

        // While a held list has faded the game's metadata out, every element that follows the game is invisible, so a step moves the list alone and the rest is rebuilt when the fade-in starts (§14.8).
        private bool Hidden => !RebuildEveryStep && !IsSystemView && _metadata.To <= 0 && _metadata.IsSettledAt(Now);

        // Turns the lever above off, so a test or bench can compare it with a rebuild on every step.
        public bool RebuildEveryStep { get; init; }

        private bool _stale;

        // A direction held from a time on; the repeats it gives are stepped by Advance; a grid's vertical direction moves by rows.
        public void Press(int direction, TimeSpan now, bool vertical = false)
        {
            _vertical = vertical && Primary?.Type == "grid";
            Step(direction, now);
            _repeat.Press(direction, now, RepeatRule);
        }

        private bool _vertical;
        private Glide _scroll = Glide.At(0);
        private Glide _focus = Glide.At(1);
        private int _focusFrom = -1;
        private double _fromLevel = 1, _toLevel;

        // The grid's layout at the element's size, as the control will lay it out; null when the view's primary element is no grid.
        public GridGeometry? Grid()
        {
            if (Primary?.Type != "grid" || Scene.Entries.FirstOrDefault(e => e.Control is ImageGrid) is not { Control: ImageGrid grid } entry) return null;
            Size box = SceneUnits.ToSize(entry.Element.Pair("size"));
            return grid.GeometryFor(new Size(box.Width * Data.Screen.Width, box.Height * Data.Screen.Height));
        }

        // An item's focus at a time: the selection rising from its level, the item left falling from its own, the rest none.
        private double FocusOf(int index, TimeSpan at)
        {
            double p = _focus.ValueAt(at);
            if (index == Index) return _focusFrom == Index ? 1 : _toLevel + (1 - _toLevel) * p;
            return index == _focusFrom ? _fromLevel * (1 - p) : 0;
        }

        // ES-DE's grid, measured: across a row's end to the next row; a tap wraps at the list's ends and a hold stops; up and down stop at the first and last rows, and down into a short last row takes its last item (§16).
        private bool MoveGrid(int delta, TimeSpan now, bool held, bool vertical)
        {
            Now = now;
            int count = Count;
            if (count == 0 || delta == 0 || Grid() is not { } g) return false;
            int target;
            if (vertical)
            {
                target = Index + Math.Sign(delta) * g.Columns;
                if (target < 0) return false;
                if (target >= count)
                {
                    if (g.RowOf(Index) >= g.RowOf(count - 1)) return false;
                    target = count - 1;
                }
            }
            else
            {
                target = Index + delta;
                target = held ? Math.Clamp(target, 0, count - 1) : ((target % count) + count) % count;
            }
            if (target == Index) return false;

            _fromLevel = FocusOf(Index, now);
            _toLevel = FocusOf(target, now);
            _focusFrom = Index;
            _focus = Primary!.String("itemTransitions") == "instant" ? Glide.At(1) : new Glide(0, 1, now, Motion.GridStep, Motion.GridEasing);
            double scroll = g.ScrollFor(target);
            _scroll = Primary.String("rowTransitions") == "instant" ? Glide.At(scroll) : _scroll.Toward(scroll, now, Motion.GridStep, Motion.GridEasing);
            Data = IsSystemView ? Data with { SystemIndex = target } : Data with { GameIndex = target };
            _selectedAt = now;
            _changed = true;
            Scene = Rebuild();
            _stale = false;
            Apply();
            Stepped?.Invoke(delta, held);
            return true;
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
            if (!_stale) return;
            Now = now;
            Scene = Rebuild();
            _stale = false;
        }

        private SceneRepeatRule RepeatRule => Primary?.Type == "grid" ? Motion.GridRepeat : Primary?.Type == "carousel" ? (Primary.Bool("fastScrolling") == true ? Motion.CarouselFastRepeat : Motion.CarouselRepeat) : Motion.ListRepeat;

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
        public bool IsMoving => !_position.IsSettledAt(Now) || !_scroll.IsSettledAt(Now) || !_focus.IsSettledAt(Now) || _repeat.Held || !_metadata.IsSettledAt(Now) || (Now < _selectedAt + Motion.ScrollFadeIn && Scene.Entries.Any(e => FadesIn(e.Element))) || Scrolls;

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
        private bool _changed;

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
            double fadeIn = Motion.ScrollFadeIn > TimeSpan.Zero && _changed ? new Glide(Motion.ScrollFadeInFrom, 1, _selectedAt, Motion.ScrollFadeIn).ValueAt(Now) : 1;
            double metadata = _metadata.ValueAt(Now);
            foreach (SceneEntry entry in Scene.Entries)
            {
                if (entry.Control is { } control && _opacity.TryGetValue(control, out double own))
                    control.Opacity = own * (FadesIn(entry.Element) ? fadeIn : 1) * (IsGameMetadata(entry.Element) ? metadata : 1);
                if (entry.Control is ImageCarousel carousel) carousel.Position = _position.ValueAt(Now);
                else if (entry.Control is ImageGrid grid)
                {
                    grid.ScrollRow = _scroll.ValueAt(Now);
                    grid.FocusFrom = _focusFrom;
                    grid.FocusFromLevel = _fromLevel;
                    grid.FocusToLevel = _toLevel;
                    grid.FocusProgress = _focus.ValueAt(Now);
                }
                else if (entry.Control is TextRowList list) list.MarqueeTime = Now - _selectedAt;
                else if (entry.Control is FontText { ScrollDirection: not TextScrollDirection.None } text) text.ScrollTime = Now - _selectedAt;
            }
        }
    }
}
