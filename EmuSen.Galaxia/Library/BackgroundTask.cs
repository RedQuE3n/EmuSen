using System;
using System.Threading.Tasks;

namespace EmuSen.Galaxia.Library
{
    // A task started and not awaited, its fault written to the error log instead of dropped - see EmuSen_Cores.md §3.5.
    public static class BackgroundTask
    {
        public static void Logged(this Task? task, string area, string what)
        {
            task?.ContinueWith(t => ErrorLog.Error(area, $"{what} failed", t.Exception?.GetBaseException()), TaskContinuationOptions.OnlyOnFaulted | TaskContinuationOptions.ExecuteSynchronously);
        }
    }
}
