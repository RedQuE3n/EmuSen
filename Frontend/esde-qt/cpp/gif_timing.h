// A GIF's frame count and first frame's delay through QImageReader, which cxx-qt-lib does not bind (EmuSen_Qt.md §4.5.9).
#pragma once

#include <QtCore/QString>
#include <cstdint>

std::int32_t gif_frame_count(const QString& path);
std::int32_t gif_first_delay(const QString& path);
