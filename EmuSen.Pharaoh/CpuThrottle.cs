using System.Diagnostics;

namespace EmuSen.Pharaoh
{
    // Re-runs the harness inside a CPU-quota cgroup so a slow laptop can be measured on fast hardware - see §3.42.
    public static class CpuThrottle
    {
        public const string ActiveVariable = "EMUSEN_THROTTLE_ACTIVE";

        // Well under a 16.64ms frame, so throttling never lands as one long stall inside a frame - see §3.42.
        public const string QuotaPeriod = "5ms";

        public static bool AlreadyThrottled =>
            Environment.GetEnvironmentVariable(ActiveVariable) == "1";

        public static bool TryParsePercent(string[] args, out int percent, out string? error)
        {
            percent = 0;
            error = null;

            for (int i = 0; i < args.Length; i++)
            {
                if (args[i] != "--throttle") continue;
                if (i + 1 >= args.Length)
                {
                    error = "[ERROR] --throttle wants a percentage, e.g. --throttle 33.";
                    return false;
                }

                string raw = args[i + 1].TrimEnd('%');
                if (!int.TryParse(raw, out percent) || percent <= 0 || percent > 100)
                {
                    error = $"[ERROR] --throttle wants 1-100, got '{args[i + 1]}'.";
                    return false;
                }
                return true;
            }

            return true;
        }

        // Deliberately fatal rather than falling back to a full-speed run - see §3.42.
        public static int ReExec(int percent, string[] args)
        {
            if (!OperatingSystem.IsLinux())
            {
                Console.WriteLine("[ERROR] --throttle needs systemd cgroup quotas and is Linux-only.");
                return 1;
            }

            string? self = Environment.ProcessPath;
            if (self == null)
            {
                Console.WriteLine("[ERROR] --throttle could not determine its own executable path.");
                return 1;
            }

            var psi = new ProcessStartInfo("systemd-run") { UseShellExecute = false };
            foreach (string a in new[]
            {
                "--user", "--scope", "--quiet",
                "-p", $"CPUQuota={percent}%",
                "-p", $"CPUQuotaPeriodSec={QuotaPeriod}",
                "--", self,
            })
            {
                psi.ArgumentList.Add(a);
            }
            foreach (string a in args) psi.ArgumentList.Add(a);
            psi.Environment[ActiveVariable] = "1";

            Console.WriteLine($"[THROTTLE] Re-running under CPUQuota={percent}% (period {QuotaPeriod}); wall times are those of a ~{100.0 / percent:0.0}x slower machine.");

            try
            {
                using var child = Process.Start(psi);
                if (child == null)
                {
                    Console.WriteLine("[ERROR] --throttle could not start systemd-run.");
                    return 1;
                }
                child.WaitForExit();
                return child.ExitCode;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[ERROR] --throttle failed to start systemd-run: {ex.Message}");
                return 1;
            }
        }
    }
}
