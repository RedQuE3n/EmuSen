using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Galaxia.Models;

namespace EmuSen.Galaxia.Library
{
    // How a checked folder setting stands: unset, usable, named but absent, or refused as relative or as a file - see EmuSen_Config_Reference.md §7.7.
    public enum FolderState { Unset, Usable, Missing, Relative, File }

    // A folder setting's value after the rules: the full path it names when it names one, and how it stands.
    public readonly record struct FolderValue(FolderState State, string? Path, string Written)
    {
        // Whether readers use the path: a usable folder, or one that is not there yet and so holds nothing.
        public bool Searched => State is FolderState.Usable or FolderState.Missing;

        // The sentence a page or a log says about a value that is not simply usable, or null.
        public string? Problem => State switch
        {
            FolderState.Relative => $"\"{Written}\" is not a full path, so it is not used. Type a full path, or one starting with ~.",
            FolderState.File => $"\"{Written}\" is a file, not a folder, so it is not used.",
            FolderState.Missing => $"The folder {Path} does not exist.",
            _ => null,
        };
    }

    // The folder each path setting names, or its default when blank; the one owner of that rule - see EmuSen_Config_Reference.md §7.
    public static class SettingFolders
    {
        public static string? Named(string? value) => string.IsNullOrWhiteSpace(value) ? null : value;

        public static string States(AppSettings settings) => Named(settings.StateDirectory) ?? DataStore.SaveStates;

        public static string Cheats(AppSettings settings) => Named(settings.CheatDatabaseDirectory) ?? DataStore.Cheats;

        public static string Artwork(AppSettings settings) => Named(settings.ArtworkDirectory) ?? DataStore.Artwork;

        public static string? Roms(AppSettings settings) => Named(settings.RomDirectory);

        // The folders firmware is looked for in, in order: the player's chosen folder when it can be used, then home/Firmware - see EmuSen_Firmware.md §2.3.
        public static IReadOnlyList<string> Firmware(AppSettings settings)
        {
            FolderValue chosen = Checked(settings.FirmwareDirectory);
            ReportOnce(nameof(AppSettings.FirmwareDirectory), chosen);
            return chosen is { Searched: true, Path: { } path } && !SameFolder(path, DataStore.Firmware) ? [path, DataStore.Firmware] : [DataStore.Firmware];
        }

        private static string? _reported;

        // A refused or absent value is said once through ConfigDiagnostics, and again only when it changes - see EmuSen_Config_Reference.md §7.7.
        private static void ReportOnce(string setting, FolderValue value)
        {
            string? key = value.Problem is null ? null : setting + "\n" + value.Written;
            if (key == _reported) return;
            _reported = key;
            if (value.Problem is { } problem) ConfigDiagnostics.Report($"appsettings.json: {setting}: {problem}");
        }

        private static bool SameFolder(string a, string b) =>
            string.Equals(System.IO.Path.TrimEndingDirectorySeparator(System.IO.Path.GetFullPath(a)), System.IO.Path.TrimEndingDirectorySeparator(System.IO.Path.GetFullPath(b)), OperatingSystem.IsLinux() ? StringComparison.Ordinal : StringComparison.OrdinalIgnoreCase);

        // A folder value by §7.7's rules: blank is unset, ~ and ~/ expand to the home, a relative path or a file is refused, an absent folder is said.
        public static FolderValue Checked(string? value)
        {
            if (Named(value) is not { } named) return new FolderValue(FolderState.Unset, null, value ?? "");
            string written = named.Trim();
            string path = Expanded(written);
            if (!System.IO.Path.IsPathFullyQualified(path)) return new FolderValue(FolderState.Relative, null, written);
            path = System.IO.Path.GetFullPath(path);
            if (File.Exists(path)) return new FolderValue(FolderState.File, null, written);
            return new FolderValue(Directory.Exists(path) ? FolderState.Usable : FolderState.Missing, path, written);
        }

        // ~ alone, or followed by a separator, is the player's home; ~name is not expanded and so stays relative.
        private static string Expanded(string value)
        {
            if (value != "~" && !value.StartsWith("~/", StringComparison.Ordinal) && !(OperatingSystem.IsWindows() && value.StartsWith("~\\", StringComparison.Ordinal))) return value;
            string home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            return value.Length == 1 ? home : System.IO.Path.Join(home, value[2..]);
        }
    }
}
