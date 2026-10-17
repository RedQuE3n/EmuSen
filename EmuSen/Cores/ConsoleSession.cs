namespace EmuSen.Cores
{
    // A console started with nothing in it has no game, so its states, locks and words go by this name - see Diamond_Shell.md §1.5.
    public static class ConsoleSession
    {
        public static string NameOf(string console) => $"{console} (no disc)";
    }
}
