using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;

namespace EmuSen.Cores.Native
{
    // One library on the core ABI v1, loaded once from its path and checked before any machine is made; a failed check is never retried - see EmuSen_CoreAPI.md §7.2, §19.
    public sealed unsafe class CoreLibrary
    {
        // A descriptor larger than this is refused at load - see EmuSen_CoreAPI.md §7.2.
        public const int MaxDescriptorBytes = 1 << 20;

        private static readonly ConcurrentDictionary<string, CoreLibrary> Loaded = new(StringComparer.Ordinal);

        public string Path { get; }
        public string Report { get; private set; } = "";
        public bool Available { get; private set; }
        public CoreInterface Api { get; private set; } = null!;
        public uint AbiVersion { get; private set; }
        public ulong Capabilities { get; private set; }
        public CoreInfo Info { get; private set; } = null!;
        public string SettingsText { get; private set; } = "[]";
        public IReadOnlyList<CoreSettingDescriptor> Settings { get; private set; } = Array.Empty<CoreSettingDescriptor>();

        private nint _handle;

        private CoreLibrary(string path) => Path = path;

        // The library at path, loaded and checked the first time it is asked for; the same object, and the same verdict, every time after.
        public static CoreLibrary Open(string path) => Loaded.GetOrAdd(System.IO.Path.GetFullPath(path), full =>
        {
            var library = new CoreLibrary(full);
            library.Load();
            return library;
        });

        public bool Has(ulong capability) => (Capabilities & capability) == capability;

        // Whether this process has loaded the library at path.
        public static bool IsOpen(string path) => Loaded.ContainsKey(System.IO.Path.GetFullPath(path));

        private void Refuse(string why)
        {
            Available = false;
            Report = $"{System.IO.Path.GetFileName(Path)}: {why}";
        }

        // Reads a text the length-query way: a call with no buffer, then one with a buffer of that size.
        public static string? Text(delegate* unmanaged<byte*, nuint, long> call)
        {
            long n = call(null, 0);
            if (n < 0 || n > MaxDescriptorBytes) return null;
            var bytes = new byte[n];
            fixed (byte* b = bytes) if (call(b, (nuint)n) != n) return null;
            return Utf8(bytes);
        }

        // Strict UTF-8: descriptor text that is not is refused, never repaired.
        public static string? Utf8(byte[] bytes)
        {
            try { return new UTF8Encoding(false, true).GetString(bytes); }
            catch (DecoderFallbackException) { return null; }
        }

        private void Load()
        {
            if (!File.Exists(Path)) { Refuse("not found"); return; }
            if (!NativeLibrary.TryLoad(Path, out nint handle)) { Refuse("could not be loaded"); return; }
            _handle = handle;
            var api = new CoreInterface(handle);
            Api = api;
            if (api.AbiVersion == null) { Refuse("has no emusen_core_abi_version; it is not on the core ABI"); return; }
            AbiVersion = api.AbiVersion();
            if (AbiVersion >> 16 != CoreInterface.Major) { Refuse($"speaks core ABI major {AbiVersion >> 16}, this host major {CoreInterface.Major}"); return; }
            Capabilities = api.Capabilities != null ? api.Capabilities() : 0;
            foreach (var (name, bit) in CoreInterface.Exports)
            {
                bool present = NativeLibrary.TryGetExport(handle, name, out _);
                if (bit == 0 && !present) { Refuse($"lacks the required export {name}"); return; }
                if (bit != 0 && present != Has(bit)) { Refuse($"{(present ? "exports" : "lacks")} {name} but {(present ? "does not claim" : "claims")} its capability"); return; }
            }
            string? info = Text(api.Info);
            if (info is null) { Refuse("its info is not UTF-8 text under 1 MiB"); return; }
            try { Info = CoreDescriptorReader.Info(info); }
            catch (System.Text.Json.JsonException e) { Refuse($"its info is not JSON: {e.Message}"); return; }
            if (CheckInfo(Info) is { } wrong) { Refuse(wrong); return; }
            string? settings = Text(api.SettingsSchema);
            if (settings is null) { Refuse("its settings schema is not UTF-8 text under 1 MiB"); return; }
            try { Settings = CoreDescriptorReader.Settings(settings); }
            catch (System.Text.Json.JsonException e) { Refuse($"its settings schema is not JSON: {e.Message}"); return; }
            SettingsText = settings;
            SetCrashLog(api);
            Available = true;
            Report = $"{Path}, {Info.DisplayName} {Info.Version}, core ABI {AbiVersion >> 16}.{AbiVersion & 0xFFFF}";
        }

