using System;

namespace EmuSen.Hotaru.Views
{
    // The live interpreter a running GameWindow lends to a shell window - see EmuSen_Frontend_Driver.md §3c.
    public interface ILiveShell
    {
        bool IsAwaitingMoreInput { get; }
        string[] SnapshotHistory();
        void Submit(string line);
        event Action<string> Output;
    }
}
