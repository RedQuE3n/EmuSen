// Headless Mesen driver: runs a ROM and dumps the state EmuSen can be diffed
// against at the same frame. Built against a Mesen2 checkout, not this repo -
// see EmuSen_Debugging_Tools_Reference_v5.md §3.39 and build-mesen-probe.sh.
#include "pch.h"
#include <thread>
#include <chrono>
#include <fstream>
#include <atomic>
#include "Shared/Emulator.h"
#include "Shared/EmuSettings.h"
#include "Shared/KeyManager.h"
#include "Shared/SettingTypes.h"
#include "Shared/Interfaces/IKeyManager.h"
#include "Shared/MemoryType.h"
#include "Shared/Video/VideoDecoder.h"
#include "SNES/SnesConsole.h"
#include "SNES/SnesPpu.h"
#include "SNES/BaseCartridge.h"
#include "SNES/Coprocessors/GSU/Gsu.h"
#include "SNES/Coprocessors/GSU/GsuTypes.h"
#include "Utilities/VirtualFile.h"
#include "Utilities/FolderUtilities.h"

// Supplied by mesen-gsu-trace.patch; the stock Mesen has no GSU trace hook.
extern bool g_gsuTraceOn;
extern std::vector<uint8_t> g_gsuTrace;

// Supplied by mesen-cpu-trace.patch - see EmuSen_Debugging_Tools_Reference_v5.md §3.40.
extern bool g_cpuTraceOn;
extern std::vector<uint8_t> g_cpuTrace;

// Synthetic scan codes this probe assigns to the SNES pad, so a --press
// script can hold a button without a real keyboard - see the man page.
enum ProbeKey : uint16_t
{
	KeyA = 1, KeyB = 2, KeyX = 3, KeyY = 4, KeyL = 5, KeyR = 6,
	KeyUp = 7, KeyDown = 8, KeyLeft = 9, KeyRight = 10,
	KeyStart = 11, KeySelect = 12,
};

// Mesen drives input through a registered key manager, so headless needs one.
// Rather than always "nothing pressed", this one answers from a frame-indexed
// schedule, which is what lets the probe walk a file-select menu and reach a
// scene that only exists after a new game is started.
class ScriptedKeyManager : public IKeyManager
{
public:
	static const uint32_t MaxFrames = 65536;

	// Filled before the emulator starts, then read-only. Indexed by frame, so
	// the schedule is resolved against Emulator::GetFrameCount() at the moment
	// the pad is polled - the probe runs at maximum speed, and a cached
	// per-poll counter would skip past whole presses.
	std::vector<std::vector<uint16_t>> Held{MaxFrames};
	Emulator* Emu = nullptr;

	// --pressuntil decides its presses while the emulator runs, so this one is
	// written from the control thread and read from the emulation thread.
	std::atomic<uint16_t> LiveKey{0};

	void Press(uint32_t startFrame, uint32_t duration, uint16_t key)
	{
		for(uint32_t f = startFrame; f < startFrame + duration && f < MaxFrames; f++) {
			Held[f].push_back(key);
		}
	}

	void RefreshState() {}
	void UpdateDevices() {}
	bool IsMouseButtonPressed(MouseButton button) { return false; }

	bool IsKeyPressed(uint16_t keyCode)
	{
		if(LiveKey.load(std::memory_order_relaxed) == keyCode) { return true; }
		uint32_t f = Emu ? Emu->GetFrameCount() : 0;
		if(f >= MaxFrames) return false;
		for(uint16_t k : Held[f]) { if(k == keyCode) return true; }
		return false;
	}

	vector<uint16_t> GetPressedKeys()
	{
		uint32_t f = Emu ? Emu->GetFrameCount() : 0;
		return f < MaxFrames ? Held[f] : vector<uint16_t>();
	}

	string GetKeyName(uint16_t keyCode) { return std::to_string(keyCode); }
	uint16_t GetKeyCode(string keyName) { return (uint16_t)atoi(keyName.c_str()); }
	bool SetKeyState(uint16_t scanCode, bool state) { return false; }
	void ResetKeyState() {}
	void SetDisabled(bool disabled) {}
};

