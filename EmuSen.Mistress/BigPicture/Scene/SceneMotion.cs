using System;
using Avalonia.Animation.Easings;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // A held direction's timing: the first repeat's delay, the interval, and from FastAfter a faster interval, the switch moving FastJump items at once.
    public sealed record SceneRepeatRule(TimeSpan Delay, TimeSpan Interval, TimeSpan? FastAfter = null, TimeSpan? FastInterval = null, int FastJump = 1);

    // How the scene moves, as ES-DE 3.4.1 was measured moving - see EmuSen_BigPicture.md §14.7.
    public sealed record SceneMotion
    {
        public TimeSpan CarouselStep { get; init; }
        public Easing? CarouselEasing { get; init; }
        public required SceneRepeatRule CarouselRepeat { get; init; }
        public required SceneRepeatRule CarouselFastRepeat { get; init; }
        public required SceneRepeatRule ListRepeat { get; init; }

        // The gamelist's metadata and media fade out from the first repeat of a held direction, and back in when scrolling stops.
        public TimeSpan MetadataFadeOut { get; init; }
        public TimeSpan MetadataFadeIn { get; init; }

        // An image or video with scrollFadeIn rises from this opacity to full over ScrollFadeIn when the game changes.
        public TimeSpan ScrollFadeIn { get; init; }
        public double ScrollFadeInFrom { get; init; }

        // The textlist's selected name: font sizes a second at speed 1, and the gap as seconds of that travel per unit of textHorizontalScrollGap.
        public double MarqueeSpeedPerEm { get; init; }
        public double MarqueeGapSeconds { get; init; }

        // Text containers: the horizontal type as the marquee; the vertical type's font sizes a second at speed 1, whole-pixel steps, and its fade-in at the top.
        public double HorizontalContainerSpeedPerEm { get; init; }
        public double HorizontalContainerGapSeconds { get; init; }
        public double VerticalContainerSpeedPerEm { get; init; }
        public TimeSpan VerticalContainerFadeIn { get; init; }

        // The grid: a step's scale and fade and a row's slide on one curve and duration, and a held direction's repeats, with no faster tier (§16).
        public TimeSpan GridStep { get; init; }
        public Easing? GridEasing { get; init; }
        public SceneRepeatRule GridRepeat { get; init; } = new(TimeSpan.FromMilliseconds(500), TimeSpan.FromMilliseconds(200));

        // The camera pan between the system and gamelist views, a screen height, the gamelist below.
        public TimeSpan ViewSlide { get; init; }
        public Easing? ViewSlideEasing { get; init; }

        private static TimeSpan Ms(double ms) => TimeSpan.FromMilliseconds(ms);

        public static SceneMotion Esde { get; } = new()
        {
            CarouselStep = Ms(400), CarouselEasing = new QuadraticEaseOut(),
            CarouselRepeat = new(Ms(500), Ms(200)),
            CarouselFastRepeat = new(Ms(500), Ms(180), Ms(1580), Ms(80)),
            ListRepeat = new(Ms(500), Ms(114), Ms(1703), Ms(15.9), 4),
            MetadataFadeOut = Ms(149), MetadataFadeIn = Ms(150),
            ScrollFadeIn = Ms(326), ScrollFadeInFrom = 0.5,
            MarqueeSpeedPerEm = 131.5 / 30, MarqueeGapSeconds = 1,
            HorizontalContainerSpeedPerEm = 131.5 / 30, HorizontalContainerGapSeconds = 1,
            VerticalContainerSpeedPerEm = 37.03 / 30, VerticalContainerFadeIn = Ms(298),
            ViewSlide = Ms(402), ViewSlideEasing = new CubicEaseOut(),
            GridStep = Ms(250), GridEasing = new QuadraticEaseOut(), GridRepeat = new(Ms(500), Ms(200)),
        };
    }
}
