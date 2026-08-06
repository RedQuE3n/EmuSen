#include "LibretroBackend.h"

#include "libretro.h"

#include <cstdarg>
#include <cstdio>
#include <cstring>
#include <dlfcn.h>
#include <fstream>
#include <vector>

class LibretroBackend;

namespace
{
	// libretro's callbacks are free functions with no user pointer, so the core
	// is process-global by design. One core per probe run, which is all this
	// tool ever wants.
	LibretroBackend* g_backend = nullptr;

	unsigned JoypadId(ProbeButton button)
	{
		switch(button) {
			case ProbeButton::A: return RETRO_DEVICE_ID_JOYPAD_A;
			case ProbeButton::B: return RETRO_DEVICE_ID_JOYPAD_B;
			case ProbeButton::X: return RETRO_DEVICE_ID_JOYPAD_X;
			case ProbeButton::Y: return RETRO_DEVICE_ID_JOYPAD_Y;
			case ProbeButton::L: return RETRO_DEVICE_ID_JOYPAD_L;
			case ProbeButton::R: return RETRO_DEVICE_ID_JOYPAD_R;
			case ProbeButton::Up: return RETRO_DEVICE_ID_JOYPAD_UP;
			case ProbeButton::Down: return RETRO_DEVICE_ID_JOYPAD_DOWN;
			case ProbeButton::Left: return RETRO_DEVICE_ID_JOYPAD_LEFT;
			case ProbeButton::Right: return RETRO_DEVICE_ID_JOYPAD_RIGHT;
			case ProbeButton::Start: return RETRO_DEVICE_ID_JOYPAD_START;
			case ProbeButton::Select: return RETRO_DEVICE_ID_JOYPAD_SELECT;
			default: return RETRO_DEVICE_ID_JOYPAD_A;
		}
	}

	// Which machine a core is emulating is not something libretro reports, and
	// the space names have to line up with the native backends' or a
	// cross-emulator diff compares "wram" against "ram". The ROM extension is
	// the one signal available before the core is even loaded.
	std::string SystemFromRom(const std::string& romPath)
	{
		size_t dot = romPath.find_last_of('.');
		std::string ext = dot == std::string::npos ? "" : romPath.substr(dot + 1);
		for(char& c : ext) { c = (char)tolower(c); }

		if(ext == "nes" || ext == "fds" || ext == "unf") { return "nes"; }
		if(ext == "smc" || ext == "sfc" || ext == "swc" || ext == "fig") { return "snes"; }
		if(ext == "gb" || ext == "gbc") { return "gb"; }
		if(ext == "gba") { return "gba"; }
		if(ext == "sms" || ext == "gg") { return "sms"; }
		if(ext == "md" || ext == "gen" || ext == "smd") { return "megadrive"; }
		return "unknown";
	}

	void LogCallback(enum retro_log_level level, const char* format, ...)
	{
		if(level < RETRO_LOG_WARN) { return; }
		va_list args;
		va_start(args, format);
		vprintf(format, args);
		va_end(args);
	}
}

class LibretroBackend : public IProbeBackend
{
public:
	explicit LibretroBackend(const std::string& corePath) : _corePath(corePath) {}

	const char* Name() const override { return _name.c_str(); }
	const char* System() const override { return _system.c_str(); }
	const char* AnchorSpace() const override { return _system == "snes" || _system == "gb" ? "wram" : "ram"; }

	bool Load(const std::string& romPath, const ProbeOptions& options) override;
	void Shutdown() override;

	// Exact by construction: retro_run() is one frame, called from this thread.
	// Nothing here can overshoot, which is the whole problem the Mesen backend
	// needed a source patch to solve.
	void RunUntil(uint32_t frame) override
	{
		while(_frame < frame) { _run(); _frame++; }
	}

	uint32_t FrameCount() override { return _frame; }
	std::vector<MemorySpace> Spaces() override { return _spaces; }
	void SetSchedule(const InputSchedule& schedule) override { _schedule = schedule; }

	void SetButton(ProbeButton button, bool held) override
	{
		if(held) { _live |= 1u << (int)button; } else { _live &= ~(1u << (int)button); }
	}

	bool Screen(ScreenView& out) override
	{
		if(_screen.empty()) { return false; }
		out = { _screen.data(), _width, _height, (uint32_t)_screen.size(), _format };
		return true;
	}

	std::string StateLine() override
	{
		char buffer[256];
		snprintf(buffer, sizeof(buffer), "\n  %s %ux%u %s\n", _name.c_str(), _width, _height, ScreenFormatName(_format));
		return buffer;
	}

	// The callbacks, which libretro hands no user pointer, so they reach the
	// single instance through g_backend.
	static bool Environment(unsigned cmd, void* data);
	static void VideoRefresh(const void* data, unsigned width, unsigned height, size_t pitch);
	static void InputPoll() {}
	static int16_t InputState(unsigned port, unsigned device, unsigned index, unsigned id);
	static void AudioSample(int16_t, int16_t) {}
	static size_t AudioBatch(const int16_t*, size_t frames) { return frames; }

private:
	void CacheSpaces();
	void AddSpace(const char* name, unsigned id);

