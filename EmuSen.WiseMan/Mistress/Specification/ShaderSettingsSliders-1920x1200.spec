# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsSliders-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 96,37,1728,1002
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. RetroArch: 5 presets, pack built 2026-09-22 02:00 UTC. The presets are in <run>/Home/Shaders/RetroArch."
case ShaderSettingsSliders
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,187,1728,699 by=panel limit=96,187,1728,698 paints=#9C9CA0FF role=none seen=96,187,1728,698
    1 DockPanel at=132,205,1656,663 by=panel limit=96,187,1728,698 role=none
      2 Grid at=132,205,1656,663 by=panel limit=96,187,1728,698 role=none
        3 Tabs at=132,205,1656,663 by=panel limit=96,187,1728,698 name=ConsoleTabs paints=#9C9CA0FF role=tab
          4 TabItem at=132,205,90,72 by=clip drawn focus heard=NES limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=NES
            5 TextBlock at=150,217,54,48 box=36,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=NES layout=35.46,31.8 limit=132,205,90,72 owner=4 paints=#86868AFF role=text size=26.5 text=NES
          6 TabItem at=222,205,72,72 by=clip drawn focus heard=GB limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=GB
            7 TextBlock at=240,217,36,48 box=24,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=GB layout=23.85,31.8 limit=222,205,72,72 owner=6 paints=#86868AFF role=text size=26.5 text=GB
          8 TabItem at=294,205,106.5,72 by=clip drawn focus heard=SNES limit=132,205,1656,663 op paints=#EEEEF0FF role=tabitem sideways text=SNES
            9 TextBlock at=312,217,70.5,48 box=47,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=SNES layout=46.3,31.8 limit=294,205,106.5,72 owner=8 paints=#EEEEF0FF role=text size=26.5 text=SNES
          10 TabItem at=400.5,205,88.5,72 by=clip drawn focus heard=N64 limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=N64
            11 TextBlock at=418.5,217,52.5,48 box=35,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=N64 layout=34.93,31.8 limit=400.5,205,88.5,72 owner=10 paints=#86868AFF role=text size=26.5 text=N64
          12 TabItem at=489,205,147,72 by=clip drawn focus heard=SG-1000 limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=SG-1000
            13 TextBlock at=507,217,111,48 box=74,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=SG-1000 layout=73.03,31.8 limit=489,205,147,72 owner=12 paints=#86868AFF role=text size=26.5 text=SG-1000
          14 TabItem at=636,205,250.5,72 by=clip drawn focus heard="MASTER SYSTEM" limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text="Master System"
            15 TextBlock at=654,217,214.5,48 box=143,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=142.91,31.8 limit=636,205,250.5,72 owner=14 paints=#86868AFF role=text size=26.5 text="MASTER SYSTEM"
          16 TabItem at=886.5,205,150,72 by=clip drawn focus heard=GENESIS limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=Genesis
            17 TextBlock at=904.5,217,114,48 box=76,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=GENESIS layout=75.5,31.8 limit=886.5,205,150,72 owner=16 paints=#86868AFF role=text size=26.5 text=GENESIS
          18 TabItem at=1036.5,205,148.5,72 by=clip drawn focus heard="SEGA CD" limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text="Sega CD"
            19 TextBlock at=1054.5,217,112.5,48 box=75,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard="SEGA CD" layout=74.36,31.8 limit=1036.5,205,148.5,72 owner=18 paints=#86868AFF role=text size=26.5 text="SEGA CD"
          20 TabItem at=1185,205,211.5,72 by=clip drawn focus heard=PLAYSTATION limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=PlayStation
            21 TextBlock at=1203,217,175.5,48 box=117,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=PLAYSTATION layout=116.52,31.8 limit=1185,205,211.5,72 owner=20 paints=#86868AFF role=text size=26.5 text=PLAYSTATION
          22 TabItem at=1396.5,205,235.5,72 by=clip drawn focus heard="PLAYSTATION 2" limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text="PlayStation 2"
            23 TextBlock at=1414.5,217,199.5,48 box=133,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=132.47,31.8 limit=1396.5,205,235.5,72 owner=22 paints=#86868AFF role=text size=26.5 text="PLAYSTATION 2"
          24 ShaderPanel at=150,292,1620,576 by=clip limit=132,280,1656,588 role=none
            25 Grid at=150,292,792,576 by=clip limit=132,280,1656,588 role=none
              26 FilterBar at=150,292,792,69 by=clip limit=132,280,1656,588 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                27 TextBlock at=150,304,109.5,43.5 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=150,292,792,69 paints=#86868AFF part role=text size=24 text=Category
                28 Dropdown at=271.5,292,270,66 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  29 TextBlock at=289.5,299.5,204,48 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=289.5,299.5,204,48 owner=28 paints=#9C9CA0FF role=text size=26 text=All
                  30 ViewboxContainer at=508.5,320.1,18,9.81 by=clip limit=508.5,320.1,18,9.81 nohit role=none
                31 TextBox at=553.5,292,388.5,69 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  32 ScrollViewer allow="a text box scrolls its own text" at=570,302.5,355.5,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    33 TextBlock allow="a text box scrolls its own text" at=570,302.5,355.5,48 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=32 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              34 GroupedList at=150,373,792,414 by=clip limit=132,280,1656,588 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                35 ScrollViewer at=150,373,792,414 fades=tb name=PART_ScrollViewer overflow=549 paints=#1E1E1EFF,#55555AFF,#FFFFFFFF part role=scrollviewer viewport=150,373,768,414
                  36 ListBoxItem at=150,310,768,66 by=scroll drawn focus heard="CRT (Lottes)" in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem seen=150,373,768,3
                    37 GroupedListRow at=150,310,768,64.5 by=clip in=35 limit=150,310,768,66 paints=#9C9CA0FF role=none seen=150,373,768,1.5
                  38 ListBoxItem at=150,376,768,66 by=scroll drawn focus heard=Scanlines in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    39 GroupedListRow at=150,376,768,64.5 by=clip in=35 limit=150,376,768,66 role=none
                      40 Border at=156,377.5,756,61.5 by=clip in=35 limit=150,376,768,66 role=none
                        41 DockPanel at=171,386.5,726,43.5 by=clip in=35 limit=150,376,768,66 role=none
                          42 StackPanel at=171,386.5,726,43.5 by=clip in=35 limit=150,376,768,66 role=none
                            43 TextBlock at=171,386.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=35 layout=77.93,28.8 limit=150,376,768,66 owner=38 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  44 ListBoxItem at=150,442,768,66 by=scroll drawn focus heard="Simple CRT" in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    45 GroupedListRow at=150,442,768,64.5 by=clip in=35 limit=150,442,768,66 role=none
                      46 Border at=156,443.5,756,61.5 by=clip in=35 limit=150,442,768,66 role=none
                        47 DockPanel at=171,452.5,726,43.5 by=clip in=35 limit=150,442,768,66 role=none
                          48 StackPanel at=171,452.5,726,43.5 by=clip in=35 limit=150,442,768,66 role=none
                            49 TextBlock at=171,452.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=35 layout=91.2,28.8 limit=150,442,768,66 owner=44 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
                  50 ListBoxItem at=150,508,768,129 by=scroll drawn focus heard="bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    51 GroupedListRow at=150,508,768,127.5 by=clip in=35 limit=150,508,768,129 role=none
                      52 DockPanel at=165,529,738,36 by=clip in=35 limit=150,508,768,129 role=none
                        53 TextBlock at=894,529,9,36 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=35 layout=5.12,24 limit=150,508,768,129 owner=50 paints=#86868AFF role=text size=20 text=1
                        54 TextBlock at=165,529,717,36 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BEZEL / MEGA BEZEL / PRESETS / BASE CRT PRESETS" in=35 layout=354.82,24 limit=150,508,768,129 owner=50 paints=#86868AFF role=text size=20 text="BEZEL / MEGA BEZEL / PRESETS / BASE CRT PRESETS"
                      55 Border at=156,572.5,756,61.5 by=clip in=35 limit=150,508,768,129 role=none
                        56 DockPanel at=171,581.5,726,43.5 by=clip in=35 limit=150,508,768,129 role=none
                          57 StackPanel at=171,581.5,726,43.5 by=clip in=35 limit=150,508,768,129 role=none
                            58 TextBlock at=171,581.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="MBZ 0 SMOOTH-ADV GDV" in=35 layout=194.47,28.8 limit=150,508,768,129 owner=50 paints=#9C9CA0FF role=text size=24 text="MBZ 0 SMOOTH-ADV GDV"
                  59 ListBoxItem at=150,637,768,129 by=scroll drawn focus heard="big: huge" in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    60 GroupedListRow at=150,637,768,127.5 by=clip in=35 limit=150,637,768,129 role=none
                      61 DockPanel at=165,658,738,36 by=clip in=35 limit=150,637,768,129 role=none
                        62 TextBlock at=894,658,9,36 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=35 layout=5.12,24 limit=150,637,768,129 owner=59 paints=#86868AFF role=text size=20 text=1
                        63 TextBlock at=165,658,717,36 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=BIG in=35 layout=22.3,24 limit=150,637,768,129 owner=59 paints=#86868AFF role=text size=20 text=BIG
                      64 Border at=156,701.5,756,61.5 by=clip in=35 limit=150,637,768,129 paints=#4C9AE8FF role=none
                        65 DockPanel at=171,710.5,726,43.5 by=clip in=35 limit=150,637,768,129 role=none
                          66 StackPanel at=171,710.5,726,43.5 by=clip in=35 limit=150,637,768,129 role=none
                            67 TextBlock at=171,710.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=huge in=35 layout=40.03,28.8 limit=150,637,768,129 owner=59 paints=#FFFFFFFF role=text size=24 text=huge
                  68 ListBoxItem at=150,766,768,129 by=scroll drawn focus heard="crt: crt-lottes" in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem seen=150,766,768,21
                    69 GroupedListRow at=150,766,768,127.5 by=clip in=35 limit=150,766,768,129 paints=#86868AFF,#9C9CA0FF role=none seen=150,766,768,21
                  70 RepeatButton at=918,373,24,24 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    71 ViewboxContainer at=924,382,12,6.6 by=clip limit=924,382,12,6.6 role=none
                  72 RepeatButton at=918,397,24,99 drawn heard="Page up" name=PART_PageUpButton part role=button
                  73 RepeatButton at=918,652,24,111 drawn heard="Page down" name=PART_PageDownButton part role=button
                  74 RepeatButton at=918,763,24,24 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    75 ViewboxContainer at=924,772,12,6.6 by=clip limit=924,772,12,6.6 role=none
              76 Border at=150,802,792,66 by=clip limit=132,280,1656,588 role=none
                77 Button at=150,802,226.5,66 by=clip drawn focus heard="UPDATE PACK" limit=132,280,1656,588 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Update Pack"
                  78 TextBlock at=175.5,811,175.5,48 box=117,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="UPDATE PACK" layout=116.04,31.2 limit=150,802,226.5,66 owner=77 paints=#9C9CA0FF role=text size=26 text="UPDATE PACK"
            79 DockPanel at=978,292,792,576 by=clip limit=132,280,1656,588 role=none
              80 StackPanel at=978,292,792,238.5 by=clip limit=132,280,1656,588 role=none
                81 Grid at=978,292,792,66 by=clip limit=132,280,1656,588 role=none
                  82 TextBlock at=978,292,292.5,66 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=huge layout=40.03,28.8 limit=132,280,1656,588 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=huge
                  83 StackPanel at=1288.5,292,481.5,66 by=clip limit=132,280,1656,588 role=none
                    84 Button at=1288.5,292,189,66 by=clip drawn heard="RESET ALL" limit=132,280,1656,588 name=SNES.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      85 TextBlock at=1314,301,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=1288.5,292,189,66 off owner=84 paints=#55555AFF role=text size=26 text="RESET ALL"
                    86 Button at=1489.5,292,280.5,66 by=clip drawn focus heard="USE THIS SHADER" limit=132,280,1656,588 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      87 TextBlock at=1515,301,229.5,48 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=1489.5,292,280.5,66 owner=86 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                88 MonoText at=978,367,792,37.5 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard=big/huge.slangp layout=138.4,24.52 limit=132,280,1656,588 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text=big/huge.slangp
                89 HintText at=978,413.5,792,36 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="944 parameters." layout=111.16,24 limit=132,280,1656,588 paints=#86868AFF role=text size=20 text="944 parameters."
                90 FilterBar at=978,461.5,771,69 by=clip limit=132,280,1656,588 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  91 TextBox at=978,461.5,771,69 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    92 ScrollViewer allow="a text box scrolls its own text" at=994.5,472,738,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      93 TextBlock allow="a text box scrolls its own text" at=994.5,472,738,48 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=92 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              94 SliderList at=978,542.5,792,325.5 by=clip limit=132,280,1656,588 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                95 ScrollViewer at=978,542.5,792,325.5 fades=b name=PART_ScrollViewer overflow=117609 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=978,542.5,768,325.5
                  96 ContentPresenter at=978,542.5,747,72 by=scroll in=95 limit=978,542.5,768,325.5 role=none
                    97 SectionHeader at=978,557.5,747,51 box=498,34 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="--- Section 0 ---" in=95 layout=160.3,33.6 limit=978,542.5,768,325.5 paints=#B4B4B8FF role=text size=28 text="--- Section 0 ---"
                  98 ContentPresenter at=978,614.5,747,141 by=scroll in=95 limit=978,542.5,768,325.5 role=none
                    99 SliderRow at=978,614.5,747,135 by=scroll in=95 limit=978,542.5,768,325.5 name=SNES.Parameter.P0001 role=none
                      100 StackPanel at=978,620.5,747,123 by=scroll in=95 limit=978,542.5,768,325.5 role=none
                        101 DockPanel at=978,620.5,747,45 by=scroll in=95 limit=978,542.5,768,325.5 role=none
                          102 Button at=1627.5,620.5,97.5,45 by=scroll drawn focus heard="Reset Parameter 0001" in=95 limit=978,542.5,768,325.5 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            103 TextBlock at=1644,625,64.5,36 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=95 layout=42.66,24 limit=1627.5,620.5,97.5,45 owner=102 paints=#9C9CA0FF role=text size=20 text=RESET
                          104 TextBlock at=1537.5,620.5,72,43.5 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.50 in=95 layout=35.69,28.8 limit=978,542.5,768,325.5 paints=#9C9CA0FF role=text size=24 text=0.50
                          105 TextBlock at=978,620.5,541.5,43.5 box=361,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Parameter 0001" in=95 layout=128.71,28.8 limit=978,542.5,768,325.5 paints=#9C9CA0FF role=text size=24 text="Parameter 0001"
                        106 Slider at=978,668.5,747,75 by=scroll drawn focus heard="Parameter 0001" in=95 limit=978,542.5,768,325.5 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          107 RepeatButton at=978,691,358.5,30 drawn in=95 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          108 RepeatButton at=1366.5,691,358.5,30 drawn in=95 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  109 ContentPresenter at=978,755.5,747,141 by=scroll in=95 limit=978,542.5,768,325.5 role=none seen=978,755.5,747,112.5
                    110 SliderRow at=978,755.5,747,135 by=scroll in=95 limit=978,542.5,768,325.5 name=SNES.Parameter.P0002 role=none seen=978,755.5,747,112.5
                      111 StackPanel at=978,761.5,747,123 by=scroll in=95 limit=978,542.5,768,325.5 role=none seen=978,761.5,747,106.5
                        112 DockPanel at=978,761.5,747,45 by=scroll in=95 limit=978,542.5,768,325.5 role=none
                          113 Button at=1627.5,761.5,97.5,45 by=scroll drawn focus heard="Reset Parameter 0002" in=95 limit=978,542.5,768,325.5 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            114 TextBlock at=1644,766,64.5,36 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=95 layout=42.66,24 limit=1627.5,761.5,97.5,45 owner=113 paints=#9C9CA0FF role=text size=20 text=RESET
                          115 TextBlock at=1537.5,761.5,72,43.5 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.50 in=95 layout=35.69,28.8 limit=978,542.5,768,325.5 paints=#9C9CA0FF role=text size=24 text=0.50
                          116 TextBlock at=978,761.5,541.5,43.5 box=361,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Parameter 0002" in=95 layout=132.22,28.8 limit=978,542.5,768,325.5 paints=#9C9CA0FF role=text size=24 text="Parameter 0002"
                        117 Slider at=978,809.5,747,75 by=scroll drawn focus heard="Parameter 0002" in=95 limit=978,542.5,768,325.5 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider seen=978,809.5,747,58.5 sideways
                          118 RepeatButton at=978,832,358.5,30 drawn in=95 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          119 RepeatButton at=1366.5,832,358.5,30 drawn in=95 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  120 RepeatButton at=1746,542.5,24,24 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    121 ViewboxContainer at=1752,551.5,12,6.6 by=clip limit=1752,551.5,12,6.6 role=none
                  122 RepeatButton at=1746,626.5,24,217.5 drawn heard="Page down" name=PART_PageDownButton part role=button
                  123 RepeatButton at=1746,844,24,24 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    124 ViewboxContainer at=1752,853,12,6.6 by=clip limit=1752,853,12,6.6 role=none
        125 ButtonBar at=1653,205,135,66 by=panel limit=96,187,1728,698 paints=#9C9CA0FF role=toolbar
          126 Button at=1653,205,135,66 by=clip drawn focus heard=CLOSE limit=1653,205,135,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            127 TextBlock at=1678.5,214,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1653,205,135,66 owner=126 paints=#9C9CA0FF role=text size=26 text=CLOSE
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
edge 91 right "item:94/1/0/0/0/0/Reset Glow strength"
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
