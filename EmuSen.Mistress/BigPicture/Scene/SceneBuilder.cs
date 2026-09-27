using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // What one element became, for tests and the geometry check: the element, its control, and whether it was drawn.
    public sealed record SceneEntry(ResolvedElement Element, Control? Control, string? Skipped);

    // The LunaP control tree for one resolved view: a pure function of the view and the data - see EmuSen_BigPicture.md §13.2.
    public sealed class SceneBuilder
    {
        private readonly List<SceneEntry> _entries = new();

        private SceneBuilder(ResolvedView view, SceneData data)
        {
            View = view;
            Data = data;
            Canvas = new NormalizedCanvas { Width = data.Screen.Width, Height = data.Screen.Height, UseLayoutRounding = false };
        }

        public ResolvedView View { get; }
        public SceneData Data { get; }
        public NormalizedCanvas Canvas { get; }
        public IReadOnlyList<SceneEntry> Entries => _entries;

        // The system view's gameselectors, picked once per build; none in the gamelist view (§36).
        public GameSelectors Selectors => _selectors ??= View.Name == "system" ? GameSelectors.For(View, Data.System, Data.Shuffle) : GameSelectors.None;

        private GameSelectors? _selectors;

        // The game an element's media and metadata come from: the list's selection, or in the system view a gameselector's pick (§36).
        public SceneGame? GameFor(ResolvedElement e) => View.Name == "system" ? Selectors.GameFor(e) : Data.Game;

        public double W => Data.Screen.Width;
        public double H => Data.Screen.Height;

        public static SceneBuilder Build(ResolvedView view, SceneData data)
        {
            var builder = new SceneBuilder(view, data);
            for (int i = 0; i < view.Elements.Count; i++) builder.Add(view.Elements[i], i);
            return builder;
        }

        public SceneEntry? Find(string type, string name) => _entries.FirstOrDefault(e => e.Element.Type == type && e.Element.Name == name);

        private void Add(ResolvedElement e, int order)
        {
            string? skip = Skip(e);
            Control? control = skip is null ? Create(e, out skip) : null;
            if (control is not null)
            {
                Place(e, control, order);
                control.UseLayoutRounding = false; // ES-DE places at fractional pixels (§13.8).
                Canvas.Children.Add(control);
            }

            _entries.Add(new SceneEntry(e, control, skip));
        }

        // The reasons an element draws nothing, before any control is made.
        private string? Skip(ResolvedElement e)
        {
            if (e.Type == "gameselector") return "a gameselector picks games and draws nothing";
            if (e.Bool("visible") == false) return "visible is false";
            if (e.Bool("metadataElement") == true && Data.HideMetadata) return "metadata elements hidden";
            if (View.Name == "gamelist" && Data.Game is { HideMetadata: true } && HiddenWithMetadata(e)) return "the entry hides its metadata";
            if (e.String("scope") is "none" or "menu") return $"scope {e.String("scope")}";
            if (e.Float("opacity") is 0) return "opacity is 0";
            if (e.Type == "clock" && !Data.ShowClock) return "the clock is turned off";
            if (e.Type == "helpsystem" && !Data.ShowHelp) return "the help is turned off";
            return null;
        }

        // THEMES.md's metadataElement: text metadata fields, ratings and badges, and elements the theme marks; the description stays, as ES-DE 3.4.1 was seen to keep it (§22).
        private static bool HiddenWithMetadata(ResolvedElement e) => e.Bool("metadataElement") == true || e.Type switch
        {
            "rating" or "badges" or "datetime" => true,
            "text" => e.String("metadata") is { } m && m is not ("description" or "name" or "systemName" or "systemFullname" or "sourceSystemName" or "sourceSystemFullname"),
            _ => false,
        };

        private Control? Create(ResolvedElement e, out string? skip)
        {
            skip = null;
            Control? control = e.Type switch
            {
                "image" => ImageElements.Image(this, e),
                "video" => ImageElements.Video(this, e),
                "text" => TextElements.Text(this, e),
                "datetime" => TextElements.DateTime(this, e),
                "carousel" => PrimaryElements.Carousel(this, e),
                "textlist" => PrimaryElements.TextList(this, e),
                "grid" => GridElements.Grid(this, e),
                "rating" => IndicatorElements.Rating(this, e),
                "badges" => IndicatorElements.Badges(this, e),
                "helpsystem" => IndicatorElements.Help(this, e),
                "clock" => IndicatorElements.Clock(this, e),
                "systemstatus" => IndicatorElements.Status(this, e),
                _ => null,
            };
            if (control is null) skip = SceneMapping.Drawn.Contains(e.Type) ? "nothing to show" : $"{e.Type} is not drawn in stage (b)";
            return control;
        }

        // Every element's box, origin, rotation, opacity and drawing order, which §4.3 gives for all of them alike.
        private void Place(ResolvedElement e, Control control, int order)
        {
            NormalizedCanvas.SetPosition(control, SceneUnits.ToPoint(e.Pair("pos")));
            NormalizedCanvas.SetOrigin(control, SceneUnits.ToPoint(e.Pair("origin")));
            if (!control.IsSet(NormalizedCanvas.SizeProperty)) NormalizedCanvas.SetSize(control, SceneUnits.ToSize(e.Pair("size")));
            NormalizedCanvas.SetRotation(control, e.Float("rotation") ?? 0);
            NormalizedCanvas.SetRotationOrigin(control, SceneUnits.ToPoint(e.Pair("rotationOrigin"), new Point(0.5, 0.5)));
            NormalizedCanvas.SetDepth(control, order);
            if (e.Float("opacity") is { } opacity) control.Opacity *= opacity;
        }
    }
}
