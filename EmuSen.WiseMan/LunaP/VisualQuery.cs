using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.VisualTree;

namespace EmuSen.WiseMan.LunaP
{
    // Typed visual-tree lookups, so a control test asserts about a rendered part rather than a field it happens to hold.
    public static class VisualQuery
    {
        public static T? FindPart<T>(this Visual root) where T : Visual =>
            root.GetVisualDescendants().OfType<T>().FirstOrDefault();

        public static IEnumerable<T> FindParts<T>(this Visual root) where T : Visual =>
            root.GetVisualDescendants().OfType<T>();

        public static int CountParts<T>(this Visual root) where T : Visual =>
            root.GetVisualDescendants().OfType<T>().Count();

        // For windows that build their own tree in code: there is no XAML namescope for GetControl to search.
        public static T FindNamed<T>(this Visual root, string name) where T : Control =>
            root.GetVisualDescendants().OfType<T>().First(c => c.Name == name);
    }
}
