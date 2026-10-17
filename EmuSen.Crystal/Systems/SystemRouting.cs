using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.Crystal.Systems
{
    // The decision, and why: a system, the recognition that made it where one did, or none with words naming the candidates.
    public sealed record Routed(SystemEntry? System, Recognition? Recognition, string? Reason);

    // The one decision of which system a file runs as when several take its extension: their packs' recognisers, then the folder's ES-DE name; emusen-systems decides unless EMUSEN_SYSTEMS_NATIVE=0 - see EmuSen_CoreAPI_Extensions.md §8.4, EmuSen_Backend.md §13.3.
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
            return SystemCode.Active && SystemCode.Route(name, asked ? folder() : null, candidates, claims) is { } routed ? routed : DecideManaged(name, folder, candidates, claims);
        }

        // The C# decision from the claims, each by its candidate's place, the folder asked for only where it decides: the failover and the parity class's oracle.
        public static Routed DecideManaged(string fileName, Func<string?> folder, IReadOnlyList<SystemEntry> candidates, IReadOnlyList<(int At, Recognition Claim)> claims)
        {
            if (candidates.Count == 1) return new Routed(candidates[0], null, null);
            if (candidates.Count == 0) return new Routed(null, null, $"no system takes {fileName}");
            if (claims.Count == 1) return new Routed(candidates[claims[0].At], claims[0].Claim, null);
            string? name = folder();
            var named = Enumerable.Range(0, candidates.Count).Where(i => string.Equals(candidates[i].EsdeSystem, name, StringComparison.OrdinalIgnoreCase)).ToArray();
            if (named.Length == 1) return new Routed(candidates[named[0]], claims.FirstOrDefault(c => c.At == named[0]).Claim, null);
            string which = string.Join(" or ", candidates.Select(c => c.Name));
            return new Routed(null, null, $"{fileName} could be {which}: {(claims.Count == 0 ? "no console recognises its content" : "more than one console recognises its content")}, and its folder is named for none; move it into a folder named {string.Join(" or ", candidates.Select(c => c.EsdeSystem))}");
        }
    }
}
