using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.VisualTree;

namespace EmuSen.WiseMan.LunaP
{
    // Typed visual-tree lookups, so a control test asserts about a rendered part rather than a field it happens to hold.
    internal static class VisualQuery
    {
        public static T? FindPart<T>(this Visual root) where T : Visual =>
            root.GetVisualDescendants().OfType<T>().FirstOrDefault();

        public static IEnumerable<T> FindParts<T>(this Visual root) where T : Visual =>
            root.GetVisualDescendants().OfType<T>();

        public static int CountParts<T>(this Visual root) where T : Visual =>
            root.GetVisualDescendants().OfType<T>().Count();
    }
}
