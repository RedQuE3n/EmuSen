using System;
using System.Collections.Generic;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.LunaP
{
    // Windowing goes through the toolkit, not around it - see EmuSen_LunaP.md §8.
    public class WindowingTests
    {
        private static readonly Assembly[] Frontends =
        {
            typeof(EmuSen.Mistress.Views.MainWindow).Assembly,
            typeof(EmuSen.Hotaru.Views.GameWindow).Assembly,
            typeof(EmuSen.Serenity.GameFrameControl).Assembly,
        };

        private static IEnumerable<Type> DeclaredWindows(Assembly assembly) =>
            assembly.GetTypes().Where(t => typeof(Window).IsAssignableFrom(t) && !t.IsAbstract);

        [Fact]
        public void Every_window_the_frontends_declare_is_a_LunaP_window()
        {
            var raw = new List<string>();

            foreach (Assembly assembly in Frontends)
            {
                foreach (Type window in DeclaredWindows(assembly))
                {
                    if (typeof(ToolWindow).IsAssignableFrom(window)) continue;

                    raw.Add($"{assembly.GetName().Name}.{window.Name} derives from {window.BaseType?.Name}");
                }
            }

            Assert.True(raw.Count == 0, "windows outside LunaP: " + string.Join("; ", raw));
        }

        // The one window nothing declares: FramePresenter builds it, so no type test can see it.
        [Fact]
        public Task The_frame_presenter_opens_a_LunaP_window() => UiTest.Run(() =>
        {
            using var presenter = new EmuSen.Serenity.FramePresenter();

            object? window = typeof(EmuSen.Serenity.FramePresenter)
                .GetField("_window", BindingFlags.Instance | BindingFlags.NonPublic)!
                .GetValue(presenter);

            Assert.IsAssignableFrom<ToolWindow>(window);
        });
    }
}
