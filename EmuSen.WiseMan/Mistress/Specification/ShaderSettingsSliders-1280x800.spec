# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsSliders-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,24,1152,668
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. RetroArch: 5 presets, pack built 2026-09-22 02:00 UTC. The presets are in <run>/Home/Shaders/RetroArch."
case ShaderSettingsSliders
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
                35 ScrollViewer at=100,248,528,275 fades=tb name=PART_ScrollViewer overflow=367 paints=#1B1B1EFF,#707070FF,#FFFFFFFF part role=scrollviewer viewport=100,248,512,275
                  36 ListBoxItem at=100,206,512,44 by=scroll drawn focus heard="CRT (Lottes)" in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem seen=100,248,512,2
                    37 GroupedListRow at=100,206,512,43 by=clip in=35 limit=100,206,512,44 paints=#D4D4D4FF role=none seen=100,248,512,1
                  38 ListBoxItem at=100,250,512,44 by=scroll drawn focus heard=Scanlines in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem
                    39 GroupedListRow at=100,250,512,43 by=clip in=35 limit=100,250,512,44 role=none
                      40 Border at=104,251,504,41 by=clip in=35 limit=100,250,512,44 role=none
                        41 DockPanel at=114,257,484,29 by=clip in=35 limit=100,250,512,44 role=none
                          42 StackPanel at=114,257,484,29 by=clip in=35 limit=100,250,512,44 role=none
                            43 TextBlock at=114,257,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=35 layout=73.66,28.8 limit=100,250,512,44 owner=38 paints=#D4D4D4FF role=text size=24 text=Scanlines
                  44 ListBoxItem at=100,294,512,44 by=scroll drawn focus heard="Simple CRT" in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem
                    45 GroupedListRow at=100,294,512,43 by=clip in=35 limit=100,294,512,44 role=none
                      46 Border at=104,295,504,41 by=clip in=35 limit=100,294,512,44 role=none
                        47 DockPanel at=114,301,484,29 by=clip in=35 limit=100,294,512,44 role=none
                          48 StackPanel at=114,301,484,29 by=clip in=35 limit=100,294,512,44 role=none
                            49 TextBlock at=114,301,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=35 layout=86.16,28.8 limit=100,294,512,44 owner=44 paints=#D4D4D4FF role=text size=24 text="Simple CRT"
                  50 ListBoxItem at=100,338,512,86 by=scroll drawn focus heard="bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem
                    51 GroupedListRow at=100,338,512,85 by=clip in=35 limit=100,338,512,86 role=none
                      52 DockPanel at=110,352,492,24 by=clip in=35 limit=100,338,512,86 role=none
                        53 TextBlock at=596,352,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=35 layout=5.24,24 limit=100,338,512,86 owner=50 paints=#8D8D91FF role=text size=20 text=1
                        54 TextBlock at=110,352,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BEZEL / MEGA BEZEL / PRESETS / BASE CRT PRESETS" in=35 layout=334.54,24 limit=100,338,512,86 owner=50 paints=#8D8D91FF role=text size=20 text="BEZEL / MEGA BEZEL / PRESETS / BASE CRT PRESETS"
                      55 Border at=104,381,504,41 by=clip in=35 limit=100,338,512,86 role=none
                        56 DockPanel at=114,387,484,29 by=clip in=35 limit=100,338,512,86 role=none
                          57 StackPanel at=114,387,484,29 by=clip in=35 limit=100,338,512,86 role=none
                            58 TextBlock at=114,387,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="MBZ 0 SMOOTH-ADV GDV" in=35 layout=185.26,28.8 limit=100,338,512,86 owner=50 paints=#D4D4D4FF role=text size=24 text="MBZ 0 SMOOTH-ADV GDV"
                  59 ListBoxItem at=100,424,512,86 by=scroll drawn focus heard="big: huge" in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem
                    60 GroupedListRow at=100,424,512,85 by=clip in=35 limit=100,424,512,86 role=none
                      61 DockPanel at=110,438,492,24 by=clip in=35 limit=100,424,512,86 role=none
                        62 TextBlock at=596,438,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=35 layout=5.24,24 limit=100,424,512,86 owner=59 paints=#8D8D91FF role=text size=20 text=1
                        63 TextBlock at=110,438,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=BIG in=35 layout=20.9,24 limit=100,424,512,86 owner=59 paints=#8D8D91FF role=text size=20 text=BIG
                      64 Border at=104,467,504,41 by=clip in=35 limit=100,424,512,86 paints=#4C9AE8FF role=none
                        65 DockPanel at=114,473,484,29 by=clip in=35 limit=100,424,512,86 role=none
                          66 StackPanel at=114,473,484,29 by=clip in=35 limit=100,424,512,86 role=none
                            67 TextBlock at=114,473,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=huge in=35 layout=38.59,28.8 limit=100,424,512,86 owner=59 paints=#0E0E10FF role=text size=24 text=huge
                  68 ListBoxItem at=100,510,512,86 by=scroll drawn focus heard="crt: crt-lottes" in=35 limit=100,248,512,275 op paints=#333337FF,#D4D4D4FF role=listitem seen=100,510,512,13
                    69 GroupedListRow at=100,510,512,85 by=clip in=35 limit=100,510,512,86 paints=#8D8D91FF,#D4D4D4FF role=none seen=100,510,512,13
                  70 RepeatButton at=612,248,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    71 ViewboxContainer at=616,254,8,4.4 by=clip limit=616,254,8,4.4 role=none
                  72 RepeatButton at=612,264,16,66 drawn heard="Page up" name=PART_PageUpButton part role=button
                  73 RepeatButton at=612,433,16,74 drawn heard="Page down" name=PART_PageDownButton part role=button
                  74 RepeatButton at=612,507,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    75 ViewboxContainer at=616,513,8,4.4 by=clip limit=616,513,8,4.4 role=none
              76 Border at=100,533,528,44 by=clip limit=88,186,1104,391 role=none
                77 Button at=100,533,146,44 by=clip drawn focus heard="UPDATE PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#707070FF,#D4D4D4FF role=button text="Update Pack"
                  78 TextBlock at=117,539,112,32 box=112,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="UPDATE PACK" layout=111.33,31.2 limit=100,533,146,44 owner=77 paints=#D4D4D4FF role=text size=26 text="UPDATE PACK"
            79 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              80 StackPanel at=652,194,528,159 by=clip limit=88,186,1104,391 role=none
                81 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  82 TextBlock at=652,194,206,44 box=206,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=huge layout=38.59,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#D4D4D4FF role=text size=24 text=huge
                  83 StackPanel at=870,194,310,44 by=clip limit=88,186,1104,391 role=none
                    84 Button at=870,194,123,44 by=clip drawn heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader off paints=#707070FF,#D4D4D4FF role=button text="Reset All"
                      85 TextBlock at=887,200,89,32 box=89,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=88.17,31.2 limit=870,194,123,44 off owner=84 paints=#707070FF role=text size=26 text="RESET ALL"
                    86 Button at=1001,194,179,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#707070FF,#D4D4D4FF role=button text="Use This Shader"
                      87 TextBlock at=1018,200,145,32 box=145,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=144.69,31.2 limit=1001,194,179,44 owner=86 paints=#D4D4D4FF role=text size=26 text="USE THIS SHADER"
                88 MonoText at=652,244,528,25 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard=big/huge.slangp layout=138.4,24.52 limit=88,186,1104,391 name=SNES.ShaderPath paints=#D4D4D4FF role=text size=18 text=big/huge.slangp
                89 HintText at=652,275,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="944 parameters." layout=106.18,24 limit=88,186,1104,391 paints=#8D8D91FF role=text size=20 text="944 parameters."
                90 FilterBar at=652,307,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#D4D4D4FF role=group
                  91 TextBox at=652,307,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit
                    92 ScrollViewer allow="a text box scrolls its own text" at=663,314,492,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                      93 TextBlock allow="a text box scrolls its own text" at=663,314,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=92 layout=161.3,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search parameters"
              94 SliderList at=652,361,528,216 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#D4D4D4FF role=list
                95 ScrollViewer at=652,361,528,216 fades=b name=PART_ScrollViewer overflow=78407 paints=#1B1B1EFF,#333337FF,#4C9AE8FF,#707070FF,#D4D4D4FF,#EEEEF0FF part role=scrollviewer viewport=652,361,512,216
                  96 ContentPresenter at=652,361,498,48 by=scroll in=95 limit=652,361,512,216 role=none
                    97 SectionHeader at=652,371,498,34 box=498,34 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="--- Section 0 ---" in=95 layout=154.56,33.6 limit=652,361,512,216 paints=#D4D4D4FF role=text size=28 text="--- Section 0 ---"
                  98 ContentPresenter at=652,409,498,94 by=scroll in=95 limit=652,361,512,216 role=none
                    99 SliderRow at=652,409,498,90 by=scroll in=95 limit=652,361,512,216 name=SNES.Parameter.P0001 role=none
                      100 StackPanel at=652,413,498,82 by=scroll in=95 limit=652,361,512,216 role=none
                        101 DockPanel at=652,413,498,30 by=scroll in=95 limit=652,361,512,216 role=none
                          102 Button at=1088,413,62,30 by=scroll drawn focus heard="Reset Parameter 0001" in=95 limit=652,361,512,216 op paints=#707070FF,#D4D4D4FF role=button text=Reset
                            103 TextBlock at=1099,416,40,24 box=40,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=95 layout=40,24 limit=1088,413,62,30 owner=102 paints=#D4D4D4FF role=text size=20 text=RESET
                          104 TextBlock at=1028,413,48,29 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.50 in=95 layout=34.8,28.8 limit=652,361,512,216 paints=#D4D4D4FF role=text size=24 text=0.50
                          105 TextBlock at=652,413,364,29 box=364,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Parameter 0001" in=95 layout=123.72,28.8 limit=652,361,512,216 paints=#D4D4D4FF role=text size=24 text="Parameter 0001"
                        106 Slider at=652,445,498,50 by=scroll drawn focus heard="Parameter 0001" in=95 limit=652,361,512,216 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          107 RepeatButton at=652,460,239,20 drawn in=95 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          108 RepeatButton at=911,460,239,20 drawn in=95 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  109 ContentPresenter at=652,503,498,94 by=scroll in=95 limit=652,361,512,216 role=none seen=652,503,498,74
                    110 SliderRow at=652,503,498,90 by=scroll in=95 limit=652,361,512,216 name=SNES.Parameter.P0002 role=none seen=652,503,498,74
                      111 StackPanel at=652,507,498,82 by=scroll in=95 limit=652,361,512,216 role=none seen=652,507,498,70
                        112 DockPanel at=652,507,498,30 by=scroll in=95 limit=652,361,512,216 role=none
                          113 Button at=1088,507,62,30 by=scroll drawn focus heard="Reset Parameter 0002" in=95 limit=652,361,512,216 op paints=#707070FF,#D4D4D4FF role=button text=Reset
                            114 TextBlock at=1099,510,40,24 box=40,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=95 layout=40,24 limit=1088,507,62,30 owner=113 paints=#D4D4D4FF role=text size=20 text=RESET
                          115 TextBlock at=1028,507,48,29 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.50 in=95 layout=34.8,28.8 limit=652,361,512,216 paints=#D4D4D4FF role=text size=24 text=0.50
                          116 TextBlock at=652,507,364,29 box=364,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Parameter 0002" in=95 layout=126.65,28.8 limit=652,361,512,216 paints=#D4D4D4FF role=text size=24 text="Parameter 0002"
                        117 Slider at=652,539,498,50 by=scroll drawn focus heard="Parameter 0002" in=95 limit=652,361,512,216 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider seen=652,539,498,38 sideways
                          118 RepeatButton at=652,554,239,20 drawn in=95 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          119 RepeatButton at=911,554,239,20 drawn in=95 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  120 RepeatButton at=1164,361,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    121 ViewboxContainer at=1168,367,8,4.4 by=clip limit=1168,367,8,4.4 role=none
                  122 RepeatButton at=1164,417,16,144 drawn heard="Page down" name=PART_PageDownButton part role=button
                  123 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    124 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        125 ButtonBar at=1105,136,87,44 by=panel limit=64,124,1152,465 paints=#D4D4D4FF role=toolbar
          126 Button at=1105,136,87,44 by=clip drawn focus heard=CLOSE limit=1105,136,87,44 op paints=#707070FF,#D4D4D4FF role=button text=Close
            127 TextBlock at=1122,142,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1105,136,87,44 owner=126 paints=#D4D4D4FF role=text size=26 text=CLOSE
