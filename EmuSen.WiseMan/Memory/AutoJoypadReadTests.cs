using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Memory
{
    // $4212 bit 0 and the auto-joypad-read window it reports - see
    // Venus_Memory.md §4.4. The window is a pure function of scanline and
    // line cycles, so most of this needs no core at all.
    [Collection(TestCollections.ProcessGlobals)]
    public class AutoJoypadReadTests
    {
        private const int Vblank = InterruptController.AutoJoypadScanline;

        [Fact]
        public void Window_is_closed_during_active_display()
        {
            Assert.False(InterruptController.InAutoJoypadWindow(0, 0));
            Assert.False(InterruptController.InAutoJoypadWindow(100, 683));
            Assert.False(InterruptController.InAutoJoypadWindow(Vblank - 1, 1363));
        }

        [Fact]
        public void Window_has_not_opened_at_the_very_start_of_vblank()
        {
            // The read begins a couple hundred master clocks in, so a game
            // reading $4212 the instant NMI fires sees bit 0 still clear.
            Assert.False(InterruptController.InAutoJoypadWindow(Vblank, 0));
        }

        [Fact]
        public void Window_is_open_across_the_scanlines_after_vblank_starts()
        {
            Assert.True(InterruptController.InAutoJoypadWindow(Vblank, 300));
            Assert.True(InterruptController.InAutoJoypadWindow(Vblank + 1, 0));
            Assert.True(InterruptController.InAutoJoypadWindow(Vblank + 2, 0));
        }

        [Fact]
        public void Window_closes_partway_into_the_fourth_scanline()
        {
            // 4482 master clocks after vblank starts, i.e. 4482 - 3*1364 = 390
            // into scanline 228 - the point a waiting game gets to run again.
            Assert.True(InterruptController.InAutoJoypadWindow(Vblank + 3, 389));
            Assert.False(InterruptController.InAutoJoypadWindow(Vblank + 3, 390));
            Assert.False(InterruptController.InAutoJoypadWindow(Vblank + 4, 0));
        }

        [Fact]
        public void HVBJOY_reports_bit0_only_while_the_window_is_open()
        {
            var irq = new InterruptController { AutoJoypadEnabled = true };

            Assert.Equal(0x01, irq.ReadHVBJOY(inHBlank: false, inAutoJoypadWindow: true, lastBusValue: 0) & 0x01);
            Assert.Equal(0x00, irq.ReadHVBJOY(inHBlank: false, inAutoJoypadWindow: false, lastBusValue: 0) & 0x01);
        }

        [Fact]
        public void HVBJOY_never_reports_bit0_when_auto_read_is_disabled()
        {
            // $4200 bit 0 off means the read never runs, so the flag never
            // sets - a game polling for it to clear must not hang either way.
            var irq = new InterruptController { AutoJoypadEnabled = false };

            Assert.Equal(0x00, irq.ReadHVBJOY(inHBlank: false, inAutoJoypadWindow: true, lastBusValue: 0) & 0x01);
        }

        [Fact]
        public void HVBJOY_bit0_does_not_disturb_the_vblank_and_hblank_bits()
        {
            var irq = new InterruptController { AutoJoypadEnabled = true, InVBlank = true };

            byte val = irq.ReadHVBJOY(inHBlank: true, inAutoJoypadWindow: true, lastBusValue: 0);

            Assert.Equal(0x80, val & 0x80);
            Assert.Equal(0x40, val & 0x40);
            Assert.Equal(0x01, val & 0x01);
        }

        [Fact]
        public void Write4200_tracks_the_auto_joypad_enable_bit()
        {
            var irq = new InterruptController();

            irq.Write4200(0x81);
            Assert.True(irq.AutoJoypadEnabled);

            irq.Write4200(0x80);
            Assert.False(irq.AutoJoypadEnabled);
        }

        [Fact]
        public void A_rom_that_never_enables_auto_read_never_sees_a_latched_button()
        {
            // All-NOP ROM: nothing ever writes $4200, so $4218/$4219 stay put
            // however long the button is held - matching real hardware.
            var core = SyntheticRom.LoadCore(SyntheticRom.Build((0, new byte[] { 0xEA, 0xEA, 0xEA, 0xEA, 0xEA })));
            core.Bus!.Input.SetButton(SnesButton.Start, true);

            for (int i = 0; i < 3; i++) core.RunFrame();

            ushort latched = (ushort)((core.Bus.Input.ReadJoy1High() << 8) | core.Bus.Input.ReadJoy1Low());
            Assert.Equal(0, latched & 0x1000);
        }

        [Fact]
        public void A_rom_that_enables_auto_read_does_see_a_latched_button()
        {
            // Same ROM shape, but with SyntheticRom's default boot stub
            // ($4200 = $01) left in place.
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            core.Bus!.Input.SetButton(SnesButton.Start, true);

            core.RunFrame();

            ushort latched = (ushort)((core.Bus.Input.ReadJoy1High() << 8) | core.Bus.Input.ReadJoy1Low());
            Assert.NotEqual(0, latched & 0x1000);
        }

        [Fact]
        public void Auto_read_leaves_the_manual_shift_registers_spent()
        {
            // Super Mario All-Stars' $00:86F9 port-assignment routine reads
            // $4016/$4017 bare, with no strobe pulse of its own, and treats a
            // 0 as "this player is on the other port" - see Venus_Memory.md §4.4a.
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            core.Bus!.Input.SetButton(SnesButton.B, true);

            core.RunFrame();

            Assert.Equal(0x01, core.Bus.Input.ReadJoy1Serial() & 0x01);
            Assert.Equal(0x01, core.Bus.Input.ReadJoy2Serial() & 0x01);
        }

        [Fact]
        public void A_strobe_pulse_after_the_auto_read_still_returns_live_buttons()
        {
            // The spent state above must not outlive a game's own strobe -
            // manual polling re-seeds the shift register from live input.
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            core.Bus!.Input.SetButton(SnesButton.B, true);

            core.RunFrame();
            core.Bus.Input.WriteStrobe(0x01);
            core.Bus.Input.WriteStrobe(0x00);

            Assert.Equal(0x01, core.Bus.Input.ReadJoy1Serial() & 0x01); // B, the first bit out
            Assert.Equal(0x00, core.Bus.Input.ReadJoy1Serial() & 0x01); // Y, not held
        }
    }
}
