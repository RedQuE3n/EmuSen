using System;
using System.Collections.Generic;
using Avalonia.Threading;
using EmuSen.Cores.Native;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // Change Disc in the game's menu, for a game of several discs on a core that changes them: the lid opened, the disc in, the lid closed a moment later - see EmuSen_Settings_Reference.md §4.106.
    public partial class MainWindow
    {
        private int _discChoice;

        // The running engine where it changes discs and the game has more than one; null otherwise.
        private CoreEngine? DiscEngine => _session?.Core is CoreEngine { CanChangeDisc: true, Discs.Count: > 1 } engine ? engine : null;

        private string DiscValue() => DiscEngine is { } e ? $"Disc {_discChoice + 1} of {e.Discs.Count}{(_discChoice == e.DiscIndex ? " (in)" : "")}" : "";

        private void StepDisc(int step)
        {
            if (DiscEngine is not { } e) return;
            _discChoice = (_discChoice + step + e.Discs.Count) % e.Discs.Count;
        }

        // On the emulation thread, as a state's load is, so no frame runs half through the change.
        private void ChangeDisc()
        {
            if (DiscEngine is null) return;
            int disc = _discChoice;
            RequestOnEmulationThread(_ =>
            {
                string status;
                try
                {
                    (_session?.Core as CoreEngine)?.ChangeDisc(disc);
                    status = $"Disc {disc + 1} is in";
                }
                catch (System.Exception ex) when (ex is System.InvalidOperationException or System.IO.IOException or System.IO.InvalidDataException)
                {
                    status = $"Change Disc failed: {ex.Message}";
                }
                Dispatcher.UIThread.Post(() => { StatusText.Text = status; Notify(status); });
            });
        }

        // The row, where there is a disc to change to.
        private IEnumerable<PadMenuEntry> DiscRows()
        {
            if (DiscEngine is not { } e) yield break;
            if (_discChoice >= e.Discs.Count || _discChoice < 0) _discChoice = Math.Max(0, e.DiscIndex);
            yield return new PadMenuEntry(() => $"Change Disc      <  {DiscValue()}  >", ChangeDisc, StepDisc) { Label = () => "Change Disc", Value = DiscValue };
        }
    }
}
