using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.Crystal.Systems
{
    // The decision, and why: a system, the recognition that made it where one did, or none with words naming the candidates.
    public sealed record Routed(SystemEntry? System, Recognition? Recognition, string? Reason);

    // The one decision of which system a file runs as when several take its extension: their packs' recognisers, then the folder's ES-DE name, emusen-systems' - see EmuSen_CoreAPI_Extensions.md §8.4, EmuSen_Backend.md §13.3.
    public static class SystemRouting
    {
        private static readonly ConcurrentDictionary<(string Path, long Length, long Time, string Candidates), Routed> Cache = new();

        // Among candidates, each a system that takes the file's extension: the only one; else the one whose recogniser claims the file; else the one its folder is named for; else none.
        public static Routed Decide(string path, IReadOnlyList<SystemEntry> candidates, Func<IMediaProbe> probe)
        {
            if (candidates.Count < 2) return Decided(path, candidates, Array.Empty<(int, Recognition)>());
            var info = new FileInfo(path);
            var key = (info.FullName, info.Exists ? info.Length : -1, info.Exists ? info.LastWriteTimeUtc.Ticks : 0, string.Join(",", candidates.Select(c => c.Id)));
            return Cache.GetOrAdd(key, _ => Recognised(path, candidates, probe));
        }

        private static Routed Recognised(string path, IReadOnlyList<SystemEntry> candidates, Func<IMediaProbe> probe)
        {
            var claims = new List<(int At, Recognition Claim)>();
            try
            {
                var p = probe();
                for (int at = 0; at < candidates.Count; at++)
                    if (candidates[at].Recognise?.Invoke(p) is { } r) claims.Add((at, r));
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException or InvalidDataException or InvalidOperationException) { }
            return Decided(path, candidates, claims);
        }

        private static Routed Decided(string path, IReadOnlyList<SystemEntry> candidates, IReadOnlyList<(int At, Recognition Claim)> claims)
        {
            string name = Path.GetFileName(path);
            Func<string?> folder = () => Path.GetFileName(Path.GetDirectoryName(Path.GetFullPath(path)));
            bool asked = candidates.Count >= 2 && claims.Count != 1;
            return SystemCode.Route(name, asked ? folder() : null, candidates, claims);
        }
    }
}