// Maps a button name from the --press script onto its synthetic scan code.
static uint16_t KeyFromName(const string& n)
{
	if(n == "A") return KeyA;          if(n == "B") return KeyB;
	if(n == "X") return KeyX;          if(n == "Y") return KeyY;
	if(n == "L") return KeyL;          if(n == "R") return KeyR;
	if(n == "Up") return KeyUp;        if(n == "Down") return KeyDown;
	if(n == "Left") return KeyLeft;    if(n == "Right") return KeyRight;
	if(n == "Start") return KeyStart;  if(n == "Select") return KeySelect;
	return 0;
}

static void WriteBlob(const string& dir, const char* name, uint32_t frame, const void* data, size_t bytes)
{
	char path[1024];
	snprintf(path, sizeof(path), "%s/mesen_%s_f%05u.bin", dir.c_str(), name, frame);
	std::ofstream f(path, std::ios::binary);
	f.write((const char*)data, bytes);
}

static void DumpHex(const char* label, uint8_t* mem, uint32_t size, uint32_t addr, uint32_t len)
{
	printf("%s @ $%05X:", label, addr);
	for(uint32_t i = 0; i < len; i++) {
		printf(" %02X", addr + i < size ? mem[addr + i] : 0);
	}
	printf("\n");
}

int main(int argc, char** argv)
{
	if(argc < 5) {
		printf("usage: mesenprobe <rom> <outDir> <startFrame> <endFrame> [stride] [gsuRamAddr] [traceUntilFrame] [--press F:BTN[:DUR]]...\n");
		printf("  Writes mesen_{vram,cgram,oam,gsuram,wram,apuram,screen}_f<frame>.bin per report.\n");
		printf("  traceUntilFrame additionally records every GSU instruction from boot\n");
		printf("  as mesen_gsutrace_f<frame>.bin, in the shared 48-byte ESGT record `tracediff`\n");
		printf("  reads. With --pressuntil it is read as frames AFTER the anchor instead.\n");
		printf("  --press holds BTN (A/B/X/Y/L/R/Up/Down/Left/Right/Start/Select) on port 1\n");
		printf("  from frame F for DUR frames (default 4) - repeatable, needed to reach any\n");
		printf("  scene behind a menu.\n");
		printf("  --cputrace F records every S-CPU instruction from boot to frame F as\n");
		printf("  mesen_cputrace_f<frame>.bin, in the shared 24-byte record `tracediff` reads.\n");
		printf("  --after N reports for N frames past a --pressuntil anchor with no GSU\n");
		printf("  trace, which is how a value is watched over hundreds of frames.\n");
		printf("  --pressuntil BTN:ADDR:VALUE[:CAP[:EVERY]] taps BTN until GSU RAM word ADDR\n");
		printf("  equals VALUE (all hex), the counterpart of the harness's own `tapuntil` -\n");
		printf("  the only way to reach the same scene in both emulators without frame maths.\n");
		return 1;
	}

	string romPath = argv[1];
	string dumpDir = argv[2];
	uint32_t startFrame = (uint32_t)atoi(argv[3]);
	uint32_t endFrame = (uint32_t)atoi(argv[4]);
	// A "--" flag ends the optional positionals, so --press can follow endFrame
	// directly without being read as a stride.
	auto positional = [&](int i) { return argc > i && argv[i][0] != '-'; };
	uint32_t stride = positional(5) ? (uint32_t)atoi(argv[5]) : 1;
	uint32_t dumpAddr = positional(6) ? (uint32_t)strtoul(argv[6], nullptr, 16) : 0x0A00;
	uint32_t traceFrame = positional(7) ? (uint32_t)atoi(argv[7]) : 0xFFFFFFFF;

	// Kept beside the dumps so a probe run never touches a real Mesen profile
	// (and so its .srm does not silently change what the ROM boots into).
	FolderUtilities::SetHomeFolder(dumpDir + "/mesenhome");

	unique_ptr<Emulator> emu(new Emulator());
	unique_ptr<ScriptedKeyManager> km(new ScriptedKeyManager());
	uint32_t cpuTraceFrame = 0xFFFFFFFF;
	uint16_t untilKey = 0;
	uint32_t untilAddr = 0, untilValue = 0, untilCap = 6000, untilEvery = 40;
	uint32_t afterFrames = 0;
	for(int i = 5; i < argc; i++) {
		if(string(argv[i]) == "--after" && i + 1 < argc) {
			afterFrames = (uint32_t)atoi(argv[++i]);
			continue;
		}
		if(string(argv[i]) == "--cputrace" && i + 1 < argc) {
			cpuTraceFrame = (uint32_t)atoi(argv[++i]);
			continue;
		}
		if(string(argv[i]) == "--pressuntil" && i + 1 < argc) {
			string spec = argv[++i];
			vector<string> f;
			size_t at = 0;
			while(at <= spec.size()) {
				size_t c = spec.find(':', at);
				if(c == string::npos) { f.push_back(spec.substr(at)); break; }
				f.push_back(spec.substr(at, c - at));
				at = c + 1;
			}
			if(f.size() < 3) { printf("[ERROR] --pressuntil wants BTN:ADDR:VALUE[:CAP[:EVERY]]\n"); return 1; }
			untilKey = KeyFromName(f[0]);
			if(untilKey == 0) { printf("[ERROR] unknown button '%s'\n", f[0].c_str()); return 1; }
			untilAddr = (uint32_t)strtoul(f[1].c_str(), nullptr, 16);
			untilValue = (uint32_t)strtoul(f[2].c_str(), nullptr, 16);
			if(f.size() >= 4) { untilCap = (uint32_t)atoi(f[3].c_str()); }
			if(f.size() >= 5) { untilEvery = (uint32_t)atoi(f[4].c_str()); }
			continue;
		}
		if(string(argv[i]) != "--press" || i + 1 >= argc) continue;
		string spec = argv[++i];
		size_t c1 = spec.find(':');
		size_t c2 = spec.find(':', c1 + 1);
		uint32_t start = (uint32_t)atoi(spec.substr(0, c1).c_str());
		string btn = spec.substr(c1 + 1, c2 == string::npos ? string::npos : c2 - c1 - 1);
		uint32_t dur = c2 == string::npos ? 4 : (uint32_t)atoi(spec.substr(c2 + 1).c_str());
		uint16_t key = KeyFromName(btn);
		if(key == 0) { printf("[ERROR] unknown button '%s'\n", btn.c_str()); return 1; }
		km->Press(start, dur, key);
		printf("[INFO] press %s frames %u..%u\n", btn.c_str(), start, start + dur - 1);
	}
	km->Emu = emu.get();
	KeyManager::RegisterKeyManager(km.get());
	KeyManager::SetSettings(emu->GetSettings());

	emu->Initialize();
	emu->GetSettings()->SetFlag(EmulationFlags::MaximumSpeed);

	// The stock config has no keyboard binding at all, so port 1 would read as
	// idle no matter what the schedule says.
	SnesConfig scfg = emu->GetSettings()->GetSnesConfig();
	// Mesen defaults to a random power-on RAM fill, which makes it non-reproducible
	// run to run - measured, not assumed. Zero matches our own fill - see §3.40.
	scfg.RamPowerOnState = RamState::AllZeros;
	scfg.Port1.Type = ControllerType::SnesController;
	KeyMapping& kmap = scfg.Port1.Keys.Mapping1;
	kmap.A = KeyA; kmap.B = KeyB; kmap.X = KeyX; kmap.Y = KeyY;
	kmap.L = KeyL; kmap.R = KeyR;
	kmap.Up = KeyUp; kmap.Down = KeyDown; kmap.Left = KeyLeft; kmap.Right = KeyRight;
	kmap.Start = KeyStart; kmap.Select = KeySelect;
	emu->GetSettings()->SetSnesConfig(scfg);

	// Armed before LoadRom, which is what starts execution - the reset vector
	// and the whole boot sequence are the point of this trace.
	if(cpuTraceFrame != 0xFFFFFFFF) { g_cpuTrace.reserve(64u * 1024 * 1024); g_cpuTraceOn = true; }

	if(!emu->LoadRom((VirtualFile)romPath, VirtualFile())) {
		printf("[ERROR] failed to load %s\n", romPath.c_str());
		return 1;
	}

	// The S-CPU's own RAM, so a divergence that starts on the CPU side can be
	// bisected the same way GSU RAM already could - see §3.39.
	ConsoleMemoryInfo wram = emu->GetMemory(MemoryType::SnesWorkRam);
	// The SPC700's RAM, so a boot-timing lag in the APU upload handshake is
	// visible as data rather than inferred - see §10.7.
	ConsoleMemoryInfo apuRam = emu->GetMemory(MemoryType::SpcRam);
	ConsoleMemoryInfo gsuRam = emu->GetMemory(MemoryType::GsuWorkRam);
	ConsoleMemoryInfo vram = emu->GetMemory(MemoryType::SnesVideoRam);
	ConsoleMemoryInfo cgram = emu->GetMemory(MemoryType::SnesCgRam);
	ConsoleMemoryInfo oam = emu->GetMemory(MemoryType::SnesSpriteRam);
	printf("[INFO] GsuWorkRam %u bytes, WRAM %u, APURAM %u, VRAM %u, CGRAM %u, OAM %u\n", gsuRam.Size, wram.Size, apuRam.Size, vram.Size, cgram.Size, oam.Size);

	SnesConsole* console = dynamic_cast<SnesConsole*>(emu->GetConsole().get());
	Gsu* gsu = console ? console->GetCartridge()->GetGsu() : nullptr;

	// Anchors the run on game state instead of a frame count, so the same scene
	// is reached here and in the harness even though the two emulators do not
	// agree on how many frames it takes - see §3.15d and §3.40.
	if(untilKey != 0) {
		uint32_t lastPress = 0;
		bool reached = false;
		uint32_t taps = 0, frame = 0;
		while(frame < untilCap) {
			emu->Pause();
			while(!emu->IsPaused()) { std::this_thread::sleep_for(std::chrono::milliseconds(1)); }
			frame = emu->GetFrameCount();
			uint32_t got = ((uint8_t*)gsuRam.Memory)[(untilAddr + 1) & (gsuRam.Size - 1)] << 8
			             | ((uint8_t*)gsuRam.Memory)[untilAddr & (gsuRam.Size - 1)];
			if(got == untilValue) { reached = true; emu->Resume(); break; }

			// Held for four frames then released; a held button reads as one
			// press to most menus, exactly as `tapuntil` does on our side.
			uint32_t phase = frame - lastPress;
			if(phase >= untilEvery) { km->LiveKey.store(untilKey); lastPress = frame; taps++; }
			else if(phase >= 4) { km->LiveKey.store(0); }

			emu->Resume();
			std::this_thread::sleep_for(std::chrono::milliseconds(1));
		}
		km->LiveKey.store(0);
		printf(reached
			? "[PRESSUNTIL] GSURAM $%04X reached $%04X after %u tap(s) at frame %u\n"
			: "[PRESSUNTIL] GSURAM $%04X NOT reached ($%04X) after %u tap(s), frame %u\n",
			untilAddr, untilValue, taps, frame);
		fflush(stdout);
		if(!reached) { emu->Stop(false); emu->Release(); return 2; }
		// Report from where the anchor landed; traceUntilFrame is re-read as
		// "how many frames of GSU trace after it" rather than an absolute frame.
		// --after is the same span without the trace, for watching a value move
		// over hundreds of frames - see §3.41.
		startFrame = emu->GetFrameCount();
		endFrame = startFrame + (traceFrame != 0xFFFFFFFF ? traceFrame : afterFrames);
		if(traceFrame != 0xFFFFFFFF) { traceFrame = endFrame; }
	}

	if(traceFrame != 0xFFFFFFFF) { g_gsuTraceOn = true; }

	uint32_t nextReport = startFrame;
	while(true) {
		uint32_t frame = emu->GetFrameCount();
		if(frame < nextReport) {
			std::this_thread::sleep_for(std::chrono::milliseconds(2));
			continue;
		}

		// Pause lands on a frame boundary, which is the only point the two
		// emulators can be compared without modelling each other's timing.
		emu->Pause();
		while(!emu->IsPaused()) { std::this_thread::sleep_for(std::chrono::milliseconds(1)); }

		frame = emu->GetFrameCount();
		printf("frame %5u | ", frame);
		if(gsu) {
			GsuState& s = gsu->GetState();
			printf("SFR %04X PBR %02X RAMBR %02X SCBR %02X objMode %d bpp %d h %d",
				s.SFR.GetFlagsHigh() << 8 | s.SFR.GetFlagsLow(), s.ProgramBank, s.RamBank,
				s.ScreenBase, (int)s.ObjMode, (int)s.PlotBpp, (int)s.ScreenHeight);
		}
		printf("\n");

		if(console) {
			SnesPpuState ps = console->GetPpu()->GetState();
			FrameInfo fi = emu->GetVideoDecoder()->GetBaseFrameInfo(true);
			printf("  PPU %ux%u: mode %d bg3prio %d TM %02X TS %02X hires %d overscan %d interlace %d extbg %d directcolor %d\n",
				fi.Width, fi.Height, ps.BgMode, (int)ps.Mode1Bg3Priority, ps.MainScreenLayers, ps.SubScreenLayers,
				(int)ps.HiResMode, (int)ps.OverscanMode, (int)ps.ScreenInterlace, (int)ps.ExtBgEnabled, (int)ps.DirectColorMode);
			for(int i = 0; i < 4; i++) {
				LayerConfig& L = ps.Layers[i];
				printf("       BG%d tilemap $%04X chr $%04X hscroll %04X vscroll %04X dblW %d dblH %d largeTiles %d wMain %d wSub %d\n",
					i + 1, L.TilemapAddress, L.ChrAddress, L.HScroll, L.VScroll,
					(int)L.DoubleWidth, (int)L.DoubleHeight, (int)L.LargeTiles,
					(int)ps.WindowMaskMain[i], (int)ps.WindowMaskSub[i]);
			}
		}
		DumpHex("  GSURAM", (uint8_t*)gsuRam.Memory, gsuRam.Size, dumpAddr, 16);

		WriteBlob(dumpDir, "vram", frame, vram.Memory, vram.Size);
		WriteBlob(dumpDir, "cgram", frame, cgram.Memory, cgram.Size);
		WriteBlob(dumpDir, "oam", frame, oam.Memory, oam.Size);
		WriteBlob(dumpDir, "gsuram", frame, gsuRam.Memory, gsuRam.Size);
		WriteBlob(dumpDir, "wram", frame, wram.Memory, wram.Size);
		WriteBlob(dumpDir, "apuram", frame, apuRam.Memory, apuRam.Size);
		if(console) {
			// Raw BGR555, row stride is the reported width - not 512.
			WriteBlob(dumpDir, "screen", frame, console->GetPpu()->GetScreenBuffer(), 512 * 478 * 2);
		}

		if(g_cpuTraceOn && frame >= cpuTraceFrame) {
			g_cpuTraceOn = false;
			char path[1024];
			snprintf(path, sizeof(path), "%s/mesen_cputrace_f%05u.bin", dumpDir.c_str(), frame);
			std::ofstream fc(path, std::ios::binary);
			// Version 2 = the 24-byte record carrying per-instruction cost.
			fc.write("ESCT\2\0\0\0", 8);
			fc.write((char*)g_cpuTrace.data(), g_cpuTrace.size());
			printf("  [cputrace %zu steps -> %s]\n", g_cpuTrace.size() / 24, path);
			g_cpuTrace.clear();
		}

		if(frame >= traceFrame) {
			g_gsuTraceOn = false;
			char path[1024];
			snprintf(path, sizeof(path), "%s/mesen_gsutrace_f%05u.bin", dumpDir.c_str(), frame);
			std::ofstream ft(path, std::ios::binary);
			// Version 1 = the 48-byte record `tracediff` reads - see §3.41.
			ft.write("ESGT\1\0\0\0", 8);
			ft.write((char*)g_gsuTrace.data(), g_gsuTrace.size());
			printf("  [gsutrace %zu steps -> %s]\n", g_gsuTrace.size() / 48, path);
			g_gsuTrace.clear();
		}

		fflush(stdout);
		nextReport = frame + stride;
		emu->Resume();
		if(frame >= endFrame) { break; }
	}

	emu->Stop(false);
	emu->Release();
	return 0;
}