	template<typename T> bool Resolve(T& target, const char* symbol)
	{
		target = (T)dlsym(_handle, symbol);
		if(target == nullptr) { printf("[ERROR] core has no %s\n", symbol); }
		return target != nullptr;
	}

	std::string _corePath, _name = "libretro", _system = "unknown", _homeFolder;
	void* _handle = nullptr;
	std::vector<uint8_t> _romData;
	std::vector<MemorySpace> _spaces;
	std::vector<uint8_t> _screen;
	InputSchedule _schedule;
	uint32_t _live = 0, _frame = 0, _width = 0, _height = 0;
	ScreenFormat _format = ScreenFormat::None;
	retro_pixel_format _pixelFormat = RETRO_PIXEL_FORMAT_0RGB1555;
	bool _loaded = false;

	void (*_init)() = nullptr;
	void (*_deinit)() = nullptr;
	void (*_run)() = nullptr;
	void (*_unloadGame)() = nullptr;
	bool (*_loadGame)(const retro_game_info*) = nullptr;
	void (*_getSystemInfo)(retro_system_info*) = nullptr;
	void (*_setControllerPortDevice)(unsigned, unsigned) = nullptr;
	void* (*_getMemoryData)(unsigned) = nullptr;
	size_t (*_getMemorySize)(unsigned) = nullptr;
};

bool LibretroBackend::Environment(unsigned cmd, void* data)
{
	switch(cmd) {
		case RETRO_ENVIRONMENT_SET_PIXEL_FORMAT:
			g_backend->_pixelFormat = *(const retro_pixel_format*)data;
			return true;

		case RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY:
		case RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY:
		case RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY:
			*(const char**)data = g_backend->_homeFolder.c_str();
			return true;

		case RETRO_ENVIRONMENT_GET_LOG_INTERFACE:
			((retro_log_callback*)data)->log = LogCallback;
			return true;

		case RETRO_ENVIRONMENT_GET_CAN_DUPE:
			*(bool*)data = true;
			return true;

		// Every core option is left at its default, deliberately: a probe that
		// silently ran with different settings than the one it is compared
		// against is the same class of error as a muted reference.
		case RETRO_ENVIRONMENT_GET_VARIABLE:
			((retro_variable*)data)->value = nullptr;
			return false;

		case RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE:
			*(bool*)data = false;
			return true;

		case RETRO_ENVIRONMENT_SET_VARIABLES:
		case RETRO_ENVIRONMENT_SET_CORE_OPTIONS:
		case RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL:
		case RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2:
		case RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL:
		case RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL:
		case RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS:
		case RETRO_ENVIRONMENT_SET_CONTROLLER_INFO:
		case RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME:
		case RETRO_ENVIRONMENT_SET_MEMORY_MAPS:
		case RETRO_ENVIRONMENT_SET_GEOMETRY:
			return true;

		default:
			return false;
	}
}

void LibretroBackend::VideoRefresh(const void* data, unsigned width, unsigned height, size_t pitch)
{
	// A null frame is a deliberate duplicate; keeping the previous buffer is
	// what every frontend does and what makes a dump at that frame meaningful.
	if(data == nullptr) { return; }

	LibretroBackend* self = g_backend;
	self->_width = width;
	self->_height = height;

	uint32_t bytesPerPixel = self->_pixelFormat == RETRO_PIXEL_FORMAT_XRGB8888 ? 4 : 2;
	self->_format = self->_pixelFormat == RETRO_PIXEL_FORMAT_XRGB8888 ? ScreenFormat::Xrgb8888
	              : self->_pixelFormat == RETRO_PIXEL_FORMAT_RGB565 ? ScreenFormat::Rgb565
	              : ScreenFormat::Bgr555;

	// Repacked to tight rows. The core's pitch is padding, not data, and a
	// consumer diffing two emulators' screens should not have to model it.
	size_t rowBytes = (size_t)width * bytesPerPixel;
	self->_screen.resize(rowBytes * height);
	for(unsigned y = 0; y < height; y++) {
		memcpy(self->_screen.data() + y * rowBytes, (const uint8_t*)data + y * pitch, rowBytes);
	}
}

int16_t LibretroBackend::InputState(unsigned port, unsigned device, unsigned index, unsigned id)
{
	(void)index;
	if(port != 0 || device != RETRO_DEVICE_JOYPAD) { return 0; }

	LibretroBackend* self = g_backend;
	for(int i = 0; i < (int)ProbeButton::Count; i++) {
		if(JoypadId((ProbeButton)i) != id) { continue; }
		bool live = (self->_live & (1u << i)) != 0;
		return live || self->_schedule.HeldAt(self->_frame, (ProbeButton)i) ? 1 : 0;
	}
	return 0;
}

void LibretroBackend::AddSpace(const char* name, unsigned id)
{
	void* data = _getMemoryData(id);
	size_t size = _getMemorySize(id);
	if(data != nullptr && size > 0) { _spaces.push_back({ name, (const uint8_t*)data, (uint32_t)size }); }
}

