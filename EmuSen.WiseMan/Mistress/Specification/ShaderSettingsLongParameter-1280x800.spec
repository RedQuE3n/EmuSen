# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsLongParameter-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,24,1152,668
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. RetroArch: 5 presets, pack built 2026-09-22 02:00 UTC. The presets are in <run>/Home/Shaders/RetroArch."
case ShaderSettingsLongParameter
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
              34 GroupedList at=100,248,528,275 by=clip limit=88,186,1104,391 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                35 ScrollViewer at=100,248,528,275 fades=t name=PART_ScrollViewer overflow=375 paints=#1B1B1EFF,#707070FF,#FFFFFFFF part role=scrollviewer viewport=100,248,512,275
                  36 ListBoxItem at=100,221,512,86 by=scroll drawn focus heard="crt: crt-lottes" in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem seen=100,248,512,59
                    37 GroupedListRow at=100,221,512,85 by=clip in=35 limit=100,221,512,86 role=none seen=100,248,512,58
                      38 DockPanel at=110,235,492,24 by=clip in=35 limit=100,221,512,86 role=none seen=110,248,492,11
                        39 TextBlock at=594,235,8,24 box=8,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=2 in=35 layout=7.68,24 limit=100,221,512,86 owner=36 paints=#8D8D91FF role=text seen=594,248,8,11 size=20 text=2
                        40 TextBlock at=110,235,476,24 box=476,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=CRT in=35 layout=24.2,24 limit=100,221,512,86 owner=36 paints=#8D8D91FF role=text seen=110,248,476,11 size=20 text=CRT
                      41 Border at=104,264,504,41 by=clip in=35 limit=100,221,512,86 role=none
                        42 DockPanel at=114,270,484,29 by=clip in=35 limit=100,221,512,86 role=none
                          43 StackPanel at=114,270,484,29 by=clip in=35 limit=100,221,512,86 role=none
                            44 TextBlock at=114,270,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-lottes in=35 layout=72,28.8 limit=100,221,512,86 owner=36 paints=#D4D4D4FF role=text size=24 text=crt-lottes
                  45 ListBoxItem at=100,307,512,44 by=scroll drawn focus heard=crt-royale-kurozumi in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem
                    46 GroupedListRow at=100,307,512,43 by=clip in=35 limit=100,307,512,44 role=none
                      47 Border at=104,308,504,41 by=clip in=35 limit=100,307,512,44 role=none
                        48 DockPanel at=114,314,484,29 by=clip in=35 limit=100,307,512,44 role=none
                          49 StackPanel at=114,314,484,29 by=clip in=35 limit=100,307,512,44 role=none
                            50 TextBlock at=114,314,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-royale-kurozumi in=35 layout=157.25,28.8 limit=100,307,512,44 owner=45 paints=#D4D4D4FF role=text size=24 text=crt-royale-kurozumi
                  51 ListBoxItem at=100,351,512,86 by=scroll drawn focus heard="handheld: lcd-grid-v2" in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem
                    52 GroupedListRow at=100,351,512,85 by=clip in=35 limit=100,351,512,86 role=none
                      53 DockPanel at=110,365,492,24 by=clip in=35 limit=100,351,512,86 role=none
                        54 TextBlock at=596,365,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=35 layout=5.24,24 limit=100,351,512,86 owner=51 paints=#8D8D91FF role=text size=20 text=1
                        55 TextBlock at=110,365,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=HANDHELD in=35 layout=68.48,24 limit=100,351,512,86 owner=51 paints=#8D8D91FF role=text size=20 text=HANDHELD
                      56 Border at=104,394,504,41 by=clip in=35 limit=100,351,512,86 role=none
                        57 DockPanel at=114,400,484,29 by=clip in=35 limit=100,351,512,86 role=none
                          58 StackPanel at=114,400,484,29 by=clip in=35 limit=100,351,512,86 role=none
                            59 TextBlock at=114,400,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=lcd-grid-v2 in=35 layout=87.43,28.8 limit=100,351,512,86 owner=51 paints=#D4D4D4FF role=text size=24 text=lcd-grid-v2
                  60 ListBoxItem at=100,437,512,86 by=scroll drawn focus heard="long: long-names" in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem
                    61 GroupedListRow at=100,437,512,85 by=clip in=35 limit=100,437,512,86 role=none
                      62 DockPanel at=110,451,492,24 by=clip in=35 limit=100,437,512,86 role=none
                        63 TextBlock at=596,451,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=35 layout=5.24,24 limit=100,437,512,86 owner=60 paints=#8D8D91FF role=text size=20 text=1
                        64 TextBlock at=110,451,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=LONG in=35 layout=34.3,24 limit=100,437,512,86 owner=60 paints=#8D8D91FF role=text size=20 text=LONG
                      65 Border at=104,480,504,41 by=clip in=35 limit=100,437,512,86 paints=#4C9AE8FF role=none
                        66 DockPanel at=114,486,484,29 by=clip in=35 limit=100,437,512,86 role=none
                          67 StackPanel at=114,486,484,29 by=clip in=35 limit=100,437,512,86 role=none
                            68 TextBlock at=114,486,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names in=35 layout=92.86,28.8 limit=100,437,512,86 owner=60 paints=#0E0E10FF role=text size=24 text=long-names
                  69 RepeatButton at=612,248,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    70 ViewboxContainer at=616,254,8,4.4 by=clip limit=616,254,8,4.4 role=none
                  71 RepeatButton at=612,264,16,141 drawn heard="Page up" name=PART_PageUpButton part role=button
                  72 RepeatButton at=612,507,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    73 ViewboxContainer at=616,513,8,4.4 by=clip limit=616,513,8,4.4 role=none
              74 Border at=100,533,528,44 by=clip limit=88,186,1104,391 role=none
                75 Button at=100,533,146,44 by=clip drawn focus heard="UPDATE PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#707070FF,#D4D4D4FF role=button text="Update Pack"
                  76 TextBlock at=117,539,112,32 box=112,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="UPDATE PACK" layout=111.33,31.2 limit=100,533,146,44 owner=75 paints=#D4D4D4FF role=text size=26 text="UPDATE PACK"
            77 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              78 StackPanel at=652,194,528,159 by=clip limit=88,186,1104,391 role=none
                79 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  80 TextBlock at=652,194,206,44 box=206,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names layout=92.86,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#D4D4D4FF role=text size=24 text=long-names
                  81 StackPanel at=870,194,310,44 by=clip limit=88,186,1104,391 role=none
                    82 Button at=870,194,123,44 by=clip drawn heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader off paints=#707070FF,#D4D4D4FF role=button text="Reset All"
                      83 TextBlock at=887,200,89,32 box=89,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=88.17,31.2 limit=870,194,123,44 off owner=82 paints=#707070FF role=text size=26 text="RESET ALL"
                    84 Button at=1001,194,179,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#707070FF,#D4D4D4FF role=button text="Use This Shader"
                      85 TextBlock at=1018,200,145,32 box=145,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=144.69,31.2 limit=1001,194,179,44 owner=84 paints=#D4D4D4FF role=text size=26 text="USE THIS SHADER"
                86 MonoText at=652,244,528,25 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard=long/long-names.slangp layout=206.23,24.52 limit=88,186,1104,391 name=SNES.ShaderPath paints=#D4D4D4FF role=text size=18 text=long/long-names.slangp
                87 HintText at=652,275,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="2 parameters." layout=90.7,24 limit=88,186,1104,391 paints=#8D8D91FF role=text size=20 text="2 parameters."
                88 FilterBar at=652,307,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#D4D4D4FF role=group
                  89 TextBox at=652,307,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit
                    90 ScrollViewer allow="a text box scrolls its own text" at=663,314,492,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                      91 TextBlock allow="a text box scrolls its own text" at=663,314,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=90 layout=161.3,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search parameters"
              92 SliderList at=652,361,528,216 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#D4D4D4FF role=list
                93 ScrollViewer at=652,361,528,216 fades=b name=PART_ScrollViewer overflow=143 paints=#1B1B1EFF,#333337FF,#4C9AE8FF,#707070FF,#D4D4D4FF,#EEEEF0FF part role=scrollviewer viewport=652,361,512,216
                  94 ContentPresenter at=652,361,498,115 by=scroll in=93 limit=652,361,512,216 role=none
                    95 SectionHeader at=652,371,498,101 box=498,101 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---" in=93 layout=481.74,100.8 limit=652,361,512,216 paints=#D4D4D4FF role=text size=28 text="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---"
                  96 ContentPresenter at=652,476,498,122 by=scroll in=93 limit=652,361,512,216 role=none seen=652,476,498,101
                    97 SliderRow at=652,476,498,118 by=scroll in=93 limit=652,361,512,216 name=SNES.Parameter.BEAM_WIDTH_AT_THE_BRIGHTEST_PART_OF_THE_PICTURE role=none seen=652,476,498,101
                      98 StackPanel at=652,480,498,110 by=scroll in=93 limit=652,361,512,216 role=none seen=652,480,498,97
                        99 DockPanel at=652,480,498,58 by=scroll in=93 limit=652,361,512,216 role=none
                          100 Button at=1088,494,62,30 by=scroll drawn focus heard="Reset Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=93 limit=652,361,512,216 op paints=#707070FF,#D4D4D4FF role=button text=Reset
                            101 TextBlock at=1099,497,40,24 box=40,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=93 layout=40,24 limit=1088,494,62,30 owner=100 paints=#D4D4D4FF role=text size=20 text=RESET
                          102 TextBlock at=1028,494,48,29 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.60 in=93 layout=34.8,28.8 limit=652,361,512,216 paints=#D4D4D4FF role=text size=24 text=0.60
                          103 TextBlock at=652,480,364,58 box=364,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=93 layout=353.23,57.6 limit=652,361,512,216 paints=#D4D4D4FF role=text size=24 text="Horizontal beam width at the brightest part of the picture, in pixels of the source image"
                        104 Slider at=652,540,498,50 by=scroll drawn focus heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=93 limit=652,361,512,216 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider seen=652,540,498,37 sideways
                          105 RepeatButton at=652,555,144,20 drawn in=93 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          106 RepeatButton at=815,555,335,20 drawn in=93 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  107 RepeatButton at=1164,361,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    108 ViewboxContainer at=1168,367,8,4.4 by=clip limit=1168,367,8,4.4 role=none
                  109 RepeatButton at=1164,488,16,74 drawn heard="Page down" name=PART_PageDownButton part role=button
                  110 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    111 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        112 ButtonBar at=1105,136,87,44 by=panel limit=64,124,1152,465 paints=#D4D4D4FF role=toolbar
          113 Button at=1105,136,87,44 by=clip drawn focus heard=CLOSE limit=1105,136,87,44 op paints=#707070FF,#D4D4D4FF role=button text=Close
            114 TextBlock at=1122,142,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1105,136,87,44 owner=113 paints=#D4D4D4FF role=text size=26 text=CLOSE
