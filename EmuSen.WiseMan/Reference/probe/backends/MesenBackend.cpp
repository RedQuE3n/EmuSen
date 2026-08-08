#include "pch.h"

#include "MesenBackend.h"

#include <atomic>
#include <chrono>
#include <cstdlib>
#include <thread>

#include "Shared/Emulator.h"
#include "Shared/EmulatorLock.h"
#include "Shared/EmuSettings.h"
#include "Shared/KeyManager.h"
#include "Shared/SettingTypes.h"
#include "Shared/Interfaces/IKeyManager.h"
#include "Shared/MemoryType.h"
#include "Shared/Video/VideoDecoder.h"
#include "Shared/Audio/SoundMixer.h"
#include "SNES/SnesConsole.h"
#include "SNES/SnesPpu.h"
#include "SNES/BaseCartridge.h"
#include "SNES/Coprocessors/GSU/Gsu.h"
#include "SNES/Coprocessors/GSU/GsuTypes.h"
#include "NES/NesConsole.h"
#include "NES/BaseMapper.h"
#include "NES/BaseNesPpu.h"
#include "NES/APU/NesApu.h"
#include "NES/NesTypes.h"
#include "Utilities/VirtualFile.h"
#include "Utilities/FolderUtilities.h"

// Supplied by the patches under Reference/patches/mesen; stock Mesen has no
// trace hook for any of these.
extern bool g_gsuTraceOn;
extern std::vector<uint8_t> g_gsuTrace;
extern bool g_cpuTraceOn;
extern std::vector<uint8_t> g_cpuTrace;
extern bool g_nesApuTraceOn;
extern std::vector<uint8_t> g_nesApuTrace;

// Supplied by probe-frame-stop.patch: the emulation thread stops itself here.
extern std::atomic<uint32_t> g_probeStopFrame;

namespace
{
	// Synthetic scan codes, because a headless Mesen has no keyboard and the
	// stock config carries no binding at all - port 1 would read as idle
	// whatever the schedule said.
	uint16_t ScanCode(ProbeButton button) { return (uint16_t)button + 1; }

	// Mesen drives input through a registered key manager, so headless needs one.
	// The schedule is resolved against GetFrameCount() *here*, at the moment the
	// pad is polled, rather than by the report loop: the probe runs at maximum
	// speed and free-runs between reports, so anything sampled out there would
	// skip whole presses. Live is the --pressuntil override, written from the
	// control thread and read from the emulation thread.
	class ScheduledKeyManager : public IKeyManager
	{
	public:
		const InputSchedule* Schedule = nullptr;
		Emulator* Emu = nullptr;
		std::atomic<uint32_t> Live{0};

		void RefreshState() {}
		void UpdateDevices() {}
		bool IsMouseButtonPressed(MouseButton button) { (void)button; return false; }

		bool IsKeyPressed(uint16_t keyCode)
		{
			if(keyCode == 0 || keyCode > (uint16_t)ProbeButton::Count) { return false; }
			if(Live.load(std::memory_order_relaxed) & (1u << (keyCode - 1))) { return true; }
			return Schedule && Emu && Schedule->HeldAt(Emu->GetFrameCount(), (ProbeButton)(keyCode - 1));
		}

		vector<uint16_t> GetPressedKeys()
		{
			vector<uint16_t> keys;
			for(uint16_t i = 1; i <= (uint16_t)ProbeButton::Count; i++) {
				if(IsKeyPressed(i)) { keys.push_back(i); }
			}
			return keys;
		}

		string GetKeyName(uint16_t keyCode) { return std::to_string(keyCode); }
		uint16_t GetKeyCode(string keyName) { return (uint16_t)atoi(keyName.c_str()); }
		bool SetKeyState(uint16_t scanCode, bool state) { (void)scanCode; (void)state; return false; }
		void ResetKeyState() { Live.store(0); }
		void SetDisabled(bool disabled) { (void)disabled; }
	};