focus 59
edge 8 up 126
edge 8 down 28
edge 28 up 8
edge 28 down 38
edge 31 up 8
edge 31 down 38
edge 31 left 28
edge 31 right 86
edge 36 up row:34/1/CRT
edge 36 down 38
edge 36 left same
edge 36 right 91
edge 38 up 36
edge 38 down 44
edge 38 left same
edge 38 right 91
edge 44 up 38
edge 44 down 50
edge 44 left same
edge 44 right 91
edge 50 up 44
edge 50 down 59
edge 50 left same
edge 50 right 91
edge 59 up 50
edge 59 down 68
edge 59 left same
edge 59 right 106
edge 68 up 59
edge 68 down row:34/8/crt-royale-kurozumi
edge 68 left same
edge 68 right 91
edge 77 up "row:34/9/handheld: lcd-grid-v2"
edge 77 down same
edge 77 left same
edge 77 right "item:94/2/0/0/1/Scanline weight"
edge 86 up 8
edge 86 down 91
edge 86 left 31
edge 86 right same
edge 91 up 86
edge 91 down "item:94/1/0/0/0/0/Reset Glow strength"
edge 91 left 50
edge 91 right same
edge 102 up 91
edge 102 down 106
edge 102 left 50
edge 102 right same
edge 106 up 102
edge 106 down 113
edge 113 up 106
edge 113 down 117
edge 113 left 68
edge 113 right same
edge 117 up 113
edge 117 down "item:94/3/0/0/0/0/Reset Parameter 0003"
edge 126 up same
edge 126 down 86
edge 126 left 8
edge 126 right same
edge "item:94/1/0/0/0/0/Reset Glow strength" up 91
edge "item:94/1/0/0/0/0/Reset Glow strength" down "item:94/1/0/0/1/Glow strength"
edge "item:94/1/0/0/0/0/Reset Glow strength" left 59
edge "item:94/1/0/0/0/0/Reset Glow strength" right same
edge "item:94/1/0/0/1/Glow strength" up "item:94/1/0/0/0/0/Reset Glow strength"
edge "item:94/1/0/0/1/Glow strength" down "item:94/2/0/0/0/0/Reset Scanline weight"
edge "item:94/3/0/0/0/0/Reset Parameter 0003" up 117
edge "item:94/3/0/0/0/0/Reset Parameter 0003" down "item:94/3/0/0/1/Parameter 0003"
edge "item:94/3/0/0/0/0/Reset Parameter 0003" left 68
edge "item:94/3/0/0/0/0/Reset Parameter 0003" right same
edge row:34/1/CRT up "row:34/0/Built into EmuSen: None"
edge row:34/1/CRT down 36
edge row:34/1/CRT left same
edge row:34/1/CRT right 91
edge row:34/8/crt-royale-kurozumi up 68
edge row:34/8/crt-royale-kurozumi down "row:34/9/handheld: lcd-grid-v2"
edge row:34/8/crt-royale-kurozumi left same
edge row:34/8/crt-royale-kurozumi right 91
edge "row:34/9/handheld: lcd-grid-v2" up row:34/8/crt-royale-kurozumi
edge "row:34/9/handheld: lcd-grid-v2" down 77
edge "row:34/9/handheld: lcd-grid-v2" left same
edge "row:34/9/handheld: lcd-grid-v2" right 91
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
