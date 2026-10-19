using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using EmuSen.Crystal.Client;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    // The player's one-time approval of a core they added, asked before a game opens on it - see EmuSen_Backend.md §29.40.8.
    public partial class MainWindow
    {
        // The question, which a test answers in place of a person.
        internal static Func<MainWindow, string, string, Task<bool>> ConfirmCoreApproval =
            (owner, title, message) => Dialogs.ConfirmAsync(owner, title, message, "Approve", "Not Now");

        // What the approval shows of a core library, read from the runtime's listing.
        internal sealed record CoreAwaiting(string File, string Name, IReadOnlyList<string> Authors, string License, IReadOnlyList<string> Systems, string Sha256);

        // Asked when the engine the game would open on waits for the player's approval; approved, the runtime records it.
        internal async Task AskCoreApprovalAsync(string romPath)
        {
            if (CoreAwaitingApproval(Runtime, romPath) is not { } core) return;
            if (!await ConfirmCoreApproval(this, "Approve a Core", ApprovalMessage(core))) return;
            try
            {
                Runtime.CoreApprove(core.Sha256);
                StatusText.Text = $"Approved {core.Name} ({core.File}).";
            }
            catch (CrystalException e) { StatusText.Text = $"{core.File} could not be approved: {e.Message}"; }
        }

        // The library of the engine route chooses for the file, where its trust is "needs_approval"; null otherwise.
        internal static CoreAwaiting? CoreAwaitingApproval(CrystalRuntime runtime, string romPath)
        {
            try
            {
                using var route = JsonDocument.Parse(runtime.Route(romPath));
                if (route.RootElement.GetProperty("engine").GetString() is not { } engine) return null;
                using var cores = JsonDocument.Parse(runtime.Cores());
                foreach (JsonElement c in cores.RootElement.EnumerateArray())
                {
                    if (c.GetProperty("engine").ValueKind != JsonValueKind.String || c.GetProperty("engine").GetString() != engine) continue;
                    if (!c.TryGetProperty("trust", out var trust) || trust.GetString() != "needs_approval") return null;
                    static string[] Texts(JsonElement e) => e.EnumerateArray().Select(x => x.GetString() ?? "").ToArray();
                    return new CoreAwaiting(System.IO.Path.GetFileName(c.GetProperty("library").GetString() ?? ""), c.GetProperty("name").GetString() ?? engine,
                        Texts(c.GetProperty("authors")), c.GetProperty("license").GetString() ?? "", Texts(c.GetProperty("systems")), c.GetProperty("sha256").GetString() ?? "");
                }
                return null;
            }
            catch (Exception e) when (e is CrystalException or JsonException or KeyNotFoundException or InvalidOperationException) { return null; }
        }

        // The words of the question, plain text: a core's descriptors are shown, never interpreted.
        internal static string ApprovalMessage(CoreAwaiting core) =>
            $"{core.Name}, the core in {core.File}, is not one this installation ships, and it has not been approved.\n\n"
            + $"By: {(core.Authors.Count > 0 ? string.Join(", ", core.Authors) : "not said")}\n"
            + $"Licence: {(core.License.Length > 0 ? core.License : "not said")}\n"
            + $"Systems: {(core.Systems.Count > 0 ? string.Join(", ", core.Systems) : "none")}\n"
            + $"SHA-256: {core.Sha256}\n\n"
            + "A core runs with your full permissions. Approve it only if you trust where it came from. If the file changes, you will be asked again.";
    }
}
