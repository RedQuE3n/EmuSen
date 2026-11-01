using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Text.Json;
using EmuSen.Galaxia.Native;

namespace EmuSen.Crystal.Systems
{
    // The libretro cores EmuSen hosts, libretro-cores.tsv as the platform library's emusen_systems_libretro_cores gives it; empty where the library cannot be read - see EmuSen_Cores.md §4.7, §4.8.
    public static unsafe class HostedCores
    {
        public sealed record Core(string Stem, IReadOnlyList<string> Systems, string Licence, string Renderer, IReadOnlyList<string> DefaultFor);

        private static readonly Lazy<IReadOnlyList<Core>> Loaded = new(Load);

        public static IReadOnlyList<Core> All => Loaded.Value;

        // The hosted core a system defaults to ahead of its native core, by stem; null where the native core is the default.
        public static string? DefaultFor(string? systemId) => systemId is null ? null : All.FirstOrDefault(c => c.DefaultFor.Contains(systemId))?.Stem;

        public static IReadOnlyList<Core> Parse(string json)
        {
            using var document = JsonDocument.Parse(json);
            static string[] Ids(JsonElement e, string key) => e.GetProperty(key).EnumerateArray().Select(s => s.GetString()!).ToArray();
            return document.RootElement.EnumerateArray().Select(c => new Core(c.GetProperty("stem").GetString()!, Ids(c, "systems"), c.GetProperty("licence").GetString()!,
                c.GetProperty("renderer").GetString()!, Ids(c, "default_for"))).ToArray();
        }

        private static IReadOnlyList<Core> Load()
        {
            try
            {
                if (!PlatformLibrary.Available) return Array.Empty<Core>();
                nint export = PlatformLibrary.Export("emusen_systems_libretro_cores");
                if (export == 0) return Array.Empty<Core>();
                var call = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)export;
                long length = call(null, 0);
                if (length < 0) return Array.Empty<Core>();
                byte[] bytes = new byte[length];
                fixed (byte* b = bytes) length = call(b, (nuint)bytes.Length);
                return Parse(Encoding.UTF8.GetString(bytes, 0, (int)Math.Max(0, length)));
            }
            catch (Exception e) when (e is JsonException or InvalidOperationException or KeyNotFoundException) { return Array.Empty<Core>(); }
        }
    }
}
