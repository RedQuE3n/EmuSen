using EmuSen.Cores;

namespace EmuSen.WiseMan.Fixtures
{
    // The create options the suite makes its cores with: no battery file read or written, so a run never depends on the last - see EmuSen_Multicore.md §6.
    public static class TestCores
    {
        public static readonly CoreOptions NoBattery = new() { BatteryRamDisabled = true };

        // The main window's sessions opened the same way, before anything is loaded.
        public static T WithoutBattery<T>(this T window) where T : EmuSen.Mistress.Views.MainWindow
        {
            var options = typeof(EmuSen.Mistress.Views.MainWindow).GetProperty("OpenOptions", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.NonPublic)!;
            options.SetValue(window, ((EmuSen.Crystal.Client.CrystalOpen)options.GetValue(window)!) with { NoBattery = true });
            return window;
        }
    }
}
