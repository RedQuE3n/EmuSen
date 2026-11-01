namespace EmuSen.Tools.Cores
{
    // The approval question, one form for Mistress's open path and Cores: what the core says it is, its hash, and for a libretro core trust on first download - see EmuSen_Cores.md §5.2.
    public static class CoreApproval
    {
        public const string Title = "Approve a Core";

        // The words of the question, plain text: a core's descriptors are shown, never interpreted.
        public static string Message(string name, string file, IReadOnlyList<string> authors, string licence, IReadOnlyList<string> systems, string sha256, bool downloaded = false)
        {
            string said = $"{name}, the core in {file}, is not one this installation ships, and it has not been approved.\n\n"
                + $"By: {(authors.Count > 0 ? string.Join(", ", authors) : "not said")}\n"
                + $"Licence: {(licence.Length > 0 ? licence : "not said")}\n"
                + $"Systems: {(systems.Count > 0 ? string.Join(", ", systems) : "none")}\n"
                + $"SHA-256: {sha256}\n\n";
            if (downloaded)
                said += "libretro signs no build, so approving trusts the build the buildbot served when it was downloaded: trust on first download. The hash holds it to exactly that build, and EmuSen's own proxy runs it in a sandbox of its own.\n\n";
            return said + "A core runs with your full permissions. Approve it only if you trust where it came from. If the file changes, you will be asked again.";
        }
    }
}
