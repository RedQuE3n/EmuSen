using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.Crystal.Systems
{
    // The decision, and why: a system, the recognition that made it where one did, or none with words naming the candidates.
    public sealed record Routed(SystemEntry? System, Recognition? Recognition, string? Reason);

    // The one decision of which system a file runs as when several take its extension: their packs' recognisers, then the folder's ES-DE name - see EmuSen_CoreAPI_Extensions.md §8.4.
    public static class SystemRouting
    {
        private static readonly ConcurrentDictionary<(string Path, long Length, long Time, string Candidates), Routed> Decided = new();

        // Among candidates, each a system that takes the file's extension: the only one; else the one whose recogniser claims the file; else the one its folder is named for; else none.
        public static Routed Decide(string path, IReadOnlyList<SystemEntry> candidates, Func<IMediaProbe> probe)
        {
            if (candidates.Count == 1) return new Routed(candidates[0], null, null);
            if (candidates.Count == 0) return new Routed(null, null, $"no system takes {Path.GetFileName(path)}");
            var info = new FileInfo(path);
            var key = (info.FullName, info.Exists ? info.Length : -1, info.Exists ? info.LastWriteTimeUtc.Ticks : 0, string.Join(",", candidates.Select(c => c.Id)));
            return Decided.GetOrAdd(key, _ => Recognised(path, candidates, probe));
        }

        private static Routed Recognised(string path, IReadOnlyList<SystemEntry> candidates, Func<IMediaProbe> probe)
        {
            var claims = new List<(SystemEntry System, Recognition Recognition)>();
            try
            {
                var p = probe();
                foreach (var c in candidates)
                    if (c.Recognise?.Invoke(p) is { } r) claims.Add((c, r));
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException or InvalidDataException or InvalidOperationException) { }
            if (claims.Count == 1) return new Routed(claims[0].System, claims[0].Recognition, null);
            string? folder = Path.GetFileName(Path.GetDirectoryName(Path.GetFullPath(path)));
            var named = candidates.Where(c => string.Equals(c.EsdeSystem, folder, StringComparison.OrdinalIgnoreCase)).ToArray();
            if (named.Length == 1) return new Routed(named[0], claims.FirstOrDefault(c => c.System == named[0]).Recognition, null);
            string which = string.Join(" or ", candidates.Select(c => c.Name));
            return new Routed(null, null, $"{Path.GetFileName(path)} could be {which}: {(claims.Count == 0 ? "no console recognises its content" : "more than one console recognises its content")}, and its folder is named for none; move it into a folder named {string.Join(" or ", candidates.Select(c => c.EsdeSystem))}");
        }
    }
}
