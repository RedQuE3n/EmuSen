using System;
using System.Reflection;
using System.Reflection.Emit;
using Xunit;
using CrystalContract = EmuSen.Crystal.Client.Contract.Crystal;

namespace EmuSen.WiseMan.Crystal
{
    // The binding's choice of implementation: EMUSEN_CRYSTAL_NATIVE when set, else the program's own default - see EmuSen_Backend.md §29.27.
    public class CrystalSwitchTests
    {
        // A program's assembly, carrying the key with this value, or none.
        private static Assembly Program(string? value, string key = "EmuSen.CrystalNative")
        {
            var name = new AssemblyName("CrystalSwitchProgram" + Guid.NewGuid().ToString("N"));
            CustomAttributeBuilder[] attributes = value is null ? [] :
                [new CustomAttributeBuilder(typeof(AssemblyMetadataAttribute).GetConstructor([typeof(string), typeof(string)])!, [key, value])];
            return AssemblyBuilder.DefineDynamicAssembly(name, AssemblyBuilderAccess.Run, attributes);
        }

        [Fact]
        public void The_variable_unset_and_the_program_s_metadata_give_the_rust()
        {
            Assert.True(CrystalContract.Asked(null, Program("1")));
            Assert.True(CrystalContract.Asked("", Program("1")));
        }

        [Fact]
        public void The_variable_0_overrides_the_metadata_to_the_csharp()
        {
            Assert.False(CrystalContract.Asked("0", Program("1")));
        }

        [Fact]
        public void The_variable_1_gives_the_rust_without_metadata()
        {
            Assert.True(CrystalContract.Asked("1", Program(null)));
            Assert.True(CrystalContract.Asked("1", null));
        }

        [Fact]
        public void No_metadata_and_no_variable_give_the_csharp()
        {
            Assert.False(CrystalContract.Asked(null, Program(null)));
            Assert.False(CrystalContract.Asked(null, null));
        }

        [Fact]
        public void Only_the_key_with_the_value_1_names_the_rust()
        {
            Assert.False(CrystalContract.Asked(null, Program("0")));
            Assert.False(CrystalContract.Asked(null, Program("true")));
            Assert.False(CrystalContract.Asked(null, Program("1", key: "EmuSen.Other")));
        }

        // Each program's own default, set by EmuSenCrystalNative in its project; Hotaru runs inside emusen-shell.
        [Theory]
        [InlineData("EmuSen.Pharaoh", true)]
        [InlineData("emusen-shell", false)]
        [InlineData("EmuSen.Mistress", false)]
        public void Each_program_names_its_default(string program, bool rust) => Assert.Equal(rust, CrystalContract.DefaultsToRust(Assembly.Load(program)));
    }
}
