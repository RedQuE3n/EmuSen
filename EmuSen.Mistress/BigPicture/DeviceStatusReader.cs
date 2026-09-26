using System;
using System.IO;
using System.Linq;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.BigPicture
{
    // The machine's battery and radios for the theme's systemstatus, read from Linux's sysfs; nothing is reported that sysfs does not show - see EmuSen_BigPicture.md §15.
    public static class DeviceStatusReader
    {
        public static DeviceStatus Read(string root = "/sys/class")
        {
            try
            {
                return new DeviceStatus(Bluetooth: Radio(root, "bluetooth"), Wifi: Wifi(root), BatteryPercent: Battery(root, out bool charging), Charging: charging);
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException)
            {
                return new DeviceStatus();
            }
        }

        private static string? Text(string path) => File.Exists(path) ? File.ReadAllText(path).Trim() : null;

        // The system's own battery: a mouse's or a pad's reports its scope as Device and is not the machine's.
        private static int? Battery(string root, out bool charging)
        {
            charging = false;
            string folder = Path.Combine(root, "power_supply");
            if (!Directory.Exists(folder)) return null;
            foreach (string supply in Directory.GetDirectories(folder).OrderBy(d => d, StringComparer.Ordinal))
            {
                if (Text(Path.Combine(supply, "type")) != "Battery" || Text(Path.Combine(supply, "scope")) == "Device") continue;
                if (!int.TryParse(Text(Path.Combine(supply, "capacity")), out int percent)) continue;
                charging = Text(Path.Combine(supply, "status")) is "Charging" or "Full";
                return Math.Clamp(percent, 0, 100);
            }
            return null;
        }

        // On when an interface with a wireless folder is up; null with none.
        private static bool? Wifi(string root)
        {
            string folder = Path.Combine(root, "net");
            if (!Directory.Exists(folder)) return null;
            string[] wireless = Directory.GetDirectories(folder).Where(d => Directory.Exists(Path.Combine(d, "wireless"))).ToArray();
            return wireless.Length == 0 ? null : wireless.Any(d => Text(Path.Combine(d, "operstate")) == "up");
        }

        // On when a switch of that type is blocked neither by software nor by hardware; null with none.
        private static bool? Radio(string root, string type)
        {
            string folder = Path.Combine(root, "rfkill");
            if (!Directory.Exists(folder)) return null;
            string[] switches = Directory.GetDirectories(folder).Where(d => Text(Path.Combine(d, "type")) == type).ToArray();
            return switches.Length == 0 ? null : switches.Any(d => Text(Path.Combine(d, "soft")) == "0" && Text(Path.Combine(d, "hard")) == "0");
        }
    }
}
