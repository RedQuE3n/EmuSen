# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsLongParameter-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 96,37,1728,1002
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. RetroArch: 5 presets, pack built 2026-09-22 02:00 UTC. The presets are in <run>/Home/Shaders/RetroArch."
case ShaderSettingsLongParameter
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
                35 ScrollViewer at=150,373,792,414 fades=t name=PART_ScrollViewer overflow=561 paints=#1E1E1EFF,#55555AFF,#FFFFFFFF part role=scrollviewer viewport=150,373,768,414
                  36 ListBoxItem at=150,334,768,129 by=scroll drawn focus heard="crt: crt-lottes" in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem seen=150,373,768,90
                    37 GroupedListRow at=150,334,768,127.5 by=clip in=35 limit=150,334,768,129 role=none seen=150,373,768,88.5
                      38 DockPanel at=165,355,738,36 by=clip in=35 limit=150,334,768,129 role=none seen=165,373,738,18
                        39 TextBlock at=889.5,355,13.5,36 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=2 in=35 layout=8.04,24 limit=150,334,768,129 owner=36 paints=#86868AFF role=text seen=889.5,373,13.5,18 size=20 text=2
                        40 TextBlock at=165,355,712.5,36 box=475,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=CRT in=35 layout=26.24,24 limit=150,334,768,129 owner=36 paints=#86868AFF role=text seen=165,373,712.5,18 size=20 text=CRT
                      41 Border at=156,398.5,756,61.5 by=clip in=35 limit=150,334,768,129 role=none
                        42 DockPanel at=171,407.5,726,43.5 by=clip in=35 limit=150,334,768,129 role=none
                          43 StackPanel at=171,407.5,726,43.5 by=clip in=35 limit=150,334,768,129 role=none
                            44 TextBlock at=171,407.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-lottes in=35 layout=76.39,28.8 limit=150,334,768,129 owner=36 paints=#9C9CA0FF role=text size=24 text=crt-lottes
                  45 ListBoxItem at=150,463,768,66 by=scroll drawn focus heard=crt-royale-kurozumi in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    46 GroupedListRow at=150,463,768,64.5 by=clip in=35 limit=150,463,768,66 role=none
                      47 Border at=156,464.5,756,61.5 by=clip in=35 limit=150,463,768,66 role=none
                        48 DockPanel at=171,473.5,726,43.5 by=clip in=35 limit=150,463,768,66 role=none
                          49 StackPanel at=171,473.5,726,43.5 by=clip in=35 limit=150,463,768,66 role=none
                            50 TextBlock at=171,473.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-royale-kurozumi in=35 layout=163.49,28.8 limit=150,463,768,66 owner=45 paints=#9C9CA0FF role=text size=24 text=crt-royale-kurozumi
                  51 ListBoxItem at=150,529,768,129 by=scroll drawn focus heard="handheld: lcd-grid-v2" in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    52 GroupedListRow at=150,529,768,127.5 by=clip in=35 limit=150,529,768,129 role=none
                      53 DockPanel at=165,550,738,36 by=clip in=35 limit=150,529,768,129 role=none
                        54 TextBlock at=894,550,9,36 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=35 layout=5.12,24 limit=150,529,768,129 owner=51 paints=#86868AFF role=text size=20 text=1
                        55 TextBlock at=165,550,717,36 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=HANDHELD in=35 layout=71.9,24 limit=150,529,768,129 owner=51 paints=#86868AFF role=text size=20 text=HANDHELD
                      56 Border at=156,593.5,756,61.5 by=clip in=35 limit=150,529,768,129 role=none
                        57 DockPanel at=171,602.5,726,43.5 by=clip in=35 limit=150,529,768,129 role=none
                          58 StackPanel at=171,602.5,726,43.5 by=clip in=35 limit=150,529,768,129 role=none
                            59 TextBlock at=171,602.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=lcd-grid-v2 in=35 layout=89.64,28.8 limit=150,529,768,129 owner=51 paints=#9C9CA0FF role=text size=24 text=lcd-grid-v2
                  60 ListBoxItem at=150,658,768,129 by=scroll drawn focus heard="long: long-names" in=35 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    61 GroupedListRow at=150,658,768,127.5 by=clip in=35 limit=150,658,768,129 role=none
                      62 DockPanel at=165,679,738,36 by=clip in=35 limit=150,658,768,129 role=none
                        63 TextBlock at=894,679,9,36 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=35 layout=5.12,24 limit=150,658,768,129 owner=60 paints=#86868AFF role=text size=20 text=1
                        64 TextBlock at=165,679,717,36 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=LONG in=35 layout=35.9,24 limit=150,658,768,129 owner=60 paints=#86868AFF role=text size=20 text=LONG
                      65 Border at=156,722.5,756,61.5 by=clip in=35 limit=150,658,768,129 paints=#4C9AE8FF role=none
                        66 DockPanel at=171,731.5,726,43.5 by=clip in=35 limit=150,658,768,129 role=none
                          67 StackPanel at=171,731.5,726,43.5 by=clip in=35 limit=150,658,768,129 role=none
                            68 TextBlock at=171,731.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names in=35 layout=96.24,28.8 limit=150,658,768,129 owner=60 paints=#FFFFFFFF role=text size=24 text=long-names
                  69 RepeatButton at=918,373,24,24 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    70 ViewboxContainer at=924,382,12,6.6 by=clip limit=924,382,12,6.6 role=none
                  71 RepeatButton at=918,397,24,211.5 drawn heard="Page up" name=PART_PageUpButton part role=button
                  72 RepeatButton at=918,763,24,24 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    73 ViewboxContainer at=924,772,12,6.6 by=clip limit=924,772,12,6.6 role=none
              74 Border at=150,802,792,66 by=clip limit=132,280,1656,588 role=none
                75 Button at=150,802,226.5,66 by=clip drawn focus heard="UPDATE PACK" limit=132,280,1656,588 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Update Pack"
                  76 TextBlock at=175.5,811,175.5,48 box=117,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="UPDATE PACK" layout=116.04,31.2 limit=150,802,226.5,66 owner=75 paints=#9C9CA0FF role=text size=26 text="UPDATE PACK"
            77 DockPanel at=978,292,792,576 by=clip limit=132,280,1656,588 role=none
              78 StackPanel at=978,292,792,238.5 by=clip limit=132,280,1656,588 role=none
                79 Grid at=978,292,792,66 by=clip limit=132,280,1656,588 role=none
                  80 TextBlock at=978,292,292.5,66 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names layout=96.24,28.8 limit=132,280,1656,588 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=long-names
                  81 StackPanel at=1288.5,292,481.5,66 by=clip limit=132,280,1656,588 role=none
                    82 Button at=1288.5,292,189,66 by=clip drawn heard="RESET ALL" limit=132,280,1656,588 name=SNES.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      83 TextBlock at=1314,301,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=1288.5,292,189,66 off owner=82 paints=#55555AFF role=text size=26 text="RESET ALL"
                    84 Button at=1489.5,292,280.5,66 by=clip drawn focus heard="USE THIS SHADER" limit=132,280,1656,588 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      85 TextBlock at=1515,301,229.5,48 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=1489.5,292,280.5,66 owner=84 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                86 MonoText at=978,367,792,37.5 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard=long/long-names.slangp layout=206.23,24.52 limit=132,280,1656,588 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text=long/long-names.slangp
                87 HintText at=978,413.5,792,36 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="2 parameters." layout=94.54,24 limit=132,280,1656,588 paints=#86868AFF role=text size=20 text="2 parameters."
                88 FilterBar at=978,461.5,771,69 by=clip limit=132,280,1656,588 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  89 TextBox at=978,461.5,771,69 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    90 ScrollViewer allow="a text box scrolls its own text" at=994.5,472,738,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      91 TextBlock allow="a text box scrolls its own text" at=994.5,472,738,48 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=90 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              92 SliderList at=978,542.5,792,325.5 by=clip limit=132,280,1656,588 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                93 ScrollViewer at=978,542.5,792,325.5 fades=b name=PART_ScrollViewer overflow=256.5 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=978,542.5,768,325.5
                  94 ContentPresenter at=978,542.5,747,172.5 by=scroll in=93 limit=978,542.5,768,325.5 role=none
                    95 SectionHeader at=978,557.5,747,151.5 box=498,101 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---" in=93 layout=484.9,100.8 limit=978,542.5,768,325.5 paints=#B4B4B8FF role=text size=28 text="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---"
                  96 ContentPresenter at=978,715,747,183 by=scroll in=93 limit=978,542.5,768,325.5 role=none seen=978,715,747,153
                    97 SliderRow at=978,715,747,177 by=scroll in=93 limit=978,542.5,768,325.5 name=SNES.Parameter.BEAM_WIDTH_AT_THE_BRIGHTEST_PART_OF_THE_PICTURE role=none seen=978,715,747,153
                      98 StackPanel at=978,721,747,165 by=scroll in=93 limit=978,542.5,768,325.5 role=none seen=978,721,747,147
                        99 DockPanel at=978,721,768,87 by=scroll in=93 limit=978,542.5,768,325.5 role=none
                          100 Button at=1648.5,742,97.5,45 by=scroll drawn focus heard="Reset Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=93 limit=978,542.5,768,325.5 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            101 TextBlock at=1665,746.5,64.5,36 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=93 layout=42.66,24 limit=1648.5,742,97.5,45 owner=100 paints=#9C9CA0FF role=text size=20 text=RESET
                          102 TextBlock at=1558.5,742,72,43.5 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.60 in=93 layout=35.71,28.8 limit=978,542.5,768,325.5 paints=#9C9CA0FF role=text size=24 text=0.60
                          103 TextBlock at=978,721,562.5,87 box=375,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=93 layout=373.3,57.6 limit=978,542.5,768,325.5 paints=#9C9CA0FF role=text size=24 text="Horizontal beam width at the brightest part of the picture, in pixels of the source image"
                        104 Slider at=978,811,747,75 by=scroll drawn focus heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=93 limit=978,542.5,768,325.5 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider seen=978,811,747,57 sideways
                          105 RepeatButton at=978,833.5,216,30 drawn in=93 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          106 RepeatButton at=1222.5,833.5,502.5,30 drawn in=93 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  107 RepeatButton at=1746,542.5,24,24 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    108 ViewboxContainer at=1752,551.5,12,6.6 by=clip limit=1752,551.5,12,6.6 role=none
                  109 RepeatButton at=1746,721,24,123 drawn heard="Page down" name=PART_PageDownButton part role=button
                  110 RepeatButton at=1746,844,24,24 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    111 ViewboxContainer at=1752,853,12,6.6 by=clip limit=1752,853,12,6.6 role=none
        112 ButtonBar at=1653,205,135,66 by=panel limit=96,187,1728,698 paints=#9C9CA0FF role=toolbar
          113 Button at=1653,205,135,66 by=clip drawn focus heard=CLOSE limit=1653,205,135,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            114 TextBlock at=1678.5,214,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1653,205,135,66 owner=113 paints=#9C9CA0FF role=text size=26 text=CLOSE
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
edge 89 right "item:92/1/0/0/0/0/Reset Glow strength"
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
