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
    // The process-wide "who exists" directory - like a real /etc/passwd -
    // shared by every tmux session in this one process (see
    // DianaOSInterpreter.CurrentUser for the separate, per-shell "who's
    // ACTIVE right now" concept this backs). Same bare-static-class shape
    // as DianaOSLogging: no per-target/per-session scoping, just one
    // process-wide table.
    //
    // In-memory only, like every other piece of DianaOS state (bp/watch/
    // cheat all reset on process restart too) - no persistence mechanism
    // yet. Real persistence (surviving a relaunch) is future work, for
    // whenever a GUI frontend's own "add yourself as a player" flow
    // actually needs accounts to survive that long.
    //
    // Built for Unix-shell FLAVOR (whoami/who/su/useradd/userdel/passwd),
    // not real access control - see `man su`/`man useradd`. A password
    // hash is stored here so a future permission system has somewhere to
    // check against, but nothing in this codebase reads GetPasswordHash
    // for gating yet; `su` today switches identity unconditionally for
    // any existing account, same as every other command in this shell
    // trusts the local process rather than enforcing anything.
    public static class DianaOSUserRegistry
    {
        // root always exists and can't be removed - the one bedrock
        // account, same role uid 0 plays on a real system. No password by
        // default (null), matching every other account's default state.
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

        // SHA-256 of the raw string - same "good enough for flavor, not a
        // real security boundary" spirit as the rest of this shell (see
        // this file's own header comment). Real password hashing needs a
        // salt and a slow KDF (PBKDF2/bcrypt/Argon2); neither matters here
        // since nothing checks this hash against anything yet.
        private static string Hash(string password)
        {
            byte[] bytes = SHA256.HashData(Encoding.UTF8.GetBytes(password));
            var sb = new StringBuilder(bytes.Length * 2);
            foreach (byte b in bytes) sb.Append(b.ToString("x2"));
            return sb.ToString();
        }
    }
}
