using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.WiseMan.Fixtures;
using Xunit;
using CrystalContract = EmuSen.Crystal.Client.Contract.Crystal;

namespace EmuSen.WiseMan.Crystal
{
    // The binding is the platform library's alone, and a leftover EMUSEN_CRYSTAL_NATIVE=0 is ignored with one line - see EmuSen_Backend.md §40.8.
    public class CrystalSwitchTests
    {
        [Fact]
        public void The_suite_s_binding_is_the_platform_library_s() => Assert.Contains($", frontend API {EmuSen.Crystal.Client.Contract.CrystalAbi.Major}.", CrystalContract.Report);

        // The line goes to the host's sink and the error log, so it is read from the binding rather than from a program's output.
        [Theory]
        [InlineData("0")]
        [InlineData("csharp")]
        public void A_leftover_variable_is_ignored_with_one_line(string value) =>
            Assert.StartsWith($"{CrystalContract.Variable}={value} is ignored: the frontend API's C# implementation was deleted, and ", CrystalContract.Ignored(value));

        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("1")]
        public void An_unset_variable_or_1_is_said_nothing_of(string? value) => Assert.Null(CrystalContract.Ignored(value));
    }
}
