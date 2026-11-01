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
            if (!await ConfirmCoreApproval(this, EmuSen.Tools.Cores.CoreApproval.Title, ApprovalMessage(core))) return;
            try
            {
                Runtime.CoreApprove(core.Sha256);
                StatusText.Text = $"Approved {core.Name} ({core.File}).";
            }
            catch (CrystalException e) { StatusText.Text = $"{core.File} could not be approved: {e.Message}"; }
        }

        // The core the game would open on, where it waits for the player's approval: the routed engine's library, or a libretro core's whose hash the listing gives (EmuSen_LibretroHost.md §10m); null otherwise.
        internal static CoreAwaiting? CoreAwaitingApproval(CrystalRuntime runtime, string romPath)
        {
            try
            {
                using var route = JsonDocument.Parse(runtime.Route(romPath));
                string? engine = Text(route.RootElement, "engine"), system = Text(route.RootElement, "system");
                if (engine is null && system is null) return null;
                using var cores = JsonDocument.Parse(runtime.Cores());
                foreach (JsonElement c in cores.RootElement.EnumerateArray())
                {
                    bool waiting = engine is null && Text(c, "engine") is null && Texts(c, "systems").Contains(system);
                    if (!waiting && (engine is null || Text(c, "engine") != engine)) continue;
                    if (c.TryGetProperty("libretro", out var l) && l.ValueKind == JsonValueKind.Object)
                        return Text(l, "trust") == "needs_approval" && Text(c, "trust") != "needs_refresh" && Text(l, "sha256") is { } hash
                            ? new CoreAwaiting(Text(l, "file") ?? "", Text(l, "name") ?? Text(c, "name") ?? engine ?? "", Texts(l, "authors"), Text(l, "license") ?? "", Texts(c, "systems"), hash) : null;
                    if (waiting || Text(c, "trust") != "needs_approval") return null;
                    return new CoreAwaiting(System.IO.Path.GetFileName(Text(c, "library") ?? ""), Text(c, "name") ?? engine!, Texts(c, "authors"), Text(c, "license") ?? "", Texts(c, "systems"), Text(c, "sha256") ?? "");
                }
                return null;
            }
            catch (Exception e) when (e is CrystalException or JsonException or KeyNotFoundException or InvalidOperationException) { return null; }
        }

        private static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out var v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        private static string[] Texts(JsonElement e, string name) =>
            e.TryGetProperty(name, out var a) && a.ValueKind == JsonValueKind.Array ? a.EnumerateArray().Select(x => x.GetString() ?? "").ToArray() : Array.Empty<string>();

        // The words of the question, one form with Cores' (EmuSen_Cores.md §3.4).
        internal static string ApprovalMessage(CoreAwaiting core) =>
            EmuSen.Tools.Cores.CoreApproval.Message(core.Name, core.File, core.Authors, core.License, core.Systems, core.Sha256);
    }
}
