/* libretro's log callback is printf-style, which stable Rust cannot define: this formats a record and hands Rust one
 * finished line. See EmuSen_LibretroHost.md section 1.2. */
#include <stdarg.h>
#include <stdio.h>

void emusen_libretro_log_line(int level, const char *line);

void emusen_libretro_log(int level, const char *fmt, ...)
{
    char line[4096];
    va_list args;
    if (!fmt)
        return;
    va_start(args, fmt);
    vsnprintf(line, sizeof line, fmt, args);
    va_end(args);
    emusen_libretro_log_line(level, line);
}