	class MesenBackend : public IProbeBackend
	{
	public:
		const char* Name() const override { return "mesen"; }
		const char* System() const override { return _nes ? "nes" : "snes"; }
		ProbeIdentity Identity() override;
		const char* AnchorSpace() const override { return _nes ? "ram" : "gsuram"; }

		bool Load(const std::string& romPath, const ProbeOptions& options) override
		{
			// Before the Emulator exists, which is what reads it.
			if(!options.HomeFolder.empty()) { FolderUtilities::SetHomeFolder(options.HomeFolder); }

			_romPath = romPath;
			_keys.reset(new ScheduledKeyManager());
			_emu.reset(new Emulator());
			_keys->Emu = _emu.get();
			_keys->Schedule = &_schedule;

			KeyManager::RegisterKeyManager(_keys.get());
			KeyManager::SetSettings(_emu->GetSettings());

			_emu->Initialize();
			_emu->GetSettings()->SetFlag(EmulationFlags::MaximumSpeed);

			RamState ramState = options.RamState == "ones" ? RamState::AllOnes
			                  : options.RamState == "random" ? RamState::Random
			                  : RamState::AllZeros;

			// Mesen's own default is a random power-on fill, which makes it
			// disagree with itself run to run - measured, not assumed. See §3.40.
			SnesConfig scfg = _emu->GetSettings()->GetSnesConfig();
			scfg.RamPowerOnState = ramState;
			scfg.Port1.Type = ControllerType::SnesController;
			KeyMapping& smap = scfg.Port1.Keys.Mapping1;
			smap.A = ScanCode(ProbeButton::A);         smap.B = ScanCode(ProbeButton::B);
			smap.X = ScanCode(ProbeButton::X);         smap.Y = ScanCode(ProbeButton::Y);
			smap.L = ScanCode(ProbeButton::L);         smap.R = ScanCode(ProbeButton::R);
			smap.Up = ScanCode(ProbeButton::Up);       smap.Down = ScanCode(ProbeButton::Down);
			smap.Left = ScanCode(ProbeButton::Left);   smap.Right = ScanCode(ProbeButton::Right);
			smap.Start = ScanCode(ProbeButton::Start); smap.Select = ScanCode(ProbeButton::Select);
			_emu->GetSettings()->SetSnesConfig(scfg);

			NesConfig ncfg = _emu->GetSettings()->GetNesConfig();
			ncfg.RamPowerOnState = ramState;
			// A headless NesConfig has every ChannelVolumes[] entry at zero, which
			// mutes the mixer no matter what the 2A03 does - see §3.43.
			for(uint32_t& volume : ncfg.ChannelVolumes) { volume = 100; }
			ncfg.Port1.Type = ControllerType::NesController;
			KeyMapping& nmap = ncfg.Port1.Keys.Mapping1;
			nmap.A = ScanCode(ProbeButton::A);         nmap.B = ScanCode(ProbeButton::B);
			nmap.Up = ScanCode(ProbeButton::Up);       nmap.Down = ScanCode(ProbeButton::Down);
			nmap.Left = ScanCode(ProbeButton::Left);   nmap.Right = ScanCode(ProbeButton::Right);
			nmap.Start = ScanCode(ProbeButton::Start); nmap.Select = ScanCode(ProbeButton::Select);
			_emu->GetSettings()->SetNesConfig(ncfg);

			if(!_emu->LoadRom((VirtualFile)romPath, VirtualFile())) { return false; }

			// After LoadRom, which rebuilds the mixer.
			if(!options.WavPath.empty()) {
				_emu->GetSoundMixer()->StartRecording(options.WavPath);
				printf("[INFO] recording audio to %s\n", options.WavPath.c_str());
			}
			_recording = !options.WavPath.empty();

			_snes = dynamic_cast<SnesConsole*>(_emu->GetConsole().get());
			_nes = dynamic_cast<NesConsole*>(_emu->GetConsole().get());
			_gsu = _snes ? _snes->GetCartridge()->GetGsu() : nullptr;
			CacheSpaces();
			return true;
		}

