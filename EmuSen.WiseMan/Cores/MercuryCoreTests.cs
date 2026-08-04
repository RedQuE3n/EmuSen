using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // MercuryCore's ICore surface - see Mercury_Core.md.
    public class MercuryCoreTests : IDisposable
    {
        private readonly List<string> _temporaryFiles = new();

        public void Dispose()
        {
            foreach (string path in _temporaryFiles)
            {
                try { File.Delete(path); } catch (IOException) { }
            }
        }

        private MercuryCore Load(params (int Offset, byte[] Bytes)[] patches)
        {
            string path = SyntheticGbRom.WriteTemp(SyntheticGbRom.Build(patches: patches));
            _temporaryFiles.Add(path);

            var core = new MercuryCore();
            core.LoadRom(path);
            return core;
        }

        [Fact]
        public void The_core_reports_the_real_screen_and_refresh_rate()
        {
            var core = Load();

            Assert.Equal("GB", core.CoreName);
            Assert.Equal(160, core.ScreenWidth);
            Assert.Equal(144, core.ScreenHeight);
            Assert.InRange(core.FrameRateHz, 59.7, 59.8);
        }

        [Fact]
        public void Running_frames_advances_the_counter_and_the_clock()
        {
            // A tight JR -2 loop, so the frame ends on the cycle budget rather than falling off the end.
            var core = Load((0, new byte[] { 0x18, 0xFE }));

            for (int i = 0; i < 3; i++) core.RunFrame();

            Assert.Equal(3, core.TotalFrames);
            Assert.True(core.Cpu!.Cycles >= 3 * MercuryCore.CyclesPerFrame);
        }

        [Fact]
        public void The_frame_buffer_is_the_size_the_screen_advertises()
        {
            var core = Load();
            Assert.Equal(core.ScreenWidth * core.ScreenHeight * 4, core.GetFrameBufferRgba().Length);
        }

        [Fact]
        public void The_pad_is_the_eight_buttons_the_hardware_has()
        {
            var core = Load();

            Assert.Equal(8, core.SupportedButtons.Count);
            Assert.DoesNotContain(PadButton.L, core.SupportedButtons);
        }

        // A pressed button reads back as 0, and only the selected nibble answers.
        [Fact]
        public void A_pressed_button_pulls_its_line_low_in_the_selected_nibble()
        {
            var core = Load();
            core.SetButton(0, PadButton.Start, pressed: true);

            core.Bus!.Write(0xFF00, 0x10);   // select the action buttons
            Assert.Equal(0x00, core.Bus.Read(0xFF00) & 0x08);

            core.Bus.Write(0xFF00, 0x20);    // select the d-pad instead
            Assert.Equal(0x08, core.Bus.Read(0xFF00) & 0x08);
        }

        [Fact]
        public void The_timer_counts_and_raises_its_interrupt_on_overflow()
        {
            var core = Load();

            core.Bus!.Write(0xFF06, 0x00);   // TMA
            core.Bus.Write(0xFF05, 0xFF);    // TIMA, one tick from overflowing
            core.Bus.Write(0xFF07, 0x05);    // TAC: enabled, the fastest divider

            core.Bus.Tick(64);

            Assert.NotEqual(0, core.Bus.InterruptFlags & (1 << (int)Interrupt.Timer));
        }

        // Writing DIV zeroes the whole internal counter, not just the visible byte.
        [Fact]
        public void Writing_div_resets_the_counter()
        {
            var core = Load();

            core.Bus!.Tick(1024);
            Assert.NotEqual(0, core.Bus.Div);

            core.Bus.Write(0xFF04, 0x00);
            Assert.Equal(0, core.Bus.Div);
        }

        [Fact]
        public void Oam_dma_copies_a_page_into_sprite_memory()
        {
            var core = Load();

            for (int i = 0; i < 0xA0; i++) core.Bus!.Write((ushort)(0xC000 + i), (byte)(i ^ 0x5A));
            core.Bus!.Write(0xFF46, 0xC0);

            for (int i = 0; i < 0xA0; i++) Assert.Equal((byte)(i ^ 0x5A), core.Bus.Oam[i]);
        }

        [Fact]
        public void A_save_state_round_trips_through_a_stream()
        {
            var core = Load((0, new byte[] { 0x18, 0xFE }));
            core.RunFrame();
            core.Bus!.Write(0xC123, 0x99);

            using var stream = new MemoryStream();
            core.SaveState(stream);
            long framesAtSave = core.TotalFrames;

            core.RunFrame();
            core.Bus.Write(0xC123, 0x11);

            stream.Position = 0;
            core.LoadState(stream);

            Assert.Equal(framesAtSave, core.TotalFrames);
            Assert.Equal(0x99, core.Bus.Read(0xC123));
        }

        [Fact]
        public void A_breakpoint_halts_the_frame_and_resumes_past_it()
        {
            var core = Load((0, new byte[] { 0x00, 0x00, 0x18, 0xFE }));
            core.Breakpoints.AddBreakpoint(SyntheticGbRom.EntryPoint + 1);

            core.RunFrame();

            Assert.True(core.IsHaltedAtBreakpoint);
            Assert.Equal(SyntheticGbRom.EntryPoint + 1, core.HaltedAddress);

            // Resuming must not re-trigger on the instruction we are already stopped in front of.
            core.RunFrame();
            Assert.Equal(1, core.TotalFrames);
        }

        [Fact]
        public void Running_before_a_rom_is_loaded_is_an_error_rather_than_a_crash()
        {
            Assert.Throws<InvalidOperationException>(() => new MercuryCore().RunFrame());
        }
    }
}
