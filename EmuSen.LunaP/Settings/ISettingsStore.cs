namespace EmuSen.LunaP.Settings
{
    // The seam between LunaP and whatever a host keeps its settings in - see EmuSen_LunaP.md §19.
    public interface ISettingsStore
    {
        // A category is a subdirectory; null is the root.
        string Directory(string? category);

        // Null for missing, unreadable or corrupt - callers fall back to their own defaults rather than crash.
        T? Load<T>(string? category, string fileName) where T : class;

        // False when the write failed; a setting that cannot reach the disk must not take the program with it.
        bool Save<T>(string? category, string fileName, T value) where T : class;
    }
}
