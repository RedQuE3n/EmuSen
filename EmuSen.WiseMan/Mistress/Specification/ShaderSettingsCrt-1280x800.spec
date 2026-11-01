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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,124,1152,465 by=panel limit=64,124,1152,465 paints=#D4D4D4FF role=none
    1 DockPanel at=88,136,1104,441 by=panel limit=64,124,1152,465 role=none
      2 Grid at=88,136,1104,441 by=panel limit=64,124,1152,465 role=none
        3 Tabs at=88,136,1104,441 by=panel limit=64,124,1152,465 name=ConsoleTabs paints=#D4D4D4FF role=tab
          4 TabItem at=88,136,60,48 by=clip drawn focus heard=NES limit=88,136,1104,441 paints=#8D8D91FF role=tabitem sideways text=NES
            5 TextBlock at=100,143,36,34 box=36,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=NES layout=35.2,33.6 limit=88,136,60,48 owner=4 paints=#8D8D91FF role=text size=28 text=NES
          6 TabItem at=148,136,49,48 by=clip drawn focus heard=GB limit=88,136,1104,441 paints=#8D8D91FF role=tabitem sideways text=GB
            7 TextBlock at=160,143,25,34 box=25,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=GB layout=24.05,33.6 limit=148,136,49,48 owner=6 paints=#8D8D91FF role=text size=28 text=GB
          8 TabItem at=197,136,71,48 by=clip drawn focus heard=SNES limit=88,136,1104,441 op paints=#EEEEF0FF role=tabitem sideways text=SNES
            9 TextBlock at=209,143,47,34 box=47,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=SNES layout=46.23,33.6 limit=197,136,71,48 owner=8 paints=#EEEEF0FF role=text size=28 text=SNES
          10 TabItem at=268,136,60,48 by=clip drawn focus heard=N64 limit=88,136,1104,441 paints=#8D8D91FF role=tabitem sideways text=N64
            11 TextBlock at=280,143,36,34 box=36,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=N64 layout=35.45,33.6 limit=268,136,60,48 owner=10 paints=#8D8D91FF role=text size=28 text=N64
          12 TabItem at=328,136,99,48 by=clip drawn focus heard=SG-1000 limit=88,136,1104,441 paints=#8D8D91FF role=tabitem sideways text=SG-1000
            13 TextBlock at=340,143,75,34 box=75,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=SG-1000 layout=74.48,33.6 limit=328,136,99,48 owner=12 paints=#8D8D91FF role=text size=28 text=SG-1000
          14 TabItem at=427,136,168,48 by=clip drawn focus heard="MASTER SYSTEM" limit=88,136,1104,441 paints=#8D8D91FF role=tabitem sideways text="Master System"
            15 TextBlock at=439,143,144,34 box=144,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=143.44,33.6 limit=427,136,168,48 owner=14 paints=#8D8D91FF role=text size=28 text="MASTER SYSTEM"
          16 TabItem at=595,136,99,48 by=clip drawn focus heard=GENESIS limit=88,136,1104,441 paints=#8D8D91FF role=tabitem sideways text=Genesis
            17 TextBlock at=607,143,75,34 box=75,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=GENESIS layout=74.56,33.6 limit=595,136,99,48 owner=16 paints=#8D8D91FF role=text size=28 text=GENESIS
          18 TabItem at=694,136,100,48 by=clip drawn focus heard="SEGA CD" limit=88,136,1104,441 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
            19 TextBlock at=706,143,76,34 box=76,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard="SEGA CD" layout=75.07,33.6 limit=694,136,100,48 owner=18 paints=#8D8D91FF role=text size=28 text="SEGA CD"
          20 TabItem at=794,136,139,48 by=clip drawn focus heard=PLAYSTATION limit=88,136,1104,441 paints=#8D8D91FF role=tabitem sideways text=PlayStation
            21 TextBlock at=806,143,115,34 box=115,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.49,33.6 limit=794,136,139,48 owner=20 paints=#8D8D91FF role=text size=28 text=PLAYSTATION
          22 TabItem at=933,136,155,48 by=clip drawn focus heard="PLAYSTATION 2" limit=88,136,1104,441 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
            23 TextBlock at=945,143,131,34 box=131,34 by=clip caps=19.5986 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=130.84,33.6 limit=933,136,155,48 owner=22 paints=#8D8D91FF role=text size=28 text="PLAYSTATION 2"
          24 ShaderPanel at=100,194,1080,383 by=clip limit=88,186,1104,391 role=none
            25 Grid at=100,194,528,383 by=clip limit=88,186,1104,391 role=none
              26 FilterBar at=100,194,528,46 by=clip limit=88,186,1104,391 name=SNES.ShaderSearch paints=#D4D4D4FF role=group
                27 TextBlock at=100,202,69,29 box=69,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=68.93,28.8 limit=100,194,528,46 paints=#8D8D91FF part role=text size=24 text=Category
                28 Dropdown at=177,194,180,44 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=combobox sideways
                  29 TextBlock at=189,199,136,32 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=19.79,31.2 limit=189,199,136,32 owner=28 paints=#D4D4D4FF role=text size=26 text=All
                  30 ViewboxContainer at=335,212.73,12,6.54 by=clip limit=335,212.73,12,6.54 nohit role=none
                31 TextBox at=365,194,263,46 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit
                  32 ScrollViewer allow="a text box scrolls its own text" at=376,201,241,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    33 TextBlock allow="a text box scrolls its own text" at=376,201,241,32 box=241,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=32 layout=58.32,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text=Search
              34 GroupedList at=100,248,528,262 by=clip limit=88,186,1104,391 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                35 ScrollViewer at=100,248,528,262 name=PART_ScrollViewer overflow=0 paints=#1B1B1EFF,#FFFFFFFF part role=scrollviewer viewport=100,248,528,262
                  36 ListBoxItem at=100,248,528,86 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=35 limit=100,248,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    37 GroupedListRow at=100,248,528,85 by=clip in=35 limit=100,248,528,86 role=none
                      38 DockPanel at=110,262,508,24 by=clip in=35 limit=100,248,528,86 role=none
                        39 TextBlock at=610,262,8,24 box=8,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=35 layout=7.88,24 limit=100,248,528,86 owner=36 paints=#8D8D91FF role=text size=20 text=5
                        40 TextBlock at=110,262,492,24 box=492,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=35 layout=124.7,24 limit=100,248,528,86 owner=36 paints=#8D8D91FF role=text size=20 text="BUILT INTO EMUSEN"
                      41 Border at=104,291,520,41 by=clip in=35 limit=100,248,528,86 role=none
                        42 DockPanel at=114,297,500,29 by=clip in=35 limit=100,248,528,86 role=none
                          43 Border at=559,299,55,26 by=clip in=35 limit=100,248,528,86 paints=#0E0E10FF role=none
                            44 TextBlock at=567,300,39,24 box=39,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=35 layout=39,24 limit=100,248,528,86 owner=36 paints=#D4D4D4FF role=text size=20 text="In use"
                          45 StackPanel at=114,297,437,29 by=clip in=35 limit=100,248,528,86 role=none
                            46 TextBlock at=114,297,437,29 box=437,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=35 layout=39.6,28.8 limit=100,248,528,86 owner=36 paints=#D4D4D4FF role=text size=24 text=None
                  47 ListBoxItem at=100,334,528,44 by=fixed-scroll drawn focus heard=CRT in=35 limit=100,248,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    48 GroupedListRow at=100,334,528,43 by=clip in=35 limit=100,334,528,44 role=none
                      49 Border at=104,335,520,41 by=clip in=35 limit=100,334,528,44 paints=#4C9AE8FF role=none
                        50 DockPanel at=114,341,500,29 by=clip in=35 limit=100,334,528,44 role=none
                          51 StackPanel at=114,341,500,29 by=clip in=35 limit=100,334,528,44 role=none
                            52 TextBlock at=114,341,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=35 layout=29.04,28.8 limit=100,334,528,44 owner=47 paints=#0E0E10FF role=text size=24 text=CRT
                  53 ListBoxItem at=100,378,528,44 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=35 limit=100,248,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    54 GroupedListRow at=100,378,528,43 by=clip in=35 limit=100,378,528,44 role=none
                      55 Border at=104,379,520,41 by=clip in=35 limit=100,378,528,44 role=none
                        56 DockPanel at=114,385,500,29 by=clip in=35 limit=100,378,528,44 role=none
                          57 StackPanel at=114,385,500,29 by=clip in=35 limit=100,378,528,44 role=none
                            58 TextBlock at=114,385,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=35 layout=92.06,28.8 limit=100,378,528,44 owner=53 paints=#D4D4D4FF role=text size=24 text="CRT (Lottes)"
                  59 ListBoxItem at=100,422,528,44 by=fixed-scroll drawn focus heard=Scanlines in=35 limit=100,248,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    60 GroupedListRow at=100,422,528,43 by=clip in=35 limit=100,422,528,44 role=none
                      61 Border at=104,423,520,41 by=clip in=35 limit=100,422,528,44 role=none
                        62 DockPanel at=114,429,500,29 by=clip in=35 limit=100,422,528,44 role=none
                          63 StackPanel at=114,429,500,29 by=clip in=35 limit=100,422,528,44 role=none
                            64 TextBlock at=114,429,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=35 layout=73.66,28.8 limit=100,422,528,44 owner=59 paints=#D4D4D4FF role=text size=24 text=Scanlines
                  65 ListBoxItem at=100,466,528,44 by=fixed-scroll drawn focus heard="Simple CRT" in=35 limit=100,248,528,262 op paints=#333337FF,#D4D4D4FF role=listitem
                    66 GroupedListRow at=100,466,528,43 by=clip in=35 limit=100,466,528,44 role=none
                      67 Border at=104,467,520,41 by=clip in=35 limit=100,466,528,44 role=none
                        68 DockPanel at=114,473,500,29 by=clip in=35 limit=100,466,528,44 role=none
                          69 StackPanel at=114,473,500,29 by=clip in=35 limit=100,466,528,44 role=none
                            70 TextBlock at=114,473,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=35 layout=86.16,28.8 limit=100,466,528,44 owner=65 paints=#D4D4D4FF role=text size=24 text="Simple CRT"
              71 Border at=100,520,528,57 by=clip limit=88,186,1104,391 role=none
                72 Button at=100,520,176,44 by=clip drawn focus heard="DOWNLOAD PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#707070FF,#D4D4D4FF role=button text="Download Pack"
                  73 TextBlock at=117,526,142,32 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=141.47,31.2 limit=100,520,176,44 owner=72 paints=#D4D4D4FF role=text size=26 text="DOWNLOAD PACK"
            74 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              75 StackPanel at=652,194,528,184 by=clip limit=88,186,1104,391 role=none
                76 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  77 TextBlock at=652,194,206,44 box=206,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT layout=29.04,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#D4D4D4FF role=text size=24 text=CRT
                  78 StackPanel at=870,194,310,44 by=clip limit=88,186,1104,391 role=none
                    79 Button at=870,194,123,44 by=clip drawn focus heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader op paints=#707070FF,#D4D4D4FF role=button text="Reset All"
                      80 TextBlock at=887,200,89,32 box=89,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=88.17,31.2 limit=870,194,123,44 owner=79 paints=#D4D4D4FF role=text size=26 text="RESET ALL"
                    81 Button at=1001,194,179,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#707070FF,#D4D4D4FF role=button text="Use This Shader"
                      82 TextBlock at=1018,200,145,32 box=145,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=144.69,31.2 limit=1001,194,179,44 owner=81 paints=#D4D4D4FF role=text size=26 text="USE THIS SHADER"
                83 MonoText at=652,244,528,50 box=528,50 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md" layout=467.8,49.03 limit=88,186,1104,391 name=SNES.ShaderPath paints=#D4D4D4FF role=text size=18 text="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md"
                84 HintText at=652,300,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="28 parameters." layout=98.54,24 limit=88,186,1104,391 paints=#8D8D91FF role=text size=20 text="28 parameters."
                85 FilterBar at=652,332,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#D4D4D4FF role=group
                  86 TextBox at=652,332,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit
                    87 ScrollViewer allow="a text box scrolls its own text" at=663,339,492,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                      88 TextBlock allow="a text box scrolls its own text" at=663,339,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=87 layout=161.3,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search parameters"
              89 SliderList at=652,386,528,191 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#D4D4D4FF role=list
                90 ScrollViewer at=652,386,528,191 fades=b name=PART_ScrollViewer overflow=2441 paints=#1B1B1EFF,#333337FF,#4C9AE8FF,#707070FF,#8D8D91FF,#D4D4D4FF,#EEEEF0FF part role=scrollviewer viewport=652,386,512,191
                  91 ContentPresenter at=652,386,498,94 by=scroll in=90 limit=652,386,512,191 role=none
                    92 SliderRow at=652,386,498,90 by=scroll in=90 limit=652,386,512,191 name=SNES.Parameter.quality role=none
                      93 StackPanel at=652,390,498,82 by=scroll in=90 limit=652,386,512,191 role=none
                        94 DockPanel at=652,390,498,30 by=scroll in=90 limit=652,386,512,191 role=none
                          95 Button at=1088,390,62,30 by=scroll drawn focus heard="Reset Quality" in=90 limit=652,386,512,191 op paints=#707070FF,#D4D4D4FF role=button text=Reset
                            96 TextBlock at=1099,393,40,24 box=40,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=90 layout=40,24 limit=1088,390,62,30 owner=95 paints=#D4D4D4FF role=text size=20 text=RESET
                          97 TextBlock at=1004,390,72,29 box=72,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Balanced in=90 layout=71.83,28.8 limit=652,386,512,191 paints=#D4D4D4FF role=text size=24 text=Balanced
                          98 TextBlock at=652,390,340,29 box=340,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Quality in=90 layout=52.37,28.8 limit=652,386,512,191 paints=#D4D4D4FF role=text size=24 text=Quality
                        99 Slider at=652,422,498,50 by=scroll drawn focus heard=Quality in=90 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          100 RepeatButton at=652,437,239,20 drawn in=90 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          101 RepeatButton at=911,437,239,20 drawn in=90 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  102 ContentPresenter at=652,480,498,94 by=scroll in=90 limit=652,386,512,191 role=none
                    103 SliderRow at=652,480,498,90 by=scroll in=90 limit=652,386,512,191 name=SNES.Parameter.signal role=none
                      104 StackPanel at=652,484,498,82 by=scroll in=90 limit=652,386,512,191 role=none
                        105 DockPanel at=652,484,498,30 by=scroll in=90 limit=652,386,512,191 role=none
                          106 Button at=1088,484,62,30 by=scroll drawn focus heard="Reset Signal" in=90 limit=652,386,512,191 op paints=#707070FF,#D4D4D4FF role=button text=Reset
                            107 TextBlock at=1099,487,40,24 box=40,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=90 layout=40,24 limit=1088,484,62,30 owner=106 paints=#D4D4D4FF role=text size=20 text=RESET
                          108 TextBlock at=994,484,82,29 box=82,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Composite in=90 layout=81.77,28.8 limit=652,386,512,191 paints=#D4D4D4FF role=text size=24 text=Composite
                          109 TextBlock at=652,484,330,29 box=330,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Signal in=90 layout=46.58,28.8 limit=652,386,512,191 paints=#D4D4D4FF role=text size=24 text=Signal
                        110 Slider at=652,516,498,50 by=scroll drawn focus heard=Signal in=90 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          111 RepeatButton at=652,531,478,20 drawn in=90 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                  112 ContentPresenter at=652,574,498,94 by=scroll in=90 limit=652,386,512,191 role=none seen=652,574,498,3
                    113 SliderRow at=652,574,498,90 by=scroll in=90 limit=652,386,512,191 name=SNES.Parameter.standard paints=#333337FF,#4C9AE8FF,#707070FF,#D4D4D4FF,#EEEEF0FF role=none seen=652,574,498,3
                  114 RepeatButton at=1164,386,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    115 ViewboxContainer at=1168,392,8,4.4 by=clip limit=1168,392,8,4.4 role=none
                  116 RepeatButton at=1164,442,16,119 drawn heard="Page down" name=PART_PageDownButton part role=button
                  117 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    118 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        119 ButtonBar at=1105,136,87,44 by=panel limit=64,124,1152,465 paints=#D4D4D4FF role=toolbar
          120 Button at=1105,136,87,44 by=clip drawn focus heard=CLOSE limit=1105,136,87,44 op paints=#707070FF,#D4D4D4FF role=button text=Close
            121 TextBlock at=1122,142,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1105,136,87,44 owner=120 paints=#EEEEF0FF role=text size=26 text=CLOSE
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
