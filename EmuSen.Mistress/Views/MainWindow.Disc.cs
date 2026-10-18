using System;
using System.Collections.Generic;
using Avalonia.Threading;
using EmuSen.Crystal.Client;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // Change Disc in the game's menu, for a game of several discs on a core that changes them: the lid opened, the disc in, the lid closed a moment later - see EmuSen_Settings_Reference.md §4.106.
    public partial class MainWindow
    {
        private int _discChoice;

        // The drive where it changes discs and the game has more than one; null otherwise.
        private (int Count, int Index)? DiscDrive => _session?.Discs is { CanChange: true, Count: > 1 } d ? (d.Count, d.Index) : null;

        private string DiscValue() => DiscDrive is { } d ? $"Disc {_discChoice + 1} of {d.Count}{(_discChoice == d.Index ? " (in)" : "")}" : "";

        private void StepDisc(int step)
        {
            if (DiscDrive is not { } d) return;
            _discChoice = (_discChoice + step + d.Count) % d.Count;
        }

        // At the next frame boundary, as the runtime applies a state's load, so no frame runs half through the change.
        private void ChangeDisc()
        {
            if (DiscDrive is null || _session is not { } session) return;
            int disc = _discChoice;
            System.Threading.Tasks.Task.Run(() =>
            {
                string status;
                try
                {
                    session.Crystal.ChangeDisc((uint)disc);
                    status = $"Disc {disc + 1} is in";
                }
                catch (CrystalException ex)
                {
                    status = $"Change Disc failed: {ex.Message}";
                }
                Dispatcher.UIThread.Post(() => { StatusText.Text = status; Notify(status); });
            });
        }

        // The row, where there is a disc to change to.
        private IEnumerable<PadMenuEntry> DiscRows()
        {
            if (DiscDrive is not { } d) yield break;
            if (_discChoice >= d.Count || _discChoice < 0) _discChoice = Math.Max(0, d.Index);
            yield return new PadMenuEntry(() => $"Change Disc      <  {DiscValue()}  >", ChangeDisc, StepDisc) { Label = () => "Change Disc", Value = DiscValue };
        }
    }
}
