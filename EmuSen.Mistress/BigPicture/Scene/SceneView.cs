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

        private bool IsSystemView => ViewName == "system";
        private int Count => IsSystemView ? Data.Systems.Count : Data.System.Games.Count;
        private int Index => IsSystemView ? Data.SystemIndex : Data.GameIndex;
        private ResolvedElement? Primary => View.Primary;

        // Whether the primary element slides between items; ES-DE's itemTransitions, and only its carousel and grid have one.
        private bool Slides => Primary is { Type: "carousel" } p && p.String("itemTransitions") != "instant";

        // One step of the selection, as a press gives it.
        public void Step(int delta, TimeSpan now)
        {
            Now = now;
            int count = Count;
            if (count == 0 || delta == 0) return;
            int target = Index + delta;
            bool wraps = IsSystemView || Primary?.Type == "carousel" || Motion.ListWraps;
            if (wraps) target = ((target % count) + count) % count;
            else target = Math.Clamp(target, 0, count - 1);
            if (target == Index) return;
            double to = _position.To + (wraps ? delta : target - Index);
            Data = IsSystemView ? Data with { SystemIndex = target } : Data with { GameIndex = target };
            _position = Slides ? _position.Toward(to, now, Motion.CarouselStep, Motion.CarouselEasing) : Glide.At(to);
            _selectedAt = now;
            Scene = Rebuild();
            Apply();
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
        }

        private SceneRepeatRule RepeatRule => Primary?.Type == "carousel" ? (Primary.Bool("fastScrolling") == true ? Motion.CarouselFastRepeat : Motion.CarouselRepeat) : Motion.ListRepeat;

        // Moves the view's clock on: key repeats that fall due, then every time-derived state at the new time.
        public void Advance(TimeSpan now)
        {
            foreach ((int direction, TimeSpan at) in _repeat.Due(now)) Step(direction, at);
            Now = now;
            Apply();
        }

        // Whether anything is still moving, so a host can stop asking for frames.
        public bool IsMoving => !_position.IsSettledAt(Now) || _repeat.Held;

        private SceneBuilder Rebuild()
        {
            SceneBuilder scene = SceneBuilder.Build(View, Data);
            Root.Children.Clear();
            Root.Children.Add(scene.Canvas);
            return scene;
        }

        private void Apply()
        {
            foreach (SceneEntry entry in Scene.Entries)
            {
                if (entry.Control is ImageCarousel carousel) carousel.Position = _position.ValueAt(Now);
                else if (entry.Control is TextRowList list) list.MarqueeTime = Now - _selectedAt;
                else if (entry.Control is FontText { ScrollDirection: not TextScrollDirection.None } text) text.ScrollTime = Now - _selectedAt;
            }
        }
    }
}
