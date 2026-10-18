namespace EmuSen.Cores
{
    // The name a console started empty goes by, Galaxia's with the rest of the save library's names - see Diamond_Shell.md §1.5.
    public static class ConsoleSession
    {
        public static string NameOf(string console) => Galaxia.Library.SaveLibrary.ConsoleSessionName(console);
    }
}
