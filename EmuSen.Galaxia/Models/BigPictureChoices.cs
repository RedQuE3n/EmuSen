namespace EmuSen.Galaxia.Models
{
    // One ES-DE theme's choices from its settings sheet, by the names its capabilities.xml declares; null is Automatic or the theme's default - see EmuSen_Settings_Reference.md §4.53.
    public sealed class BigPictureChoices
    {
        public string? Variant { get; set; }
        public string? ColorScheme { get; set; }
        public string? FontSize { get; set; }
        public string? AspectRatio { get; set; }
        public string? Language { get; set; }
        public string? Transitions { get; set; }
    }
}
