// The kit's bundled faces put in the application's font database once, and the platform's default family; the reasons are EmuSen_Kit.md §7.117.
#pragma once

#include <QtCore/QString>
#include <QtCore/QStringList>

QStringList bundled_families();
QString platform_family();