void LibretroBackend::CacheSpaces()
{
	_spaces.clear();
	// SYSTEM_RAM is the console's main work RAM, which the native backends call
	// "ram" on an NES and "wram" on a SNES or Game Boy.
	AddSpace(AnchorSpace(), RETRO_MEMORY_SYSTEM_RAM);
	AddSpace("sram", RETRO_MEMORY_SAVE_RAM);
	AddSpace("vram", RETRO_MEMORY_VIDEO_RAM);
	AddSpace("rtc", RETRO_MEMORY_RTC);
}

bool LibretroBackend::Load(const std::string& romPath, const ProbeOptions& options)
{
	g_backend = this;
	_system = SystemFromRom(romPath);
	_homeFolder = options.HomeFolder;

	_handle = dlopen(_corePath.c_str(), RTLD_NOW | RTLD_LOCAL);
	if(_handle == nullptr) { printf("[ERROR] dlopen %s: %s\n", _corePath.c_str(), dlerror()); return false; }

	unsigned (*apiVersion)() = nullptr;
	void (*setEnvironment)(retro_environment_t) = nullptr;
	void (*setVideoRefresh)(retro_video_refresh_t) = nullptr;
	void (*setInputPoll)(retro_input_poll_t) = nullptr;
	void (*setInputState)(retro_input_state_t) = nullptr;
	void (*setAudioSample)(retro_audio_sample_t) = nullptr;
	void (*setAudioSampleBatch)(retro_audio_sample_batch_t) = nullptr;

	if(!Resolve(apiVersion, "retro_api_version") || !Resolve(_init, "retro_init")
	   || !Resolve(_deinit, "retro_deinit") || !Resolve(_run, "retro_run")
	   || !Resolve(_loadGame, "retro_load_game") || !Resolve(_unloadGame, "retro_unload_game")
	   || !Resolve(_getSystemInfo, "retro_get_system_info")
	   || !Resolve(_setControllerPortDevice, "retro_set_controller_port_device")
	   || !Resolve(_getMemoryData, "retro_get_memory_data")
	   || !Resolve(_getMemorySize, "retro_get_memory_size")
	   || !Resolve(setEnvironment, "retro_set_environment")
	   || !Resolve(setVideoRefresh, "retro_set_video_refresh")
	   || !Resolve(setInputPoll, "retro_set_input_poll")
	   || !Resolve(setInputState, "retro_set_input_state")
	   || !Resolve(setAudioSample, "retro_set_audio_sample")
	   || !Resolve(setAudioSampleBatch, "retro_set_audio_sample_batch")) {
		return false;
	}

	if(apiVersion() != RETRO_API_VERSION) {
		printf("[ERROR] core speaks libretro %u, this probe speaks %u\n", apiVersion(), RETRO_API_VERSION);
		return false;
	}

	retro_system_info info = {};
	_getSystemInfo(&info);
	_name = info.library_name ? info.library_name : "libretro";
	// The dump prefix becomes a filename, so it cannot carry spaces.
	for(char& c : _name) { c = c == ' ' ? '-' : (char)tolower(c); }
	printf("[INFO] core %s %s\n", _name.c_str(), info.library_version ? info.library_version : "");

	// Before retro_init, which is the one ordering rule the ABI actually has.
	setEnvironment(Environment);
	_init();
	setVideoRefresh(VideoRefresh);
	setInputPoll(InputPoll);
	setInputState(InputState);
	setAudioSample(AudioSample);
	setAudioSampleBatch(AudioBatch);

	retro_game_info game = {};
	game.path = romPath.c_str();
	if(!info.need_fullpath) {
		std::ifstream file(romPath, std::ios::binary | std::ios::ate);
		if(!file) { printf("[ERROR] cannot read %s\n", romPath.c_str()); return false; }
		_romData.resize((size_t)file.tellg());
		file.seekg(0);
		file.read((char*)_romData.data(), _romData.size());
		game.data = _romData.data();
		game.size = _romData.size();
	}

	if(!_loadGame(&game)) { printf("[ERROR] core refused %s\n", romPath.c_str()); return false; }
	_loaded = true;

	_setControllerPortDevice(0, RETRO_DEVICE_JOYPAD);
	CacheSpaces();

	if(!options.WavPath.empty()) { printf("[WARN] --wav is not supported by the libretro backend\n"); }
	if(options.RamState != "zeros") { printf("[WARN] --ramstate is not settable through libretro\n"); }
	return true;
}

void LibretroBackend::Shutdown()
{
	if(_loaded) { _unloadGame(); _loaded = false; }
	if(_deinit) { _deinit(); }
	if(_handle) { dlclose(_handle); _handle = nullptr; }
	g_backend = nullptr;
}

std::unique_ptr<IProbeBackend> CreateLibretroBackend(const std::string& corePath)
{
	return std::unique_ptr<IProbeBackend>(new LibretroBackend(corePath));
}
