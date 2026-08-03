// Headless Mesen driver: runs a ROM and dumps the state EmuSen can be diffed
// against at the same frame. Built against a Mesen2 checkout, not this repo -
// see EmuSen_Debugging_Tools_Reference_v5.md §3.39 and build-mesen-probe.sh.
#include "pch.h"
#include <thread>
#include <chrono>
#include <fstream>
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
extern std::vector<uint32_t> g_gsuTrace;

// Mesen drives input through a registered key manager, so headless needs one
// that is always "nothing pressed" rather than none at all.
class NullKeyManager : public IKeyManager
{
public:
	void RefreshState() {}
	void UpdateDevices() {}
	bool IsMouseButtonPressed(MouseButton button) { return false; }
	bool IsKeyPressed(uint16_t keyCode) { return false; }
	vector<uint16_t> GetPressedKeys() { return {}; }
	string GetKeyName(uint16_t keyCode) { return ""; }
	uint16_t GetKeyCode(string keyName) { return 0; }
	bool SetKeyState(uint16_t scanCode, bool state) { return false; }
	void ResetKeyState() {}
	void SetDisabled(bool disabled) {}
};

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
		printf("usage: mesenprobe <rom> <outDir> <startFrame> <endFrame> [stride] [gsuRamAddr] [traceUntilFrame]\n");
		printf("  Writes mesen_{vram,cgram,oam,gsuram,screen}_f<frame>.bin per report.\n");
		printf("  traceUntilFrame additionally records every GSU instruction from boot\n");
		printf("  as mesen_gsutrace_f<frame>.bin - 18 uint32 per step: addr, opcode, R0-R15.\n");
		return 1;
	}

	string romPath = argv[1];
	string dumpDir = argv[2];
	uint32_t startFrame = (uint32_t)atoi(argv[3]);
	uint32_t endFrame = (uint32_t)atoi(argv[4]);
	uint32_t stride = argc > 5 ? (uint32_t)atoi(argv[5]) : 1;
	uint32_t dumpAddr = argc > 6 ? (uint32_t)strtoul(argv[6], nullptr, 16) : 0x0A00;
	uint32_t traceFrame = argc > 7 ? (uint32_t)atoi(argv[7]) : 0xFFFFFFFF;

	// Kept beside the dumps so a probe run never touches a real Mesen profile
	// (and so its .srm does not silently change what the ROM boots into).
	FolderUtilities::SetHomeFolder(dumpDir + "/mesenhome");

	unique_ptr<Emulator> emu(new Emulator());
	NullKeyManager km;
	KeyManager::RegisterKeyManager(&km);
	KeyManager::SetSettings(emu->GetSettings());

	emu->Initialize();
	emu->GetSettings()->SetFlag(EmulationFlags::MaximumSpeed);

	if(!emu->LoadRom((VirtualFile)romPath, VirtualFile())) {
		printf("[ERROR] failed to load %s\n", romPath.c_str());
		return 1;
	}

	ConsoleMemoryInfo gsuRam = emu->GetMemory(MemoryType::GsuWorkRam);
	ConsoleMemoryInfo vram = emu->GetMemory(MemoryType::SnesVideoRam);
	ConsoleMemoryInfo cgram = emu->GetMemory(MemoryType::SnesCgRam);
	ConsoleMemoryInfo oam = emu->GetMemory(MemoryType::SnesSpriteRam);
	printf("[INFO] GsuWorkRam %u bytes, VRAM %u, CGRAM %u, OAM %u\n", gsuRam.Size, vram.Size, cgram.Size, oam.Size);

	SnesConsole* console = dynamic_cast<SnesConsole*>(emu->GetConsole().get());
	Gsu* gsu = console ? console->GetCartridge()->GetGsu() : nullptr;

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
		if(console) {
			// Raw BGR555, row stride is the reported width - not 512.
			WriteBlob(dumpDir, "screen", frame, console->GetPpu()->GetScreenBuffer(), 512 * 478 * 2);
		}

		if(frame >= traceFrame) {
			g_gsuTraceOn = false;
			char path[1024];
			snprintf(path, sizeof(path), "%s/mesen_gsutrace_f%05u.bin", dumpDir.c_str(), frame);
			std::ofstream ft(path, std::ios::binary);
			ft.write((char*)g_gsuTrace.data(), g_gsuTrace.size() * 4);
			printf("  [gsutrace %zu steps -> %s]\n", g_gsuTrace.size() / 18, path);
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
