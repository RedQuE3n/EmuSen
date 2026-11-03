namespace EmuSen.Tools.Cores
{
    // The runtime's calls Cores makes, handed in by its host so the tool needs no binding of its own - see EmuSen_Cores.md §1 and §4.3.
    public sealed class CoresRuntime
    {
        // The runtime's `cores` listing as JSON.
        public required Func<string> Cores { get; init; }
        // The runtime's `systems` listing as JSON: each system's console and engines.
        public required Func<string> Systems { get; init; }
        // A console's stored setting, as `settings_get` answers.
        public required Func<string, string, string?> Setting { get; init; }
        // A console's setting stored, as `settings_set` stores it, the runtime's checks kept.
        public required Action<string, string, string?> SetSetting { get; init; }
        // `core_approve` and `core_revoke` of a library's SHA-256.
        public required Action<string> Approve { get; init; }
        public required Action<string> Revoke { get; init; }
        // `firmware_status` of a system id: its firmware each present, missing or not needed; null where the host has none (EmuSen_Cores.md §3.11).
        public Func<string, string>? Firmware { get; init; }
        // `rescan`: the cores directories listed again, after a job of Cores' changed them; null where the host has none.
        public Action? Rescan { get; init; }
    }

    // What the host opens for Cores: a console's settings with an engine named, the firmware pane and a folder.
    public sealed class CoresHost
    {
        public Action<string, string>? Options { get; init; }
        public Action? Firmware { get; init; }
        public Action<string>? OpenFolder { get; init; }
        // The approval question, answered by the player; a test answers it in place of a person.
        public Func<string, string, Task<bool>> Confirm { get; init; } = (_, _) => Task.FromResult(false);
        // A refusal said to the player, as a message over the window; null leaves it to the notice line.
        public Func<string, string, Task>? Tell { get; init; }
    }
}
