# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsGenesis-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 96,56.5,1728,982.5
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. Not downloaded. libretro's slang pack, about 54 MB from buildbot.libretro.com, is the file RetroArch's updater fetches: some 2,600 presets. EmuSen ships none of it, and each shader keeps its authors' licence."
case ShaderSettingsGenesis
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,206,1728,678 by=panel limit=96,206,1728,678 paints=#D4D4D4FF role=none
    1 DockPanel at=132,224,1656,642 by=panel limit=96,206,1728,678 role=none
      2 Grid at=132,224,1656,642 by=panel limit=96,206,1728,678 role=none
        3 Tabs at=132,224,1656,642 by=panel limit=96,206,1728,678 name=ConsoleTabs paints=#D4D4D4FF role=tab
          4 TabItem at=132,224,90,72 by=clip drawn focus heard=NES limit=132,224,1656,642 paints=#8D8D91FF role=tabitem sideways text=NES
            5 TextBlock at=150,234.5,54,51 box=36,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=NES layout=35.2,33.6 limit=132,224,90,72 owner=4 paints=#8D8D91FF role=text size=28 text=NES
          6 TabItem at=222,224,73.5,72 by=clip drawn focus heard=GB limit=132,224,1656,642 paints=#8D8D91FF role=tabitem sideways text=GB
            7 TextBlock at=240,234.5,37.5,51 box=25,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=GB layout=24.05,33.6 limit=222,224,73.5,72 owner=6 paints=#8D8D91FF role=text size=28 text=GB
          8 TabItem at=295.5,224,106.5,72 by=clip drawn focus heard=SNES limit=132,224,1656,642 paints=#8D8D91FF role=tabitem sideways text=SNES
            9 TextBlock at=313.5,234.5,70.5,51 box=47,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=SNES layout=46.23,33.6 limit=295.5,224,106.5,72 owner=8 paints=#8D8D91FF role=text size=28 text=SNES
          10 TabItem at=402,224,90,72 by=clip drawn focus heard=N64 limit=132,224,1656,642 paints=#8D8D91FF role=tabitem sideways text=N64
            11 TextBlock at=420,234.5,54,51 box=36,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=N64 layout=35.45,33.6 limit=402,224,90,72 owner=10 paints=#8D8D91FF role=text size=28 text=N64
          12 TabItem at=492,224,148.5,72 by=clip drawn focus heard=SG-1000 limit=132,224,1656,642 paints=#8D8D91FF role=tabitem sideways text=SG-1000
            13 TextBlock at=510,234.5,112.5,51 box=75,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=SG-1000 layout=74.48,33.6 limit=492,224,148.5,72 owner=12 paints=#8D8D91FF role=text size=28 text=SG-1000
          14 TabItem at=640.5,224,252,72 by=clip drawn focus heard="MASTER SYSTEM" limit=132,224,1656,642 paints=#8D8D91FF role=tabitem sideways text="Master System"
            15 TextBlock at=658.5,234.5,216,51 box=144,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=143.44,33.6 limit=640.5,224,252,72 owner=14 paints=#8D8D91FF role=text size=28 text="MASTER SYSTEM"
          16 TabItem at=892.5,224,148.5,72 by=clip drawn focus heard=GENESIS limit=132,224,1656,642 op paints=#EEEEF0FF role=tabitem sideways text=Genesis
            17 TextBlock at=910.5,234.5,112.5,51 box=75,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=GENESIS layout=74.56,33.6 limit=892.5,224,148.5,72 owner=16 paints=#EEEEF0FF role=text size=28 text=GENESIS
          18 TabItem at=1041,224,150,72 by=clip drawn focus heard="SEGA CD" limit=132,224,1656,642 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
            19 TextBlock at=1059,234.5,114,51 box=76,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard="SEGA CD" layout=75.07,33.6 limit=1041,224,150,72 owner=18 paints=#8D8D91FF role=text size=28 text="SEGA CD"
          20 TabItem at=1191,224,208.5,72 by=clip drawn focus heard=PLAYSTATION limit=132,224,1656,642 paints=#8D8D91FF role=tabitem sideways text=PlayStation
            21 TextBlock at=1209,234.5,172.5,51 box=115,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.49,33.6 limit=1191,224,208.5,72 owner=20 paints=#8D8D91FF role=text size=28 text=PLAYSTATION
          22 TabItem at=1399.5,224,232.5,72 by=clip drawn focus heard="PLAYSTATION 2" limit=132,224,1656,642 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
            23 TextBlock at=1417.5,234.5,196.5,51 box=131,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=130.84,33.6 limit=1399.5,224,232.5,72 owner=22 paints=#8D8D91FF role=text size=28 text="PLAYSTATION 2"
          24 ShaderPanel at=150,311,1620,555 by=clip limit=132,299,1656,567 role=none
            25 Grid at=150,311,792,555 by=clip limit=132,299,1656,567 role=none
              26 FilterBar at=150,311,792,69 by=clip limit=132,299,1656,567 name=Genesis.ShaderSearch paints=#D4D4D4FF role=group
                27 TextBlock at=150,323,103.5,43.5 box=69,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=68.93,28.8 limit=150,311,792,69 paints=#8D8D91FF part role=text size=24 text=Category
                28 Dropdown at=265.5,311,270,66 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=combobox sideways
                  29 TextBlock at=283.5,318.5,204,48 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=19.79,31.2 limit=283.5,318.5,204,48 owner=28 paints=#D4D4D4FF role=text size=26 text=All
                  30 ViewboxContainer at=502.5,339.1,18,9.81 by=clip limit=502.5,339.1,18,9.81 nohit role=none
                31 TextBox at=547.5,311,394.5,69 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit
                  32 ScrollViewer allow="a text box scrolls its own text" at=564,321.5,361.5,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    33 TextBlock allow="a text box scrolls its own text" at=564,321.5,361.5,48 box=241,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=32 layout=58.32,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text=Search
              34 GroupedList at=150,392,792,393 by=clip limit=132,299,1656,567 name=Genesis.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                35 ScrollViewer at=150,392,792,393 name=PART_ScrollViewer overflow=0 paints=#1B1B1EFF,#FFFFFFFF part role=scrollviewer viewport=150,392,792,393
                  36 ListBoxItem at=150,392,792,129 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=35 limit=150,392,792,393 op paints=#333337FF,#D4D4D4FF role=listitem
                    37 GroupedListRow at=150,392,792,127.5 by=clip in=35 limit=150,392,792,129 role=none
                      38 DockPanel at=165,413,762,36 by=clip in=35 limit=150,392,792,129 role=none
                        39 TextBlock at=915,413,12,36 box=8,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=35 layout=7.88,24 limit=150,392,792,129 owner=36 paints=#8D8D91FF role=text size=20 text=5
                        40 TextBlock at=165,413,738,36 box=492,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=35 layout=124.7,24 limit=150,392,792,129 owner=36 paints=#8D8D91FF role=text size=20 text="BUILT INTO EMUSEN"
                      41 Border at=156,456.5,780,61.5 by=clip in=35 limit=150,392,792,129 paints=#4C9AE8FF role=none
                        42 DockPanel at=171,465.5,750,43.5 by=clip in=35 limit=150,392,792,129 role=none
                          43 Border at=838.5,468.5,82.5,39 by=clip in=35 limit=150,392,792,129 paints=#0E0E10FF role=none
                            44 TextBlock at=850.5,470,58.5,36 box=39,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=35 layout=39,24 limit=150,392,792,129 owner=36 paints=#4C9AE8FF role=text size=20 text="In use"
                          45 StackPanel at=171,465.5,655.5,43.5 by=clip in=35 limit=150,392,792,129 role=none
                            46 TextBlock at=171,465.5,655.5,43.5 box=437,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=35 layout=39.6,28.8 limit=150,392,792,129 owner=36 paints=#0E0E10FF role=text size=24 text=None
                  47 ListBoxItem at=150,521,792,66 by=fixed-scroll drawn focus heard=CRT in=35 limit=150,392,792,393 op paints=#333337FF,#D4D4D4FF role=listitem
                    48 GroupedListRow at=150,521,792,64.5 by=clip in=35 limit=150,521,792,66 role=none
                      49 Border at=156,522.5,780,61.5 by=clip in=35 limit=150,521,792,66 role=none
                        50 DockPanel at=171,531.5,750,43.5 by=clip in=35 limit=150,521,792,66 role=none
                          51 StackPanel at=171,531.5,750,43.5 by=clip in=35 limit=150,521,792,66 role=none
                            52 TextBlock at=171,531.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=35 layout=29.04,28.8 limit=150,521,792,66 owner=47 paints=#D4D4D4FF role=text size=24 text=CRT
                  53 ListBoxItem at=150,587,792,66 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=35 limit=150,392,792,393 op paints=#333337FF,#D4D4D4FF role=listitem
                    54 GroupedListRow at=150,587,792,64.5 by=clip in=35 limit=150,587,792,66 role=none
                      55 Border at=156,588.5,780,61.5 by=clip in=35 limit=150,587,792,66 role=none
                        56 DockPanel at=171,597.5,750,43.5 by=clip in=35 limit=150,587,792,66 role=none
                          57 StackPanel at=171,597.5,750,43.5 by=clip in=35 limit=150,587,792,66 role=none
                            58 TextBlock at=171,597.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=35 layout=92.06,28.8 limit=150,587,792,66 owner=53 paints=#D4D4D4FF role=text size=24 text="CRT (Lottes)"
                  59 ListBoxItem at=150,653,792,66 by=fixed-scroll drawn focus heard=Scanlines in=35 limit=150,392,792,393 op paints=#333337FF,#D4D4D4FF role=listitem
                    60 GroupedListRow at=150,653,792,64.5 by=clip in=35 limit=150,653,792,66 role=none
                      61 Border at=156,654.5,780,61.5 by=clip in=35 limit=150,653,792,66 role=none
                        62 DockPanel at=171,663.5,750,43.5 by=clip in=35 limit=150,653,792,66 role=none
                          63 StackPanel at=171,663.5,750,43.5 by=clip in=35 limit=150,653,792,66 role=none
                            64 TextBlock at=171,663.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=35 layout=73.66,28.8 limit=150,653,792,66 owner=59 paints=#D4D4D4FF role=text size=24 text=Scanlines
                  65 ListBoxItem at=150,719,792,66 by=fixed-scroll drawn focus heard="Simple CRT" in=35 limit=150,392,792,393 op paints=#333337FF,#D4D4D4FF role=listitem
                    66 GroupedListRow at=150,719,792,64.5 by=clip in=35 limit=150,719,792,66 role=none
                      67 Border at=156,720.5,780,61.5 by=clip in=35 limit=150,719,792,66 role=none
                        68 DockPanel at=171,729.5,750,43.5 by=clip in=35 limit=150,719,792,66 role=none
                          69 StackPanel at=171,729.5,750,43.5 by=clip in=35 limit=150,719,792,66 role=none
                            70 TextBlock at=171,729.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=35 layout=86.16,28.8 limit=150,719,792,66 owner=65 paints=#D4D4D4FF role=text size=24 text="Simple CRT"
              71 Border at=150,800,792,66 by=clip limit=132,299,1656,567 role=none
                72 Button at=150,800,264,66 by=clip drawn focus heard="DOWNLOAD PACK" limit=132,299,1656,567 name=Genesis.DownloadPack op paints=#707070FF,#D4D4D4FF role=button text="Download Pack"
                  73 TextBlock at=175.5,809,213,48 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=141.47,31.2 limit=150,800,264,66 owner=72 paints=#D4D4D4FF role=text size=26 text="DOWNLOAD PACK"
            74 DockPanel at=978,311,792,555 by=clip limit=132,299,1656,567 role=none
              75 StackPanel at=978,311,792,157.5 by=clip limit=132,299,1656,567 role=none
                76 Grid at=978,311,792,66 by=clip limit=132,299,1656,567 role=none
                  77 TextBlock at=978,311,444,66 box=296,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None layout=39.6,28.8 limit=132,299,1656,567 name=Genesis.ShaderName paints=#D4D4D4FF role=text size=24 text=None
                  78 StackPanel at=1440,311,330,66 by=clip limit=132,299,1656,567 role=none
                    79 Button at=1440,311,184.5,66 by=clip drawn heard="RESET ALL" limit=132,299,1656,567 name=Genesis.ResetShader off paints=#707070FF,#D4D4D4FF role=button text="Reset All"
                      80 TextBlock at=1465.5,320,133.5,48 box=89,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=88.17,31.2 limit=1440,311,184.5,66 off owner=79 paints=#707070FF role=text size=26 text="RESET ALL"
                    81 Button at=1636.5,311,133.5,66 by=clip drawn heard="IN USE" limit=132,299,1656,567 name=Genesis.UseShader off paints=#707070FF,#D4D4D4FF role=button text="In Use"
                      82 TextBlock at=1662,320,82.5,48 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="IN USE" layout=54.44,31.2 limit=1636.5,311,133.5,66 off owner=81 paints=#707070FF role=text size=26 text="IN USE"
                83 MonoText at=978,386,792,37.5 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen" layout=150.25,24.52 limit=132,299,1656,567 name=Genesis.ShaderPath paints=#D4D4D4FF role=text size=18 text="Built into EmuSen"
                84 HintText at=978,432.5,792,36 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="This shader has nothing to adjust." layout=219.16,24 limit=132,299,1656,567 paints=#8D8D91FF role=text size=20 text="This shader has nothing to adjust."
              85 SliderList at=978,480.5,792,385.5 by=clip limit=132,299,1656,567 name=Genesis.ShaderParameters paints=#D4D4D4FF role=list
                86 ScrollViewer at=978,480.5,792,385.5 name=PART_ScrollViewer overflow=0 paints=#1B1B1EFF,#D4D4D4FF part role=scrollviewer viewport=978,480.5,792,385.5
        87 ButtonBar at=1657.5,224,130.5,66 by=panel limit=96,206,1728,678 paints=#D4D4D4FF role=toolbar
          88 Button at=1657.5,224,130.5,66 by=clip drawn focus heard=CLOSE limit=1657.5,224,130.5,66 op paints=#707070FF,#D4D4D4FF role=button text=Close
            89 TextBlock at=1683,233,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1657.5,224,130.5,66 owner=88 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 88
edge 16 up 88
edge 16 down 28
edge 28 up 16
edge 28 down 36
edge 31 up 16
edge 31 down 36
edge 31 left 28
edge 31 right 16
edge 36 up 31
edge 36 down 47
edge 36 left same
edge 36 right 16
edge 47 up 36
edge 47 down 53
edge 47 left same
edge 47 right 16
edge 53 up 47
edge 53 down 59
edge 53 left same
edge 53 right 16
edge 59 up 53
edge 59 down 65
edge 59 left same
edge 59 right 16
edge 65 up 59
edge 65 down 72
edge 65 left same
edge 65 right 16
edge 72 up 65
edge 72 down same
edge 72 left same
edge 72 right 31
edge 88 up same
edge 88 down 31
edge 88 left 16
edge 88 right same
press pageup focus="Dropdown PART_Facet (Category)" tab="Master System" title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=Genesis title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=Genesis title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=Genesis title=Shaders top=ShaderSettingsWindow
press back top=none