		void Shutdown() override
		{
			if(!_emu) { return; }
			// Resume first, or Stop waits on a thread parked in WaitForPauseEnd.
			g_probeStopFrame.store(0);
			_emu->Resume();
			if(_recording) { _emu->GetSoundMixer()->StopRecording(); }
			_emu->Stop(false);
			_emu->Release();
		}

		// Free-runs at maximum speed and stops *on* the requested frame, because
		// the emulation thread stops itself there (probe-frame-stop.patch).
		//
		// The two obvious ways both fail, and both were measured. Pause() only
		// sets _paused, which IsPaused() reads straight back, so "pause then wait
		// for IsPaused" returns immediately and every dump is read out of a
		// running machine - two runs of the old probe differ by 22 bytes of NES
		// RAM at frame 180. AcquireLock() does genuinely park the thread, but
		// only once the control thread has noticed the frame went by, and at max
		// speed that notice arrives two to five frames late.
		void RunUntil(uint32_t frame) override
		{
			if(_emu->GetFrameCount() >= frame) { return; }
			g_probeStopFrame.store(frame);
			_emu->Resume();
			while(!_emu->IsPaused() || _emu->GetFrameCount() < frame) {
				std::this_thread::sleep_for(std::chrono::milliseconds(1));
			}
		}

		uint32_t FrameCount() override { return _emu->GetFrameCount(); }
		std::vector<MemorySpace> Spaces() override { return _spaces; }

		void SetSchedule(const InputSchedule& schedule) override { _schedule = schedule; }

		void SetButton(ProbeButton button, bool held) override
		{
			uint32_t bit = 1u << (ScanCode(button) - 1);
			if(held) { _keys->Live.fetch_or(bit); } else { _keys->Live.fetch_and(~bit); }
		}

		bool Screen(ScreenView& out) override
		{
			if(_nes) {
				// Raw palette indices, one uint16 per pixel - not RGB. See §3.43.
				out = { _nes->GetPpu()->GetScreenBuffer(true), 256, 240, 256 * 240 * 2, ScreenFormat::PaletteIndex16 };
				return true;
			}
			if(_snes) {
				// Raw BGR555; the row stride is the reported width, not 512.
				out = { _snes->GetPpu()->GetScreenBuffer(), 512, 478, 512 * 478 * 2, ScreenFormat::Bgr555 };
				return true;
			}
			return false;
		}

		std::string StateLine() override;

		bool BeginTrace(TraceKind kind) override
		{
			switch(kind) {
				case TraceKind::Cpu: g_cpuTrace.reserve(64u * 1024 * 1024); g_cpuTraceOn = true; return true;
				case TraceKind::ApuWrites: g_nesApuTrace.reserve(16u * 1024 * 1024); g_nesApuTraceOn = true; return true;
				case TraceKind::Gsu: g_gsuTraceOn = true; return true;
			}
			return false;
		}

		// out is cleared first: a caller reusing one buffer across kinds would
		// otherwise swap the previous trace back into the global and ship it.
		bool EndTrace(TraceKind kind, std::vector<uint8_t>& out) override
		{
			out.clear();
			switch(kind) {
				case TraceKind::Cpu: g_cpuTraceOn = false; out.swap(g_cpuTrace); return true;
				case TraceKind::ApuWrites: g_nesApuTraceOn = false; out.swap(g_nesApuTrace); return true;
				case TraceKind::Gsu: g_gsuTraceOn = false; out.swap(g_gsuTrace); return true;
			}
			return false;
		}

	private:
		void CacheSpaces();
		void Add(const char* name, MemoryType type);

