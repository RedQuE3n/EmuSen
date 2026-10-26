using System;
using System.Linq;
using Avalonia.Controls;
using Avalonia.VisualTree;

namespace EmuSen.Mistress.Views
{
    // A console tab strip kept to one row: where its headers would wrap, every header's text is made smaller together, never below MinFontSize - see Sapphire_Native.md §49.6.
    public static class TabStripFit
    {
        public const double MinFontSize = 20;

        // Fits tabs' headers to its strip less reserve, the width of what the window draws at the strip's right.
        public static void OneRow(TabControl tabs, Func<double> reserve)
        {
            double themed = 0;
            tabs.LayoutUpdated += (_, _) =>
            {
                TabItem[] items = tabs.Items.OfType<TabItem>().Where(t => t.IsVisible).ToArray();
                if (items.Length == 0 || items.Any(t => t.DesiredSize.Width <= 0)) return;
                if (themed == 0) themed = items[0].FontSize;
                double strip = (items[0].GetVisualParent() as Control)?.Bounds.Width ?? tabs.Bounds.Width;
                double size = items[0].FontSize, padding = items.Sum(t => t.DesiredSize.Width - TextWidth(t)), text = items.Sum(TextWidth);
                double room = strip - reserve() - padding;
                if (strip <= 0 || text <= 0 || room <= 0) return;
                double wanted = Math.Max(MinFontSize, Math.Min(themed, Math.Floor(size * room / text * 2) / 2));
                if (Math.Abs(wanted - size) < 0.25) return;
                foreach (TabItem t in items) t.FontSize = wanted;
            };
        }

        // The header's own width, without the item's padding and margin.
        private static double TextWidth(TabItem t) => Math.Max(0, t.DesiredSize.Width - t.Padding.Left - t.Padding.Right - t.Margin.Left - t.Margin.Right);
    }
}
