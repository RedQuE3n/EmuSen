# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsCrt-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,24,1152,668
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. Not downloaded. libretro's slang pack, about 54 MB from buildbot.libretro.com, is the file RetroArch's updater fetches: some 2,600 presets. EmuSen ships none of it, and each shader keeps its authors' licence."
case ShaderSettingsCrt
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,124,1152,465 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=none
    1 DockPanel at=88,136,1104,441 by=panel limit=64,124,1152,465 role=none
      2 Grid at=88,136,1104,441 by=panel limit=64,124,1152,465 role=none
        3 Tabs at=88,136,1104,441 by=panel limit=64,124,1152,465 name=ConsoleTabs paints=#9C9CA0FF role=tab
          4 TabItem at=88,136,60,48 by=clip drawn focus heard=NES limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=NES
            5 TextBlock at=100,144,36,32 box=36,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=NES layout=35.46,31.8 limit=88,136,60,48 owner=4 paints=#86868AFF role=text size=26.5 text=NES
          6 TabItem at=148,136,48,48 by=clip drawn focus heard=GB limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=GB
            7 TextBlock at=160,144,24,32 box=24,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=GB layout=23.85,31.8 limit=148,136,48,48 owner=6 paints=#86868AFF role=text size=26.5 text=GB
          8 TabItem at=196,136,71,48 by=clip drawn focus heard=SNES limit=88,136,1104,441 op paints=#EEEEF0FF role=tabitem sideways text=SNES
            9 TextBlock at=208,144,47,32 box=47,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=SNES layout=46.3,31.8 limit=196,136,71,48 owner=8 paints=#EEEEF0FF role=text size=26.5 text=SNES
          10 TabItem at=267,136,59,48 by=clip drawn focus heard=N64 limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=N64
            11 TextBlock at=279,144,35,32 box=35,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=N64 layout=34.93,31.8 limit=267,136,59,48 owner=10 paints=#86868AFF role=text size=26.5 text=N64
          12 TabItem at=326,136,98,48 by=clip drawn focus heard=SG-1000 limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=SG-1000
            13 TextBlock at=338,144,74,32 box=74,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=SG-1000 layout=73.03,31.8 limit=326,136,98,48 owner=12 paints=#86868AFF role=text size=26.5 text=SG-1000
          14 TabItem at=424,136,167,48 by=clip drawn focus heard="MASTER SYSTEM" limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text="Master System"
            15 TextBlock at=436,144,143,32 box=143,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=142.91,31.8 limit=424,136,167,48 owner=14 paints=#86868AFF role=text size=26.5 text="MASTER SYSTEM"
          16 TabItem at=591,136,100,48 by=clip drawn focus heard=GENESIS limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=Genesis
            17 TextBlock at=603,144,76,32 box=76,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=GENESIS layout=75.5,31.8 limit=591,136,100,48 owner=16 paints=#86868AFF role=text size=26.5 text=GENESIS
          18 TabItem at=691,136,99,48 by=clip drawn focus heard="SEGA CD" limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text="Sega CD"
            19 TextBlock at=703,144,75,32 box=75,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard="SEGA CD" layout=74.36,31.8 limit=691,136,99,48 owner=18 paints=#86868AFF role=text size=26.5 text="SEGA CD"
          20 TabItem at=790,136,141,48 by=clip drawn focus heard=PLAYSTATION limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=PlayStation
            21 TextBlock at=802,144,117,32 box=117,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=PLAYSTATION layout=116.52,31.8 limit=790,136,141,48 owner=20 paints=#86868AFF role=text size=26.5 text=PLAYSTATION
          22 TabItem at=931,136,157,48 by=clip drawn focus heard="PLAYSTATION 2" limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text="PlayStation 2"
            23 TextBlock at=943,144,133,32 box=133,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=132.47,31.8 limit=931,136,157,48 owner=22 paints=#86868AFF role=text size=26.5 text="PLAYSTATION 2"
          24 ShaderPanel at=100,194,1080,383 by=clip limit=88,186,1104,391 role=none
            25 Grid at=100,194,528,383 by=clip limit=88,186,1104,391 role=none
              26 FilterBar at=100,194,528,46 by=clip limit=88,186,1104,391 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                27 TextBlock at=100,202,73,29 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=100,194,528,46 paints=#86868AFF part role=text size=24 text=Category
                28 Dropdown at=181,194,180,44 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  29 TextBlock at=193,199,136,32 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=193,199,136,32 owner=28 paints=#9C9CA0FF role=text size=26 text=All
                  30 ViewboxContainer at=339,212.73,12,6.54 by=clip limit=339,212.73,12,6.54 nohit role=none
                31 TextBox at=369,194,259,46 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  32 ScrollViewer allow="a text box scrolls its own text" at=380,201,237,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    33 TextBlock allow="a text box scrolls its own text" at=380,201,237,32 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=32 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              34 GroupedList at=100,248,528,262 by=clip limit=88,186,1104,391 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                35 ScrollViewer at=100,248,528,262 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#FFFFFFFF part role=scrollviewer viewport=100,248,528,262
                  36 ListBoxItem at=100,248,528,86 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=35 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    37 GroupedListRow at=100,248,528,85 by=clip in=35 limit=100,248,528,86 role=none
                      38 DockPanel at=110,262,508,24 by=clip in=35 limit=100,248,528,86 role=none
                        39 TextBlock at=609,262,9,24 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=35 layout=8.18,24 limit=100,248,528,86 owner=36 paints=#86868AFF role=text size=20 text=5
                        40 TextBlock at=110,262,491,24 box=491,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=35 layout=132.18,24 limit=100,248,528,86 owner=36 paints=#86868AFF role=text size=20 text="BUILT INTO EMUSEN"
                      41 Border at=104,291,520,41 by=clip in=35 limit=100,248,528,86 role=none
                        42 DockPanel at=114,297,500,29 by=clip in=35 limit=100,248,528,86 role=none
                          43 Border at=557,299,57,26 by=clip in=35 limit=100,248,528,86 paints=#0E0E10FF role=none
                            44 TextBlock at=565,300,41,24 box=41,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=35 layout=40.88,24 limit=100,248,528,86 owner=36 paints=#9C9CA0FF role=text size=20 text="In use"
                          45 StackPanel at=114,297,435,29 by=clip in=35 limit=100,248,528,86 role=none
                            46 TextBlock at=114,297,435,29 box=435,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=35 layout=41.74,28.8 limit=100,248,528,86 owner=36 paints=#9C9CA0FF role=text size=24 text=None
                  47 ListBoxItem at=100,334,528,44 by=fixed-scroll drawn focus heard=CRT in=35 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    48 GroupedListRow at=100,334,528,43 by=clip in=35 limit=100,334,528,44 role=none
                      49 Border at=104,335,520,41 by=clip in=35 limit=100,334,528,44 paints=#4C9AE8FF role=none
                        50 DockPanel at=114,341,500,29 by=clip in=35 limit=100,334,528,44 role=none
                          51 StackPanel at=114,341,500,29 by=clip in=35 limit=100,334,528,44 role=none
                            52 TextBlock at=114,341,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=35 layout=31.49,28.8 limit=100,334,528,44 owner=47 paints=#FFFFFFFF role=text size=24 text=CRT
                  53 ListBoxItem at=100,378,528,44 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=35 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    54 GroupedListRow at=100,378,528,43 by=clip in=35 limit=100,378,528,44 role=none
                      55 Border at=104,379,520,41 by=clip in=35 limit=100,378,528,44 role=none
                        56 DockPanel at=114,385,500,29 by=clip in=35 limit=100,378,528,44 role=none
                          57 StackPanel at=114,385,500,29 by=clip in=35 limit=100,378,528,44 role=none
                            58 TextBlock at=114,385,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=35 layout=97.92,28.8 limit=100,378,528,44 owner=53 paints=#9C9CA0FF role=text size=24 text="CRT (Lottes)"
                  59 ListBoxItem at=100,422,528,44 by=fixed-scroll drawn focus heard=Scanlines in=35 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    60 GroupedListRow at=100,422,528,43 by=clip in=35 limit=100,422,528,44 role=none
                      61 Border at=104,423,520,41 by=clip in=35 limit=100,422,528,44 role=none
                        62 DockPanel at=114,429,500,29 by=clip in=35 limit=100,422,528,44 role=none
                          63 StackPanel at=114,429,500,29 by=clip in=35 limit=100,422,528,44 role=none
                            64 TextBlock at=114,429,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=35 layout=77.93,28.8 limit=100,422,528,44 owner=59 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  65 ListBoxItem at=100,466,528,44 by=fixed-scroll drawn focus heard="Simple CRT" in=35 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    66 GroupedListRow at=100,466,528,43 by=clip in=35 limit=100,466,528,44 role=none
                      67 Border at=104,467,520,41 by=clip in=35 limit=100,466,528,44 role=none
                        68 DockPanel at=114,473,500,29 by=clip in=35 limit=100,466,528,44 role=none
                          69 StackPanel at=114,473,500,29 by=clip in=35 limit=100,466,528,44 role=none
                            70 TextBlock at=114,473,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=35 layout=91.2,28.8 limit=100,466,528,44 owner=65 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
              71 Border at=100,520,528,57 by=clip limit=88,186,1104,391 role=none
                72 Button at=100,520,182,44 by=clip drawn focus heard="DOWNLOAD PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Download Pack"
                  73 TextBlock at=117,526,148,32 box=148,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=147.39,31.2 limit=100,520,182,44 owner=72 paints=#9C9CA0FF role=text size=26 text="DOWNLOAD PACK"
            74 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              75 StackPanel at=652,194,528,184 by=clip limit=88,186,1104,391 role=none
                76 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  77 TextBlock at=652,194,195,44 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT layout=31.49,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=CRT
                  78 StackPanel at=859,194,321,44 by=clip limit=88,186,1104,391 role=none
                    79 Button at=859,194,126,44 by=clip drawn focus heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader op paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      80 TextBlock at=876,200,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=859,194,126,44 owner=79 paints=#9C9CA0FF role=text size=26 text="RESET ALL"
                    81 Button at=993,194,187,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      82 TextBlock at=1010,200,153,32 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=993,194,187,44 owner=81 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                83 MonoText at=652,244,528,50 box=528,50 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md" layout=467.8,49.03 limit=88,186,1104,391 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md"
                84 HintText at=652,300,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="28 parameters." layout=102.98,24 limit=88,186,1104,391 paints=#86868AFF role=text size=20 text="28 parameters."
                85 FilterBar at=652,332,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  86 TextBox at=652,332,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    87 ScrollViewer allow="a text box scrolls its own text" at=663,339,492,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      88 TextBlock allow="a text box scrolls its own text" at=663,339,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=87 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              89 SliderList at=652,386,528,191 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                90 ScrollViewer at=652,386,528,191 fades=b name=PART_ScrollViewer overflow=2441 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#86868AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=652,386,512,191
                  91 ContentPresenter at=652,386,498,94 by=scroll in=90 limit=652,386,512,191 role=none
                    92 SliderRow at=652,386,498,90 by=scroll in=90 limit=652,386,512,191 name=SNES.Parameter.quality role=none
                      93 StackPanel at=652,390,498,82 by=scroll in=90 limit=652,386,512,191 role=none
                        94 DockPanel at=652,390,498,30 by=scroll in=90 limit=652,386,512,191 role=none
                          95 Button at=1085,390,65,30 by=scroll drawn focus heard="Reset Quality" in=90 limit=652,386,512,191 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            96 TextBlock at=1096,393,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=90 layout=42.66,24 limit=1085,390,65,30 owner=95 paints=#9C9CA0FF role=text size=20 text=RESET
                          97 TextBlock at=998,390,75,29 box=75,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Balanced in=90 layout=74.69,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Balanced
                          98 TextBlock at=652,390,334,29 box=334,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Quality in=90 layout=55.3,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Quality
                        99 Slider at=652,422,498,50 by=scroll drawn focus heard=Quality in=90 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          100 RepeatButton at=652,437,239,20 drawn in=90 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          101 RepeatButton at=911,437,239,20 drawn in=90 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  102 ContentPresenter at=652,480,498,94 by=scroll in=90 limit=652,386,512,191 role=none
                    103 SliderRow at=652,480,498,90 by=scroll in=90 limit=652,386,512,191 name=SNES.Parameter.signal role=none
                      104 StackPanel at=652,484,498,82 by=scroll in=90 limit=652,386,512,191 role=none
                        105 DockPanel at=652,484,498,30 by=scroll in=90 limit=652,386,512,191 role=none
                          106 Button at=1085,484,65,30 by=scroll drawn focus heard="Reset Signal" in=90 limit=652,386,512,191 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            107 TextBlock at=1096,487,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=90 layout=42.66,24 limit=1085,484,65,30 owner=106 paints=#9C9CA0FF role=text size=20 text=RESET
                          108 TextBlock at=986,484,87,29 box=87,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Composite in=90 layout=86.3,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Composite
                          109 TextBlock at=652,484,322,29 box=322,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Signal in=90 layout=49.32,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Signal
                        110 Slider at=652,516,498,50 by=scroll drawn focus heard=Signal in=90 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          111 RepeatButton at=652,531,478,20 drawn in=90 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                  112 ContentPresenter at=652,574,498,94 by=scroll in=90 limit=652,386,512,191 role=none seen=652,574,498,3
                    113 SliderRow at=652,574,498,90 by=scroll in=90 limit=652,386,512,191 name=SNES.Parameter.standard paints=#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=none seen=652,574,498,3
                  114 RepeatButton at=1164,386,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    115 ViewboxContainer at=1168,392,8,4.4 by=clip limit=1168,392,8,4.4 role=none
                  116 RepeatButton at=1164,442,16,119 drawn heard="Page down" name=PART_PageDownButton part role=button
                  117 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    118 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        119 ButtonBar at=1102,136,90,44 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=toolbar
          120 Button at=1102,136,90,44 by=clip drawn focus heard=CLOSE limit=1102,136,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            121 TextBlock at=1119,142,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,136,90,44 owner=120 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 120
