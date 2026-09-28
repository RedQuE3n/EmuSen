using System;
using System.IO;
using System.Linq;

namespace EmuSen.Mistress.Library
{
    // Where a game sits below its console's folder, as ES-DE's ROMs/<system>/<folder>/<file> - see EmuSen_Settings_Reference.md §4.67.
    public static class GameFolders
    {
        // The folders between the console folder (the ROM folder's first level) and the file, '/'-separated; "" at the console folder's top or outside the ROM folder.
        public static string Of(string? romDirectory, string path)
        {
            if (string.IsNullOrWhiteSpace(romDirectory) || Path.GetDirectoryName(path) is not { Length: > 0 } directory) return "";
            string relative = Path.GetRelativePath(Path.GetFullPath(romDirectory), Path.GetFullPath(directory));
            if (relative == "." || relative == ".." || relative.StartsWith(".." + Path.DirectorySeparatorChar, StringComparison.Ordinal) || Path.IsPathRooted(relative)) return "";
            string[] parts = relative.Split(Path.DirectorySeparatorChar, StringSplitOptions.RemoveEmptyEntries);
            return string.Join('/', parts.Skip(1));
        }

        // The folder that holds a path, one level up; "" at the top.
        public static string Parent(string folder) => folder.LastIndexOf('/') is var at and >= 0 ? folder[..at] : "";

        // The last part of a folder's path, which the list shows as its name.
        public static string Name(string folder) => folder[(folder.LastIndexOf('/') + 1)..];

        public static int Depth(string folder) => folder.Length == 0 ? 0 : folder.Count(c => c == '/') + 1;

        // The folder on disk that a game's folder path names, found by walking up from the game's own file.
        public static string OnDisk(string gameFile, string gameFolder, string folder)
        {
            string directory = Path.GetDirectoryName(gameFile) ?? "";
            for (int up = Depth(gameFolder) - Depth(folder); up > 0; up--) directory = Path.GetDirectoryName(directory) ?? directory;
            return directory;
        }

        // Where a folder path's child lies below another: null when the game is not inside it, "" when it is directly there, else the next folder's name.
        public static string? ChildWithin(string gameFolder, string folder)
        {
            if (folder.Length == 0) return gameFolder.Length == 0 ? "" : gameFolder.Split('/')[0];
            if (gameFolder == folder) return "";
            if (!gameFolder.StartsWith(folder + "/", StringComparison.Ordinal)) return null;
            return gameFolder[(folder.Length + 1)..].Split('/')[0];
        }
    }
}
