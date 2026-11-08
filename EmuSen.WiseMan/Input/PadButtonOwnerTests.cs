using System;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Input
{
    // PadButton and PadAxis held to emusen_core.h's enums, which the ABI check holds to emusen-native's Control and Axis, their one owner - see EmuSen_CoreAPI.md §6.4.
    public class PadButtonOwnerTests
    {
        private static string[] Header(string enumName, string prefix)
        {
            string header = File.ReadAllText(Path.Combine(ShaderCompiler.RepositoryRoot, "EmuSen", "Cores", "Shared", "emusen-native", "include", "emusen_core.h"));
            Match body = Regex.Match(header, $@"enum {enumName} \{{([^}}]*)\}};");
            Assert.True(body.Success, $"emusen_core.h has no enum {enumName}");
            return Regex.Matches(body.Groups[1].Value, $@"{prefix}(\w+) = (\d+)").OrderBy(m => int.Parse(m.Groups[2].Value)).Select(m => m.Groups[1].Value.Replace("_", "")).ToArray();
        }

        private static string[] Upper<T>() where T : struct, Enum => Enum.GetValues<T>().OrderBy(v => Convert.ToInt32(v)).Select(v => v.ToString().ToUpperInvariant()).ToArray();

        [Fact]
        public void The_buttons_are_the_headers_controls_in_their_order()
        {
            Assert.Equal(Header("emusen_control", "EMUSEN_CONTROL_"), Upper<PadButton>());
            Assert.Equal(Enumerable.Range(0, 16), Enum.GetValues<PadButton>().Select(v => (int)v));
        }

        [Fact]
        public void The_axes_are_the_headers_axes_in_their_order()
        {
            Assert.Equal(Header("emusen_axis", "EMUSEN_AXIS_"), Upper<PadAxis>());
        }
    }
}