		std::unique_ptr<Emulator> _emu;
		std::unique_ptr<ScheduledKeyManager> _keys;
		InputSchedule _schedule;
		std::vector<MemorySpace> _spaces;
		std::string _romPath;
		SnesConsole* _snes = nullptr;
		NesConsole* _nes = nullptr;
		Gsu* _gsu = nullptr;
		bool _recording = false;
	};

	// The mapper id here is the one the emulator *settled on*, not the one in the
	// file's header - this reference overrides a damaged header from a CRC-keyed
	// game database, and that difference is exactly what a comparability gate has
	// to see. See EmuSen_Debugging_Tools_Reference_v5.md §3.48.
	ProbeIdentity MesenBackend::Identity()
	{
		ProbeIdentity id;
		if(!_emu) { return id; }

		switch(_emu->GetRegion()) {
			case ConsoleRegion::Ntsc: id.Region = "ntsc"; break;
			case ConsoleRegion::NtscJapan: id.Region = "ntsc"; break;
			case ConsoleRegion::Pal: id.Region = "pal"; break;
			case ConsoleRegion::Dendy: id.Region = "dendy"; break;
			default: id.Region = "auto"; break;
		}

		if(_nes && _nes->GetMapper()) {
			char board[32];
			snprintf(board, sizeof(board), "%u", _nes->GetMapper()->GetRomInfo().MapperID);
			id.Board = board;
			// HasBattery() is protected, so presence of save RAM stands in for it.
			id.SaveLoaded = _emu->GetMemory(MemoryType::NesSaveRam).Size > 0;
			id.PrgBytes = _emu->GetMemory(MemoryType::NesPrgRom).Size;
			ConsoleMemoryInfo chrRam = _emu->GetMemory(MemoryType::NesChrRam);
			id.ChrBytes = chrRam.Size ? chrRam.Size : _emu->GetMemory(MemoryType::NesChrRom).Size;
		}

		// Deliberately left empty: this reference has no notion of how much of a
		// header it believed, so the gate must read it as "cannot say".
		id.HeaderTrust = "";
		return id;
	}

	void MesenBackend::Add(const char* name, MemoryType type)
	{
		ConsoleMemoryInfo info = _emu->GetMemory(type);
		if(info.Memory != nullptr && info.Size > 0) {
			_spaces.push_back({ name, (const uint8_t*)info.Memory, info.Size });
		}
	}

	// Which memories exist is what tells the two machines apart; the names are
	// the ones already on disk, so an existing dump set keeps its filenames.
	void MesenBackend::CacheSpaces()
	{
		_spaces.clear();
		if(_nes) {
			Add("ram", MemoryType::NesInternalRam);
			Add("sram", MemoryType::NesSaveRam);
			Add("work", MemoryType::NesWorkRam);
			Add("nametable", MemoryType::NesNametableRam);
			Add("oam", MemoryType::NesSpriteRam);
			Add("palette", MemoryType::NesPaletteRam);
			// CHR is RAM on some boards and ROM on others; only one is present.
			ConsoleMemoryInfo chrRam = _emu->GetMemory(MemoryType::NesChrRam);
			Add("chr", chrRam.Size ? MemoryType::NesChrRam : MemoryType::NesChrRom);
		} else {
			Add("vram", MemoryType::SnesVideoRam);
			Add("cgram", MemoryType::SnesCgRam);
			Add("oam", MemoryType::SnesSpriteRam);
			Add("gsuram", MemoryType::GsuWorkRam);
			Add("wram", MemoryType::SnesWorkRam);
			Add("apuram", MemoryType::SpcRam);
		}
	}

