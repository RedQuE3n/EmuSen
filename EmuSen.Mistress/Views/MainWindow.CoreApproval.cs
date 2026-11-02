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

        // The runtime's questions before a launch, asked off the interface thread so a slow answer never freezes it (EmuSen_LibretroHost.md §10r); a test replaces them.
        internal static Func<string, CoreAwaiting?> AwaitingQuery = rom => CoreAwaitingApproval(Runtime, rom);
        internal static Func<string, string> FirmwareQuery = rom => Runtime.FirmwareNeeded(rom);
        internal static Func<string, IReadOnlyList<CardMedium>> CardsQuery = console => CardMedium.Of(Runtime, console);

        // A question run on a worker, its answer taken at once when it comes within a tenth of a second, else awaited with the interface free.
        internal static async Task<T> OffInterface<T>(Func<T> query)
        {
            Task<T> asked = Task.Run(query);
            ((IAsyncResult)asked).AsyncWaitHandle.WaitOne(TimeSpan.FromMilliseconds(100));
            return await asked;
        }

        // What the approval shows of a core library, read from the runtime's listing.
        internal sealed record CoreAwaiting(string File, string Name, IReadOnlyList<string> Authors, string License, IReadOnlyList<string> Systems, string Sha256)
        {
            // The engine the game opens on instead when the player says Not Now, where the route's engine is only the fallback for a waiting default (EmuSen_LibretroHost.md §10t).
            public string? Instead { get; init; }

            // Why the console's approved default could not be read, so the game opens on Instead with nothing to ask (EmuSen_LibretroHost.md §10v).
            public string? Held { get; init; }
        }

        // Shown once the game is on screen, saying why it opened on the fallback.
        private string? _engineFallbackNotice;

        // Asked when the engine the game would open on waits for the player's approval; approved, the runtime records it.
        internal async Task AskCoreApprovalAsync(string romPath)
        {
            _engineFallbackNotice = null;
            if (await OffInterface(() => AwaitingQuery(romPath)) is not { } core) return;
            if (core.Held is { } why)
            {
                StatusText.Text = _engineFallbackNotice = HeldNotice(core, core.Instead ?? "", why);
                return;
            }
            if (!await ConfirmCoreApproval(this, EmuSen.Tools.Cores.CoreApproval.Title, ApprovalMessage(core)))
            {
                if (core.Instead is { } instead) StatusText.Text = _engineFallbackNotice = FallbackNotice(core, instead);
                return;
            }
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
                if (route.RootElement.TryGetProperty("default_held", out var h) && h.ValueKind == JsonValueKind.Object && Text(h, "sha256") is { } kept)
                    return (Waiting(cores.RootElement, kept) ?? new CoreAwaiting(Text(h, "stem") ?? "", Text(h, "stem") ?? "", [], "", [], kept)) with { Instead = engine, Held = Text(h, "reason") ?? "" };
                if (route.RootElement.TryGetProperty("waiting_default", out var w) && w.ValueKind == JsonValueKind.Object && Text(w, "sha256") is { } named)
                    return Waiting(cores.RootElement, named) is { } first ? first with { Instead = engine } : null;
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

        // The listing's row of the libretro core whose hash the route names as the waiting default.
        private static CoreAwaiting? Waiting(JsonElement cores, string sha256)
        {
            foreach (JsonElement c in cores.EnumerateArray())
            {
                if (!c.TryGetProperty("libretro", out var l) || l.ValueKind != JsonValueKind.Object || !string.Equals(Text(l, "sha256"), sha256, StringComparison.OrdinalIgnoreCase)) continue;
                return new CoreAwaiting(Text(l, "file") ?? "", Text(l, "name") ?? Text(c, "name") ?? "", Texts(l, "authors"), Text(l, "license") ?? "", Texts(c, "systems"), sha256);
            }
            return null;
        }

        // The one line saying a game opened on the fallback because its console's default waits for approval.
        internal static string FallbackNotice(CoreAwaiting core, string instead) => $"{core.Name} waits for your approval, so this game opens on {instead}.";

        // The line saying a game opened on the fallback because its console's approved default could not be read, and why.
        internal static string HeldNotice(CoreAwaiting core, string instead, string why) => $"{core.Name} could not be used, so this game opens on {instead}: {why}";

        private static string? Text(JsonElement e, string name) => e.TryGetProperty(name, out var v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        private static string[] Texts(JsonElement e, string name) =>
            e.TryGetProperty(name, out var a) && a.ValueKind == JsonValueKind.Array ? a.EnumerateArray().Select(x => x.GetString() ?? "").ToArray() : Array.Empty<string>();

        // The words of the question, one form with Cores' (EmuSen_Cores.md §3.4).
        internal static string ApprovalMessage(CoreAwaiting core) =>
            EmuSen.Tools.Cores.CoreApproval.Message(core.Name, core.File, core.Authors, core.License, core.Systems, core.Sha256);
    }
}
