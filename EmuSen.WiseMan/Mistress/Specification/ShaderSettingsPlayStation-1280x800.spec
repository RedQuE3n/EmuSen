# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsPlayStation-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,37,1152,655
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. Not downloaded. libretro's slang pack, about 54 MB from buildbot.libretro.com, is the file RetroArch's updater fetches: some 2,600 presets. EmuSen ships none of it, and each shader keeps its authors' licence."
case ShaderSettingsPlayStation
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,137,1152,452 by=panel limit=64,137,1152,452 paints=#D4D4D4FF role=none
    1 DockPanel at=88,149,1104,428 by=panel limit=64,137,1152,452 role=none
      2 Grid at=88,149,1104,428 by=panel limit=64,137,1152,452 role=none
        3 Tabs at=88,149,1104,428 by=panel limit=64,137,1152,452 name=ConsoleTabs paints=#D4D4D4FF role=tab
          4 TabItem at=88,149,60,48 by=clip drawn focus heard=NES limit=88,149,1104,428 paints=#8D8D91FF role=tabitem sideways text=NES
            5 TextBlock at=100,156,36,34 box=36,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=NES layout=35.2,33.6 limit=88,149,60,48 owner=4 paints=#8D8D91FF role=text size=28 text=NES
          6 TabItem at=148,149,49,48 by=clip drawn focus heard=GB limit=88,149,1104,428 paints=#8D8D91FF role=tabitem sideways text=GB
            7 TextBlock at=160,156,25,34 box=25,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=GB layout=24.05,33.6 limit=148,149,49,48 owner=6 paints=#8D8D91FF role=text size=28 text=GB
          8 TabItem at=197,149,71,48 by=clip drawn focus heard=SNES limit=88,149,1104,428 paints=#8D8D91FF role=tabitem sideways text=SNES
            9 TextBlock at=209,156,47,34 box=47,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=SNES layout=46.23,33.6 limit=197,149,71,48 owner=8 paints=#8D8D91FF role=text size=28 text=SNES
          10 TabItem at=268,149,60,48 by=clip drawn focus heard=N64 limit=88,149,1104,428 paints=#8D8D91FF role=tabitem sideways text=N64
            11 TextBlock at=280,156,36,34 box=36,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=N64 layout=35.45,33.6 limit=268,149,60,48 owner=10 paints=#8D8D91FF role=text size=28 text=N64
          12 TabItem at=328,149,99,48 by=clip drawn focus heard=SG-1000 limit=88,149,1104,428 paints=#8D8D91FF role=tabitem sideways text=SG-1000
            13 TextBlock at=340,156,75,34 box=75,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=SG-1000 layout=74.48,33.6 limit=328,149,99,48 owner=12 paints=#8D8D91FF role=text size=28 text=SG-1000
          14 TabItem at=427,149,168,48 by=clip drawn focus heard="MASTER SYSTEM" limit=88,149,1104,428 paints=#8D8D91FF role=tabitem sideways text="Master System"
            15 TextBlock at=439,156,144,34 box=144,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=143.44,33.6 limit=427,149,168,48 owner=14 paints=#8D8D91FF role=text size=28 text="MASTER SYSTEM"
          16 TabItem at=595,149,99,48 by=clip drawn focus heard=GENESIS limit=88,149,1104,428 paints=#8D8D91FF role=tabitem sideways text=Genesis
            17 TextBlock at=607,156,75,34 box=75,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=GENESIS layout=74.56,33.6 limit=595,149,99,48 owner=16 paints=#8D8D91FF role=text size=28 text=GENESIS
          18 TabItem at=694,149,100,48 by=clip drawn focus heard="SEGA CD" limit=88,149,1104,428 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
            19 TextBlock at=706,156,76,34 box=76,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard="SEGA CD" layout=75.07,33.6 limit=694,149,100,48 owner=18 paints=#8D8D91FF role=text size=28 text="SEGA CD"
          20 TabItem at=794,149,139,48 by=clip drawn focus heard=PLAYSTATION limit=88,149,1104,428 op paints=#EEEEF0FF role=tabitem sideways text=PlayStation
            21 TextBlock at=806,156,115,34 box=115,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.49,33.6 limit=794,149,139,48 owner=20 paints=#EEEEF0FF role=text size=28 text=PLAYSTATION
          22 TabItem at=933,149,155,48 by=clip drawn focus heard="PLAYSTATION 2" limit=88,149,1104,428 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
            23 TextBlock at=945,156,131,34 box=131,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=130.84,33.6 limit=933,149,155,48 owner=22 paints=#8D8D91FF role=text size=28 text="PLAYSTATION 2"
          24 ShaderPanel at=100,207,1080,370 by=clip limit=88,199,1104,378 role=none
            25 Grid at=100,207,528,370 by=clip limit=88,199,1104,378 role=none
              26 FilterBar at=100,207,528,46 by=clip limit=88,199,1104,378 name=PlayStation.ShaderSearch paints=#D4D4D4FF role=group
                27 TextBlock at=100,215,69,29 box=69,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=68.93,28.8 limit=100,207,528,46 paints=#8D8D91FF part role=text size=24 text=Category
                28 Dropdown at=177,207,180,44 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=combobox sideways
                  29 TextBlock at=189,212,136,32 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=19.79,31.2 limit=189,212,136,32 owner=28 paints=#D4D4D4FF role=text size=26 text=All
                  30 ViewboxContainer at=335,225.73,12,6.54 by=clip limit=335,225.73,12,6.54 nohit role=none
                31 TextBox at=365,207,263,46 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit
                  32 ScrollViewer allow="a text box scrolls its own text" at=376,214,241,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    33 TextBlock allow="a text box scrolls its own text" at=376,214,241,32 box=241,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=32 layout=58.32,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text=Search
              34 GroupedList at=100,261,528,262 by=clip limit=88,199,1104,378 name=PlayStation.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                35 ScrollViewer at=100,261,528,262 name=PART_ScrollViewer overflow=0 paints=#1B1B1EFF,#FFFFFFFF part role=scrollviewer viewport=100,261,528,262
                  36 ListBoxItem at=100,261,528,86 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=35 limit=100,261,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    37 GroupedListRow at=100,261,528,85 by=clip in=35 limit=100,261,528,86 role=none
                      38 DockPanel at=110,275,508,24 by=clip in=35 limit=100,261,528,86 role=none
                        39 TextBlock at=610,275,8,24 box=8,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=35 layout=7.88,24 limit=100,261,528,86 owner=36 paints=#8D8D91FF role=text size=20 text=5
                        40 TextBlock at=110,275,492,24 box=492,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=35 layout=124.7,24 limit=100,261,528,86 owner=36 paints=#8D8D91FF role=text size=20 text="BUILT INTO EMUSEN"
                      41 Border at=104,304,520,41 by=clip in=35 limit=100,261,528,86 paints=#4C9AE8FF role=none
                        42 DockPanel at=114,310,500,29 by=clip in=35 limit=100,261,528,86 role=none
                          43 Border at=559,312,55,26 by=clip in=35 limit=100,261,528,86 paints=#0E0E10FF role=none
                            44 TextBlock at=567,313,39,24 box=39,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=35 layout=39,24 limit=100,261,528,86 owner=36 paints=#4C9AE8FF role=text size=20 text="In use"
                          45 StackPanel at=114,310,437,29 by=clip in=35 limit=100,261,528,86 role=none
                            46 TextBlock at=114,310,437,29 box=437,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=35 layout=39.6,28.8 limit=100,261,528,86 owner=36 paints=#0E0E10FF role=text size=24 text=None
                  47 ListBoxItem at=100,347,528,44 by=fixed-scroll drawn focus heard=CRT in=35 limit=100,261,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    48 GroupedListRow at=100,347,528,43 by=clip in=35 limit=100,347,528,44 role=none
                      49 Border at=104,348,520,41 by=clip in=35 limit=100,347,528,44 role=none
                        50 DockPanel at=114,354,500,29 by=clip in=35 limit=100,347,528,44 role=none
                          51 StackPanel at=114,354,500,29 by=clip in=35 limit=100,347,528,44 role=none
                            52 TextBlock at=114,354,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=35 layout=29.04,28.8 limit=100,347,528,44 owner=47 paints=#D4D4D4FF role=text size=24 text=CRT
                  53 ListBoxItem at=100,391,528,44 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=35 limit=100,261,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    54 GroupedListRow at=100,391,528,43 by=clip in=35 limit=100,391,528,44 role=none
                      55 Border at=104,392,520,41 by=clip in=35 limit=100,391,528,44 role=none
                        56 DockPanel at=114,398,500,29 by=clip in=35 limit=100,391,528,44 role=none
                          57 StackPanel at=114,398,500,29 by=clip in=35 limit=100,391,528,44 role=none
                            58 TextBlock at=114,398,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=35 layout=92.06,28.8 limit=100,391,528,44 owner=53 paints=#D4D4D4FF role=text size=24 text="CRT (Lottes)"
                  59 ListBoxItem at=100,435,528,44 by=fixed-scroll drawn focus heard=Scanlines in=35 limit=100,261,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    60 GroupedListRow at=100,435,528,43 by=clip in=35 limit=100,435,528,44 role=none
                      61 Border at=104,436,520,41 by=clip in=35 limit=100,435,528,44 role=none
                        62 DockPanel at=114,442,500,29 by=clip in=35 limit=100,435,528,44 role=none
                          63 StackPanel at=114,442,500,29 by=clip in=35 limit=100,435,528,44 role=none
                            64 TextBlock at=114,442,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=35 layout=73.66,28.8 limit=100,435,528,44 owner=59 paints=#D4D4D4FF role=text size=24 text=Scanlines
                  65 ListBoxItem at=100,479,528,44 by=fixed-scroll drawn focus heard="Simple CRT" in=35 limit=100,261,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    66 GroupedListRow at=100,479,528,43 by=clip in=35 limit=100,479,528,44 role=none
                      67 Border at=104,480,520,41 by=clip in=35 limit=100,479,528,44 role=none
                        68 DockPanel at=114,486,500,29 by=clip in=35 limit=100,479,528,44 role=none
                          69 StackPanel at=114,486,500,29 by=clip in=35 limit=100,479,528,44 role=none
                            70 TextBlock at=114,486,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=35 layout=86.16,28.8 limit=100,479,528,44 owner=65 paints=#D4D4D4FF role=text size=24 text="Simple CRT"
              71 Border at=100,533,528,44 by=clip limit=88,199,1104,378 role=none
                72 Button at=100,533,176,44 by=clip drawn focus heard="DOWNLOAD PACK" limit=88,199,1104,378 name=PlayStation.DownloadPack op paints=#707070FF,#D4D4D4FF role=button text="Download Pack"
                  73 TextBlock at=117,539,142,32 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=141.47,31.2 limit=100,533,176,44 owner=72 paints=#D4D4D4FF role=text size=26 text="DOWNLOAD PACK"
            74 DockPanel at=652,207,528,370 by=clip limit=88,199,1104,378 role=none
              75 StackPanel at=652,207,528,105 by=clip limit=88,199,1104,378 role=none
                76 Grid at=652,207,528,44 by=clip limit=88,199,1104,378 role=none
                  77 TextBlock at=652,207,296,44 box=296,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None layout=39.6,28.8 limit=88,199,1104,378 name=PlayStation.ShaderName paints=#D4D4D4FF role=text size=24 text=None
                  78 StackPanel at=960,207,220,44 by=clip limit=88,199,1104,378 role=none
                    79 Button at=960,207,123,44 by=clip drawn heard="RESET ALL" limit=88,199,1104,378 name=PlayStation.ResetShader off paints=#707070FF,#D4D4D4FF role=button text="Reset All"
                      80 TextBlock at=977,213,89,32 box=89,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=88.17,31.2 limit=960,207,123,44 off owner=79 paints=#707070FF role=text size=26 text="RESET ALL"
                    81 Button at=1091,207,89,44 by=clip drawn heard="IN USE" limit=88,199,1104,378 name=PlayStation.UseShader off paints=#707070FF,#D4D4D4FF role=button text="In Use"
                      82 TextBlock at=1108,213,55,32 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="IN USE" layout=54.44,31.2 limit=1091,207,89,44 off owner=81 paints=#707070FF role=text size=26 text="IN USE"
                83 MonoText at=652,257,528,25 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen" layout=150.25,24.52 limit=88,199,1104,378 name=PlayStation.ShaderPath paints=#D4D4D4FF role=text size=18 text="Built into EmuSen"
                84 HintText at=652,288,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="This shader has nothing to adjust." layout=219.16,24 limit=88,199,1104,378 paints=#8D8D91FF role=text size=20 text="This shader has nothing to adjust."
              85 SliderList at=652,320,528,257 by=clip limit=88,199,1104,378 name=PlayStation.ShaderParameters paints=#D4D4D4FF role=list
                86 ScrollViewer at=652,320,528,257 name=PART_ScrollViewer overflow=0 paints=#1B1B1EFF,#D4D4D4FF part role=scrollviewer viewport=652,320,528,257
        87 ButtonBar at=1105,149,87,44 by=panel limit=64,137,1152,452 paints=#D4D4D4FF role=toolbar
          88 Button at=1105,149,87,44 by=clip drawn focus heard=CLOSE limit=1105,149,87,44 op paints=#707070FF,#D4D4D4FF role=button text=Close
            89 TextBlock at=1122,155,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1105,149,87,44 owner=88 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 88
edge 20 up 88
edge 20 down 28
edge 28 up 20
edge 28 down 36
edge 31 up 20
edge 31 down 36
edge 31 left 28
edge 31 right 20
edge 36 up 31
edge 36 down 47
edge 36 left same
edge 36 right 20
edge 47 up 36
edge 47 down 53
edge 47 left same
edge 47 right 20
edge 53 up 47
edge 53 down 59
edge 53 left same
edge 53 right 20
edge 59 up 53
edge 59 down 65
edge 59 left same
edge 59 right 20
edge 65 up 59
edge 65 down 72
edge 65 left same
edge 65 right 20
edge 72 up 65
edge 72 down same
edge 72 left same
edge 72 right 31
edge 88 up same
edge 88 down 31
edge 88 left 20
edge 88 right same
press pageup focus="Dropdown PART_Facet (Category)" tab="Sega CD" title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=PlayStation title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=PlayStation title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=PlayStation title=Shaders top=ShaderSettingsWindow
press back top=none