	std::string MesenBackend::StateLine()
	{
		char buffer[4096];
		std::string out;

		if(_gsu) {
			GsuState& s = _gsu->GetState();
			snprintf(buffer, sizeof(buffer), "SFR %04X PBR %02X RAMBR %02X SCBR %02X objMode %d bpp %d h %d",
				s.SFR.GetFlagsHigh() << 8 | s.SFR.GetFlagsLow(), s.ProgramBank, s.RamBank,
				s.ScreenBase, (int)s.ObjMode, (int)s.PlotBpp, (int)s.ScreenHeight);
			out += buffer;
		}
		out += "\n";

		if(_snes) {
			SnesPpuState ps = _snes->GetPpu()->GetState();
			FrameInfo fi = _emu->GetVideoDecoder()->GetBaseFrameInfo(true);
			snprintf(buffer, sizeof(buffer),
				"  PPU %ux%u: mode %d bg3prio %d TM %02X TS %02X hires %d overscan %d interlace %d extbg %d directcolor %d\n",
				fi.Width, fi.Height, ps.BgMode, (int)ps.Mode1Bg3Priority, ps.MainScreenLayers, ps.SubScreenLayers,
				(int)ps.HiResMode, (int)ps.OverscanMode, (int)ps.ScreenInterlace, (int)ps.ExtBgEnabled, (int)ps.DirectColorMode);
			out += buffer;

			for(int i = 0; i < 4; i++) {
				LayerConfig& L = ps.Layers[i];
				snprintf(buffer, sizeof(buffer),
					"       BG%d tilemap $%04X chr $%04X hscroll %04X vscroll %04X dblW %d dblH %d largeTiles %d wMain %d wSub %d\n",
					i + 1, L.TilemapAddress, L.ChrAddress, L.HScroll, L.VScroll,
					(int)L.DoubleWidth, (int)L.DoubleHeight, (int)L.LargeTiles,
					(int)ps.WindowMaskMain[i], (int)ps.WindowMaskSub[i]);
				out += buffer;
			}
		}

		if(_nes) {
			NesPpuState ps = {};
			_nes->GetPpu()->GetState(ps);
			ApuState as = _nes->GetApu()->GetState();
			snprintf(buffer, sizeof(buffer),
				"  PPU bgChr $%04X sprChr $%04X inc32 %d 8x16 %d nmi %d | bg %d spr %d bgLeft %d sprLeft %d gray %d | spr0 %d ovf %d vbl %d | v %04X t %04X x %d line %d cyc %u\n",
				ps.Control.BackgroundPatternAddr, ps.Control.SpritePatternAddr, (int)ps.Control.VerticalWrite,
				(int)ps.Control.LargeSprites, (int)ps.Control.NmiOnVerticalBlank,
				(int)ps.Mask.BackgroundEnabled, (int)ps.Mask.SpritesEnabled, (int)ps.Mask.BackgroundMask,
				(int)ps.Mask.SpriteMask, (int)ps.Mask.Grayscale,
				(int)ps.StatusFlags.Sprite0Hit, (int)ps.StatusFlags.SpriteOverflow, (int)ps.StatusFlags.VerticalBlank,
				ps.VideoRamAddr, ps.TmpVideoRamAddr, (int)ps.ScrollX, ps.Scanline, ps.Cycle);
			out += buffer;

			snprintf(buffer, sizeof(buffer),
				"  APU sq1 en %d per %4u vol %2u | sq2 en %d per %4u vol %2u | tri en %d per %4u vol %2u | noi en %d per %4u vol %2u | dmc len %u out %u | frame 5step %d irq %d\n",
				(int)as.Square1.Enabled, as.Square1.Period, as.Square1.OutputVolume,
				(int)as.Square2.Enabled, as.Square2.Period, as.Square2.OutputVolume,
				(int)as.Triangle.Enabled, as.Triangle.Period, as.Triangle.OutputVolume,
				(int)as.Noise.Enabled, as.Noise.Period, as.Noise.OutputVolume,
				as.Dmc.BytesRemaining, as.Dmc.OutputVolume,
				(int)as.FrameCounter.FiveStepMode, (int)as.FrameCounter.IrqEnabled);
			out += buffer;
		}

		return out;
	}
}

std::unique_ptr<IProbeBackend> CreateMesenBackend()
{
	return std::unique_ptr<IProbeBackend>(new MesenBackend());
}
