namespace EmuSen.Tools.Cores
{
    // The approval question, one form for Mistress's open path and Cores: what the core says it is, its hash, and for a libretro core trust on first download - see EmuSen_Cores.md §5.2.
    public static class CoreApproval
    {
        public const string Title = "Approve a Core";

        // The words of the question in the register of EmuSen_Interface.md §15.6, plain text: a core's descriptors are shown, never interpreted.
        public static string Message(string name, string file, IReadOnlyList<string> authors, string licence, IReadOnlyList<string> systems, string sha256, bool downloaded = false, string? from = null, string? when = null, string? tested = null) =>
            Sentences(name, file, authors, licence, systems, downloaded, from, when) + "\n\n" + SmallPrint(sha256, tested);

        // The question's sentences, without the small print.
        public static string Sentences(string name, string file, IReadOnlyList<string> authors, string licence, IReadOnlyList<string> systems, bool downloaded = false, string? from = null, string? when = null)
        {
            string by = $"By {(authors.Count > 0 ? string.Join(", ", authors) : "authors not said")}, under {(licence.Length > 0 ? "the " + licence : "a licence not said")}; it runs {(systems.Count > 0 ? string.Join(" and ", systems) + " games" : "games of no console EmuSen knows")}.";
            string said = downloaded
                ? $"{name} was downloaded from {from ?? "libretro's buildbot"} {when ?? "earlier"} and has not been approved. {by}\n\n"
                    + "libretro signs no build, so approving trusts the build the buildbot served that day. The hash below holds EmuSen to exactly that build, and EmuSen's own proxy runs it in a sandbox. "
                    + "It runs with your permissions: approve it only if you trust where it came from. If the file changes, you will be asked again."
                : $"{name}, in {file}, did not ship with EmuSen and has not been approved. {by}\n\n"
                    + "It runs with your full permissions: approve it only if you trust where it came from. If the file changes, you will be asked again.";
            return said;
        }

        // What the project found of the build, then the hash: the question's small print, in Muted Caption where the form draws it so.
        public static string SmallPrint(string sha256, string? tested = null) => (tested is { Length: > 0 } ? tested + "\n\n" : "") + "SHA-256\n" + Grouped(sha256);

        // The hash in groups of eight, four to a line.
        private static string Grouped(string sha) => string.Join("\n", Enumerable.Range(0, (sha.Length + 31) / 32).Select(l => string.Join(" ", Enumerable.Range(0, 4).Select(g => l * 32 + g * 8).Where(at => at < sha.Length).Select(at => sha.Substring(at, Math.Min(8, sha.Length - at))))));
    }
}
