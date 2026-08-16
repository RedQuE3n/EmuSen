using System;
using System.Collections.Generic;
using System.Security.Cryptography;
using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Etc
{
    // Unix flavour, in memory only, with a hash nothing gates on - see `man su` and `man useradd`.
    public static class DianaOSUserRegistry
    {
        // root always exists and cannot be removed, as uid 0 cannot.
        private static readonly Dictionary<string, string?> _passwordHashes =
            new(StringComparer.OrdinalIgnoreCase) { ["root"] = null };

        public static IReadOnlyCollection<string> Users => _passwordHashes.Keys;

        public static bool Exists(string name) => _passwordHashes.ContainsKey(name);

        public static bool TryAdd(string name) => _passwordHashes.TryAdd(name, null);

        public static bool TryRemove(string name) =>
            !name.Equals("root", StringComparison.OrdinalIgnoreCase) && _passwordHashes.Remove(name);

        public static void SetPassword(string name, string password) =>
            _passwordHashes[name] = Hash(password);

        public static string? GetPasswordHash(string name) =>
            _passwordHashes.TryGetValue(name, out string? hash) ? hash : null;

        // Flavour, not a security boundary; a real hash would need a salt and a slow KDF.
        private static string Hash(string password)
        {
            byte[] bytes = SHA256.HashData(Encoding.UTF8.GetBytes(password));
            var sb = new StringBuilder(bytes.Length * 2);
            foreach (byte b in bytes) sb.Append(b.ToString("x2"));
            return sb.ToString();
        }
    }
}
