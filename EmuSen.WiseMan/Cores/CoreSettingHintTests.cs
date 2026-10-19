using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Every discovered core's settings as the settings window words them, the schema's cost after the core's help - see EmuSen_CoreAPI.md §6.13 and EmuSen_Backend.md §26.9.
    [Collection(TestCollections.ProcessGlobals)]
    public class CoreSettingHintTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSettingHints_" + Guid.NewGuid().ToString("N"));

        public CoreSettingHintTests()
        {
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(true);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // A hint is the help, then the schema's cost: a cost sharing a run of five words with the help says it twice, as does any sentence written twice.
        [Fact]
        public void No_setting_s_hint_says_a_sentence_twice()
        {
            Assert.NotNull(CoreDiscovery.Found.Single(c => c.Info.Id == "marsrt").Open());
            var repeated = (from core in CoreDiscovery.Found
                            let library = core.Open()
                            where library is not null
                            from s in library.Settings
                            let hint = s.AsCoreSetting()?.Hint ?? (s.Cost is null ? s.Help : $"{s.Help} {s.Cost}")
                            from twice in Runs(s.Help).Intersect(Runs(s.Cost ?? "")).Concat(Sentences(hint).GroupBy(t => t).Where(g => g.Count() > 1).Select(g => g.Key))
                            select $"{core.Info.Id} {s.Key}: \"{twice}\"").ToArray();
            Assert.True(repeated.Length == 0, "a hint says a sentence twice: " + string.Join("; ", repeated));
        }

        private static string[] Sentences(string text) => Regex.Split(text, @"(?<=[.!?])\s+").Where(t => t.Length > 0).ToArray();

        // Every run of five words in the text, ignoring case and punctuation.
        private static IEnumerable<string> Runs(string text)
        {
            string[] words = Regex.Matches(text.ToLowerInvariant(), @"[\p{L}\p{N}']+").Select(m => m.Value).ToArray();
            for (int w = 0; w + 5 <= words.Length; w++) yield return string.Join(' ', words, w, 5);
        }
    }
}
