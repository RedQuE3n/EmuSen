using EmuSen.Cores;

namespace EmuSen.WiseMan.Fixtures
{
    // The create options the suite makes its cores with: no battery file read or written, so a run never depends on the last - see EmuSen_Multicore.md §6.
    public static class TestCores
    {
        public static readonly CoreOptions NoBattery = new() { BatteryRamDisabled = true };

        // The main window's cores made the same way, before anything is loaded.
        public static T WithoutBattery<T>(this T window) where T : EmuSen.Mistress.Views.MainWindow
        {
            typeof(EmuSen.Mistress.Views.MainWindow).GetProperty("CoreOptions", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!.SetValue(window, NoBattery);
            return window;
        }
    }
}
