using System.Runtime.InteropServices;

namespace EmuSen.WiseMan.Audio
{
    // Sets an environment variable so a native P/Invoke'd library in THIS
    // process can actually see it - not the same guarantee as
    // Environment.SetEnvironmentVariable, which was directly verified (via
    // a P/Invoke'd getenv(3) call made immediately afterward) to update
    // .NET's own managed view without updating the real libc environment
    // this runtime's native interop reads, on Ubuntu's distro-packaged
    // dotnet-sdk-10.0 specifically - see AudioPlayerTests.cs for what that
    // broke and why (SDL kept selecting ALSA - absent in a sandboxed test
    // environment - instead of the "dummy" driver this suite needs).
    //
    // Platform-specific on purpose - there's no single portable native call
    // for this. Unix's libc setenv(3) and Windows' environment handling
    // are different APIs in different libraries with different semantics:
    //
    //   - Unix (Linux/macOS): P/Invoke setenv(3) directly. Verified
    //     working on Linux exactly as done here (see AudioPlayerTests.cs's
    //     own comment on how); macOS shares the same POSIX libc API and
    //     .NET's own DllImport("libc") resolution already handles the
    //     libSystem.dylib redirect Apple's platforms need for this exact
    //     import, so the same call is expected to work unchanged - not
    //     independently verified on real macOS hardware, since this
    //     project's sandboxed dev/CI environment is Linux-only.
    //   - Windows: SDL2's own SDL_getenv reads via the Win32
    //     GetEnvironmentVariable API directly, not a CRT-module-cached
    //     getenv() (the well-documented reason "managed code set an env
    //     var but a native DLL didn't see it" usually bites on Windows -
    //     each CRT module can carry its own cached copy). Environment.
    //     SetEnvironmentVariable already calls the real Win32
    //     SetEnvironmentVariableW under the hood on Windows, which is
    //     exactly what SDL reads - nothing to work around there. NOT
    //     independently verified on real Windows hardware for the same
    //     reason as the macOS note above. If this assumption ever turns
    //     out wrong, the fix is a kernel32 SetEnvironmentVariableW
    //     P/Invoke here, same shape as the Unix branch below - not a
    //     deeper redesign.
    internal static class NativeEnvironment
    {
        [DllImport("libc", EntryPoint = "setenv")]
        private static extern int UnixSetEnv(string name, string value, int overwrite);

        public static void Set(string name, string value)
        {
            if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                Environment.SetEnvironmentVariable(name, value);
            }
            else
            {
                UnixSetEnv(name, value, overwrite: 1);
            }
        }
    }
}