focus 60
edge 8 up 113
edge 8 down 28
edge 28 up 8
edge 28 down 36
edge 31 up 8
edge 31 down 36
edge 31 left 28
edge 31 right 84
edge 36 up "row:34/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV"
edge 36 down 45
edge 36 left same
edge 36 right 89
edge 45 up 36
edge 45 down 51
edge 45 left same
edge 45 right 89
edge 51 up 45
edge 51 down 60
edge 51 left same
edge 51 right 89
edge 60 up 51
edge 60 down 75
edge 60 left same
edge 60 right 100
edge 75 up 60
edge 75 down "item:92/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge 75 left same
edge 75 right 104
edge 84 up 8
edge 84 down 89
edge 84 left 31
edge 84 right same
edge 89 up 84
edge 89 down "item:92/1/0/0/0/0/Reset Glow strength"
edge 89 left 45
edge 89 right same
edge 100 up 89
edge 100 down 104
edge 100 left 60
edge 100 right same
edge 104 up 100
edge 104 down "item:92/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge 113 up same
edge 113 down 84
edge 113 left 8
edge 113 right same
edge "item:92/1/0/0/0/0/Reset Glow strength" up 89
edge "item:92/1/0/0/0/0/Reset Glow strength" down "item:92/1/0/0/1/Glow strength"
edge "item:92/1/0/0/0/0/Reset Glow strength" left 51
edge "item:92/1/0/0/0/0/Reset Glow strength" right same
edge "item:92/1/0/0/1/Glow strength" up "item:92/1/0/0/0/0/Reset Glow strength"
edge "item:92/1/0/0/1/Glow strength" down "item:92/2/0/0/0/0/Reset Scanline weight"
edge "item:92/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" up 104
edge "item:92/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" down "item:92/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge "item:92/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" left 60
edge "item:92/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" right same
edge "item:92/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" up "item:92/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge "item:92/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" down same
edge "row:34/4/Simple CRT" up row:34/3/Scanlines
edge "row:34/4/Simple CRT" down "row:34/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV"
edge "row:34/4/Simple CRT" left same
edge "row:34/4/Simple CRT" right 89
edge "row:34/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" up "row:34/4/Simple CRT"
edge "row:34/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" down 36
edge "row:34/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" left same
edge "row:34/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" right 89
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
