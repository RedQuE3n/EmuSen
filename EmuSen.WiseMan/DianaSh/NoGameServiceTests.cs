using System;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Crystal.Session;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;
using Xunit;
using CrystalContract = EmuSen.Crystal.Client.Contract.Crystal;

namespace EmuSen.WiseMan.DianaSh
{
    // A shell with no game over the runtime's table with no game (frontend API 1.12) answers as the C# provider over no target did - see EmuSen_Backend.md §40.7.
    [Collection(TestCollections.ProcessGlobals)]
    public sealed unsafe class NoGameServiceTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenNoGameService_" + Guid.NewGuid().ToString("N"));
        private readonly CrystalRuntime _runtime;

        public NoGameServiceTests()
        {
            string db = Path.Combine(_root, "Cheats");
            Directory.CreateDirectory(Path.Combine(db, "Nintendo - Super Nintendo Entertainment System"));
            File.WriteAllText(Path.Combine(db, "Nintendo - Super Nintendo Entertainment System", "Super Mario World (USA).cht"), Cht);
            File.WriteAllText(Path.Combine(_root, "import.cht"), Cht);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            new AppSettings { CheatDatabaseDirectory = db }.Save();
            _runtime = new CrystalRuntime();
        }

        public void Dispose()
        {
            _runtime.Dispose();
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private const string Cht = "cheats = 1\ncheat0_desc = \"Infinite Lives\"\ncheat0_code = \"7E0DBE09\"\ncheat0_enable = false\n";

        private const string NoRom = "Error: No ROM loaded - this command needs an active debug target.";

        // Each line and the C# provider's answer over no target, recorded from it on 2026-11-06: the cheat command needs a game whatever it asks.
        private static readonly (string Line, string Answer)[] Recorded =
        {
            ("cheat db", NoRom),
            ("cheat db status", NoRom),
            ("cheat db find mario", NoRom),
            ("cheat db load \"Super Mario World (USA)\"", NoRom),
            ("cheat files", NoRom),
            ("cheat import <root>/import.cht", NoRom),
            ("cheat db prune", NoRom),
            ("regs", NoRom),
            ("summary", "No target attached."),
            ("bp", "Usage: bp [<cpu>] add|list|remove ..."),
            ("watch", "Usage: watch add|list|log|clear|remove ..."),
            ("dump", "Usage: dump <space> <addr> <len> <file>"),
            ("disasm", "Usage: disasm <space>|<cpu> <addr> [<count>]"),
        };

        private string Line(string line) => line.Replace("<root>", _root, StringComparison.Ordinal);

        [Fact]
        public void A_shell_with_no_game_answers_as_the_csharp_provider_over_no_target_did()
        {
            DianaShInterpreter shell = DianaShInterpreter.CreateWithoutSession(_runtime.NoGameService());
            foreach (var (line, answer) in Recorded) Assert.Equal((line, answer), (line, shell.Execute(Line(line))));
            var (_, said, action) = shell.Submit("resume");
            Assert.Equal("Resuming.", said);
            Assert.IsType<HostAction.Resume>(action);
        }

        private static string Words() => Encoding.UTF8.GetString(CrystalRuntime.Bytes((o, n) => CrystalContract.LastError(0, o, n)));

        [Fact]
        public void A_caller_without_the_flag_is_answered_as_before_and_the_flag_names_no_session()
        {
            Assert.Equal(0, CrystalContract.SessionService(_runtime.Handle, 0, 0));
            Assert.Equal("no session is open", Words());
            Assert.Equal(0, CrystalContract.SessionService(_runtime.Handle, 0, CrystalAbi.ServiceReadOnly));
            Assert.Equal(0, CrystalContract.SessionService(_runtime.Handle, 0, 4));
            Assert.Equal($"service flags 0x4 are not known to 1.{CrystalAbi.Minor}", Words());
            Assert.Equal(0, CrystalContract.SessionService(_runtime.Handle, 3, CrystalAbi.ServiceNoGame));
            Assert.Equal("the table with no game names no session, and 3 was named", Words());

            nint table = _runtime.NoGameService();
            Assert.Equal(table, _runtime.NoGameService());
            Assert.NotEqual(table, _runtime.NoGameService(readOnly: true));
            var t = (EmuSenSessionV1*)table;
            var room = new byte[64];
            fixed (byte* r = room)
            {
                long n = t->sessions(t->ctx, r, (nuint)room.Length);
                Assert.Equal("[]", Encoding.UTF8.GetString(room, 0, (int)n));
                Assert.Equal(SessionAbi.NoSession, (int)t->info(t->ctx, 0, r, (nuint)room.Length));
            }
        }
    }
}
