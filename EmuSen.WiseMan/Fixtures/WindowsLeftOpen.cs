using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Threading;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Xunit.Sdk;

[assembly: EmuSen.WiseMan.Fixtures.WindowsLeftOpen]

namespace EmuSen.WiseMan.Fixtures
{
    // Every window a test opens and leaves open is closed when the test ends - see EmuSen_Debugging_Tools_Reference_v5.md §3.62.
    [AttributeUsage(AttributeTargets.Assembly)]
    public sealed class WindowsLeftOpen : BeforeAfterTestAttribute
    {
        private static readonly AsyncLocal<List<WeakReference<Window>>?> Opened = new();
        private static readonly ConditionalWeakTable<Window, object> Closed = new();
        private static int _watching;

        private static void Watch()
        {
            if (Interlocked.Exchange(ref _watching, 1) == 1) return;
            Window.WindowOpenedEvent.AddClassHandler<Window>((w, _) =>
            {
                if (Opened.Value is { } list) lock (list) list.Add(new WeakReference<Window>(w));
            });
            Window.WindowClosedEvent.AddClassHandler<Window>((w, _) => Closed.AddOrUpdate(w, Closed));
        }

        public override void Before(MethodInfo methodUnderTest)
        {
            Watch();
            Opened.Value = new List<WeakReference<Window>>();
        }

        public override void After(MethodInfo methodUnderTest)
        {
            List<WeakReference<Window>>? list = Opened.Value;
            Opened.Value = null;
            if (list is null) return;
            Window[] left;
            lock (list) left = list.Select(r => r.TryGetTarget(out Window? w) ? w : null).OfType<Window>().Where(w => !Closed.TryGetValue(w, out _)).Distinct().ToArray();
            if (left.Length == 0) return;
            if (Environment.GetEnvironmentVariable("EMUSEN_LEFT_WINDOWS") is { Length: > 0 } record)
                lock (Closed) File.AppendAllText(record, $"{methodUnderTest.DeclaringType?.FullName}.{methodUnderTest.Name}: {string.Join(", ", left.Select(w => w.GetType().Name))}{Environment.NewLine}");
            UiTest.Session.Dispatch(() =>
            {
                foreach (Window w in left.Reverse()) if (!Closed.TryGetValue(w, out _)) w.Close();
            }, default).GetAwaiter().GetResult();
        }
    }
}
