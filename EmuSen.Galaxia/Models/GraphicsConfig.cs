using System.Collections.Generic;

namespace EmuSen.Galaxia.Models
{
    // On-disk mirror of EmuSen.Graphics.GraphicsSettings - see
    // EmuSen_Config_Reference.md §3.3. WindowTitle is deliberately absent:
    // it is branding, not a preference, and a corrupt config blanking the
    // title is a worse outcome than not being able to change it.
    public class GraphicsConfig
    {
        public int WindowWidth { get; set; } = 1060;
        public int WindowHeight { get; set; } = 580;
        public int TargetFps { get; set; } = 60;
        public bool VSyncEnabled { get; set; } = true;
        public bool WindowResizable { get; set; } = true;
        public bool BilinearFiltering { get; set; } = true;

        // Each console's own settings by key, in their text form, as the console's core reads and writes them - see EmuSen_Config_Reference.md §3.3.
        public Dictionary<string, Dictionary<string, string>> Consoles { get; set; } = new();

        public string? Value(string console, string key) =>
            Consoles.TryGetValue(console, out var settings) && settings.TryGetValue(key, out string? value) ? value : null;

        public void SetValue(string console, string key, string value)
        {
            if (!Consoles.TryGetValue(console, out var settings)) Consoles[console] = settings = new Dictionary<string, string>();
            settings[key] = value;
        }

        public void Forget(string console) => Consoles.Remove(console);

        // Each console's values for each shader's parameters, the shader keyed as its ScreenFilter value is, in text - see EmuSen_Config_Reference.md §3.3.
        public Dictionary<string, Dictionary<string, Dictionary<string, string>>> ShaderParameters { get; set; } = new();

        // The shaders last used on each console, newest first - see EmuSen_Config_Reference.md §3.3.
        public Dictionary<string, List<string>> RecentShaders { get; set; } = new();

        public IReadOnlyDictionary<string, string> ParametersFor(string console, string shader) =>
            ShaderParameters.TryGetValue(console, out var shaders) && shaders.TryGetValue(shader, out var values) ? values : new Dictionary<string, string>();

        public void SetParameter(string console, string shader, string id, string value)
        {
            if (!ShaderParameters.TryGetValue(console, out var shaders)) ShaderParameters[console] = shaders = new Dictionary<string, Dictionary<string, string>>();
            if (!shaders.TryGetValue(shader, out var values)) shaders[shader] = values = new Dictionary<string, string>();
            values[id] = value;
        }

        // An empty shader or console is removed with its last value, so a reset leaves nothing behind in the file.
        public void ForgetParameter(string console, string shader, string? id = null)
        {
            if (!ShaderParameters.TryGetValue(console, out var shaders) || !shaders.TryGetValue(shader, out var values)) return;
            if (id is null) values.Clear(); else values.Remove(id);
            if (values.Count == 0) shaders.Remove(shader);
            if (shaders.Count == 0) ShaderParameters.Remove(console);
        }

        public IReadOnlyList<string> RecentFor(string console) =>
            RecentShaders.TryGetValue(console, out var recent) ? recent : new List<string>();

        public void NoteRecent(string console, string shader, int keep)
        {
            if (!RecentShaders.TryGetValue(console, out var recent)) RecentShaders[console] = recent = new List<string>();
            recent.Remove(shader);
            recent.Insert(0, shader);
            if (recent.Count > keep) recent.RemoveRange(keep, recent.Count - keep);
        }

        private static readonly ConfigFile<GraphicsConfig> File = new("graphics.json");

        public bool Save() => File.Save(this);

        public static GraphicsConfig Load() => File.Load(() => new GraphicsConfig());

        public static bool Exists => File.Exists;
    }
}
