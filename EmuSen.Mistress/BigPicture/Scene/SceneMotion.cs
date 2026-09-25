using System;
using Avalonia.Animation.Easings;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // A held direction's timing: the first repeat's delay, the interval, and a faster tier after a while held.
    public sealed record SceneRepeatRule(TimeSpan Delay, TimeSpan Interval, TimeSpan? FastAfter = null, TimeSpan? FastInterval = null);

    // How the scene moves, as ES-DE 3.4.1 was measured moving - see EmuSen_BigPicture.md §14.7.
    public sealed record SceneMotion
    {
        public TimeSpan CarouselStep { get; init; }
        public Easing? CarouselEasing { get; init; }
        public required SceneRepeatRule CarouselRepeat { get; init; }
        public required SceneRepeatRule CarouselFastRepeat { get; init; }
        public required SceneRepeatRule ListRepeat { get; init; }
        public bool ListWraps { get; init; }

        // The gamelist's metadata and media fade out while the list moves fast, and back in when it stops.
        public TimeSpan MetadataFadeOut { get; init; }
        public TimeSpan MetadataFadeIn { get; init; }

        // An image or video with scrollFadeIn fades in over this when the game changes.
        public TimeSpan ScrollFadeIn { get; init; }

        // The textlist's selected name: pixels a second per unit of textHorizontalScrollSpeed and per pixel of font size, and the gap per unit in font sizes.
        public double MarqueeSpeedPerEm { get; init; }
        public double MarqueeGapPerUnit { get; init; }

        // Text containers: the same for the horizontal type; the vertical type's speed per unit in line heights a second, and its fade-in.
        public double HorizontalContainerSpeedPerEm { get; init; }
        public double HorizontalContainerGapPerUnit { get; init; }
        public double VerticalContainerLinesPerSecond { get; init; }
        public TimeSpan VerticalContainerFadeIn { get; init; }

        // The slide between the system and gamelist views.
        public TimeSpan ViewSlide { get; init; }
        public Easing? ViewSlideEasing { get; init; }

        public static SceneMotion Esde { get; } = new()
        {
            CarouselRepeat = new(TimeSpan.Zero, TimeSpan.Zero),
            CarouselFastRepeat = new(TimeSpan.Zero, TimeSpan.Zero),
            ListRepeat = new(TimeSpan.Zero, TimeSpan.Zero),
        };
    }
}
