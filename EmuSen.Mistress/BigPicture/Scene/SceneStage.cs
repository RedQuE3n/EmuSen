using System;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.LunaP.Motion;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // The system and gamelist views and the move between them, instant or sliding as the theme's transition profile says - see EmuSen_BigPicture.md §14.7.
    public sealed class SceneStage
    {
        private SceneView? _leaving;
        private Glide _slide = Glide.At(0);
        private int _direction;

        public SceneStage(SceneData data, string view, TimeSpan now)
        {
            Current = new SceneView(data, view, now, data.Motion);
            Root = new Panel { Width = data.Screen.Width, Height = data.Screen.Height, ClipToBounds = true };
            Root.Children.Add(Current.Root);
        }

        public Panel Root { get; }
        public SceneView Current { get; private set; }
        public TimeSpan Now => Current.Now;

        public bool IsMoving => Current.IsMoving || _leaving is not null;

        // Opens the other view with the current selection: the gamelist from the system view, or back.
        public void Switch(TimeSpan now)
        {
            SceneData data = Current.Data;
            bool toGamelist = Current.ViewName == "system";
            TransitionProfile profile = data.System.Theme.Transitions;
            TransitionAnimation animation = toGamelist ? profile.SystemToGamelist : profile.GamelistToSystem;
            var next = new SceneView(data, toGamelist ? "gamelist" : "system", now, data.Motion);
            Finish();
            Root.Children.Add(next.Root);
            if (animation == TransitionAnimation.Slide && data.Motion.ViewSlide > TimeSpan.Zero)
            {
                _leaving = Current;
                _direction = toGamelist ? 1 : -1;
                _slide = new Glide(0, 1, now, data.Motion.ViewSlide, data.Motion.ViewSlideEasing);
            }
            else Root.Children.Remove(Current.Root);

            Current = next;
            Advance(now);
        }

        public void Advance(TimeSpan now)
        {
            Current.Advance(now);
            if (_leaving is null) return;
            double h = Current.Data.Screen.Height, p = _slide.ValueAt(now), y = _direction * (1 - p) * h;
            _leaving.Root.RenderTransform = new TranslateTransform(0, -_direction * p * h);
            Current.Root.RenderTransform = new TranslateTransform(0, y);
            foreach (SceneEntry e in Overlays(_leaving)) e.Control!.IsVisible = false;
            foreach (SceneEntry e in Overlays(Current)) e.Control!.RenderTransform = new TranslateTransform(0, -y);
            if (_slide.IsSettledAt(now)) Finish();
        }

        // The help bar, clock and status stay on the screen while the views pan beneath them: the new view's at once, the old view's gone.
        private static System.Collections.Generic.IEnumerable<SceneEntry> Overlays(SceneView view) =>
            System.Linq.Enumerable.Where(view.Scene.Entries, e => e.Control is not null && e.Element.Type is "helpsystem" or "clock" or "systemstatus");

        private void Finish()
        {
            if (_leaving is not null) Root.Children.Remove(_leaving.Root);
            _leaving = null;
            Current.Root.RenderTransform = null;
            foreach (SceneEntry e in Overlays(Current)) e.Control!.RenderTransform = null;
        }
    }
}
