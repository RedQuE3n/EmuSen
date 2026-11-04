using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using EmuSen.Galaxia;
using EmuSen.WiseMan.Crystal;

namespace EmuSen.WiseMan.Galaxia
{
    // The answers the C# gave, recorded before it was deleted, and the library's held to them case by case - see EmuSen_Backend.md §40.2.
    public partial class GalaxiaParityTests
    {
        private static readonly string Repo = ConfigRoot.Directory;
        private static readonly string Home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

        private readonly Dictionary<string, ParityGolden> _goldens = new(StringComparer.Ordinal);

        // The golden of a case: the test's name, and a theory's data after it.
        private string? _case;

        private ParityGolden Golden(string test)
        {
            string name = "GalaxiaParity." + (_case ?? test);
            if (!_goldens.TryGetValue(name, out ParityGolden? golden)) _goldens[name] = golden = new ParityGolden(name);
            return golden;
        }

        // A text with the test's folder, the checkout and the home folder written by name, so that the golden holds on any machine.
        private string Plain(string text) => text.Replace(_temp, "<root>").Replace(Repo, "<repo>").Replace(Home, "<home>");

        private static string AsText(object? value) => value switch
        {
            null => "null",
            string text => text,
            byte[] bytes => Convert.ToHexString(bytes),
            Exception fault => $"threw {fault.GetType().Name}: {fault.Message}",
            IEnumerable<string?> items => "[" + string.Join("|", items.Select(i => i ?? "null")) + "]",
            IEnumerable items => "[" + string.Join("|", items.Cast<object?>().Select(AsText)) + "]",
            _ => value.ToString() ?? "null",
        };

        // The library's answer, held to the C#'s recorded answer for the key; given back so a test can go on with it.
        private T G<T>(string key, T answer, [CallerMemberName] string test = "")
        {
            Golden(test).Add(Plain(key), Plain(AsText(answer)));
            return answer;
        }

        // A sweep's answer, held with the others of its group as one digest of up to size answers.
        private void GSweep(string group, string key, object? answer, int size = 64, [CallerMemberName] string test = "") =>
            Golden(test).AddChunked(group, Plain(key) + " = " + Plain(AsText(answer)), size);

        // What a call answered or threw, as one text.
        private static object? Caught(Func<object?> act)
        {
            try { return act(); }
            catch (Exception fault) { return fault; }
        }

        private void CheckGoldens()
        {
            foreach (ParityGolden golden in _goldens.Values) golden.Check();
            _goldens.Clear();
        }
    }
}