edge 8 up 120
edge 8 down 28
edge 28 up 8
edge 28 down 36
edge 31 up 8
edge 31 down 36
edge 31 left 28
edge 31 right 79
edge 36 up 31
edge 36 down 47
edge 36 left same
edge 36 right 99
edge 47 up 36
edge 47 down 53
edge 47 left same
edge 47 right 86
edge 53 up 47
edge 53 down 59
edge 53 left same
edge 53 right 95
edge 59 up 53
edge 59 down 65
edge 59 left same
edge 59 right 86
edge 65 up 59
edge 65 down 72
edge 65 left same
edge 65 right 106
edge 72 up 65
edge 72 down "item:89/2/0/0/1/Television standard"
edge 72 left same
edge 72 right 110
edge 79 up 8
edge 79 down 86
edge 79 left 31
edge 79 right 81
edge 81 up 8
edge 81 down 86
edge 81 left 79
edge 81 right same
edge 86 up 79
edge 86 down 95
edge 86 left 47
edge 86 right same
edge 95 up 86
edge 95 down 99
edge 95 left 53
edge 95 right same
edge 99 up 95
edge 99 down 106
edge 106 up 99
edge 106 down 110
edge 106 left 65
edge 106 right same
edge 110 up 106
edge 110 down "item:89/2/0/0/0/0/Reset Television standard"
edge 120 up same
edge 120 down 81
edge 120 left 8
edge 120 right same
edge "item:89/2/0/0/0/0/Reset Television standard" up 110
edge "item:89/2/0/0/0/0/Reset Television standard" down "item:89/2/0/0/1/Television standard"
edge "item:89/2/0/0/0/0/Reset Television standard" left 65
edge "item:89/2/0/0/0/0/Reset Television standard" right same
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