        // The checks of §7.2's step 4 that read the info: its abi, its capability names, its host obligations and its required fields.
        private string? CheckInfo(CoreInfo info)
        {
            string abi = $"{AbiVersion >> 16}.{AbiVersion & 0xFFFF}";
            if (info.Abi != abi) return $"its info says abi {info.Abi}, its export {abi}";
            if (info.Id.Length == 0 || info.Name.Length == 0 || info.Version.Length == 0 || info.License.Length == 0) return "its info lacks id, name, version or license";
            if (!info.Id.All(c => c is >= 'a' and <= 'z' or >= '0' and <= '9' or '-')) return $"its id {info.Id} is not [a-z0-9-]+";
            var claimed = CoreInterface.CapabilityNames.Where(c => (Capabilities & c.Bit) != 0).Select(c => c.Name).ToHashSet();
            var listed = info.Capabilities.Where(n => CoreInterface.CapabilityNames.Any(c => c.Name == n)).ToHashSet();
            if (!claimed.SetEquals(listed)) return $"its info lists the capabilities {string.Join(", ", listed.Order())}, its bits {string.Join(", ", claimed.Order())}";
            if (info.HostRequires.FirstOrDefault(r => !CoreInterface.HostObligations.Contains(r)) is { } unknown) return $"it requires the host obligation {unknown}, which this host does not know";
            return null;
        }

        private void SetCrashLog(CoreInterface api)
        {
            string log = System.IO.Path.Combine(EmuSen.Galaxia.Library.DataStore.Logs, $"{Info.Id}_crash_{Environment.ProcessId}.txt");
            nint text = Marshal.StringToCoTaskMemUTF8(log);
            api.SetCrashLog((byte*)text);
        }

        // Words for a status, from the core's status_text, with the number where it has none.
        public string Words(long status)
        {
            if (!Available || status < int.MinValue) return $"status {status}";
            long n = Api.StatusText((int)status, null, 0);
            if (n <= 0 || n > MaxDescriptorBytes) return $"status {status}";
            var bytes = new byte[n];
            fixed (byte* b = bytes) Api.StatusText((int)status, b, (nuint)n);
            return Utf8(bytes) ?? $"status {status}";
        }

        // What an image needs from the core, from its bytes alone.
        public IReadOnlyList<CoreFirmware> FirmwareFor(byte[] image)
        {
            if (!Available) return Array.Empty<CoreFirmware>();
            fixed (byte* img = image)
            {
                long n = Api.FirmwareFor(img, (nuint)image.Length, null, 0);
                if (n <= 0 || n > MaxDescriptorBytes) return Array.Empty<CoreFirmware>();
                var bytes = new byte[n];
                fixed (byte* b = bytes) Api.FirmwareFor(img, (nuint)image.Length, b, (nuint)n);
                return Utf8(bytes) is { } text ? CoreDescriptorReader.FirmwareList(text) : Array.Empty<CoreFirmware>();
            }
        }

        // One of the core's own extension exports by name, zero when the library is not in use or lacks it - see EmuSen_CoreAPI.md §4.7.
        public nint Export(string name) => Available && NativeLibrary.TryGetExport(_handle, name, out nint export) ? export : 0;

        // The records the library logged outside any machine, as lines.
        public IReadOnlyList<string> DrainLog() => Available ? CoreMachine.DrainLog(Api, 0) : Array.Empty<string>();
    }
}
