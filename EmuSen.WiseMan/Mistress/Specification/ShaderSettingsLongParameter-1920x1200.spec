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
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,187,1728,699 by=panel limit=96,187,1728,698 paints=#9C9CA0FF role=none seen=96,187,1728,698
    1 DockPanel at=132,205,1656,663 by=panel limit=96,187,1728,698 role=none
      2 Grid at=132,205,1656,663 by=panel limit=96,187,1728,698 role=none
        3 Tabs at=132,205,1656,663 by=panel limit=96,187,1728,698 name=ConsoleTabs paints=#9C9CA0FF role=tab
          4 TabItem at=132,205,97.5,72 by=clip drawn focus heard=NES limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=NES
            5 TextBlock at=150,214,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=132,205,97.5,72 owner=4 paints=#86868AFF role=text size=30 text=NES
          6 TabItem at=229.5,205,76.5,72 by=clip drawn focus heard=GB limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=GB
            7 TextBlock at=247.5,214,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=229.5,205,76.5,72 owner=6 paints=#86868AFF role=text size=30 text=GB
          8 TabItem at=306,205,115.5,72 by=clip drawn focus heard=SNES limit=132,205,1656,663 op paints=#EEEEF0FF role=tabitem sideways text=SNES
            9 TextBlock at=324,214,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=306,205,115.5,72 owner=8 paints=#EEEEF0FF role=text size=30 text=SNES
          10 TabItem at=421.5,205,96,72 by=clip drawn focus heard=N64 limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=N64
            11 TextBlock at=439.5,214,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=421.5,205,96,72 owner=10 paints=#86868AFF role=text size=30 text=N64
          12 TabItem at=517.5,205,165,72 by=clip drawn focus heard=GENESIS limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=Genesis
            13 TextBlock at=535.5,214,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=517.5,205,165,72 owner=12 paints=#86868AFF role=text size=30 text=GENESIS
          14 TabItem at=682.5,205,234,72 by=clip drawn focus heard=PLAYSTATION limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=PlayStation
            15 TextBlock at=700.5,214,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=682.5,205,234,72 owner=14 paints=#86868AFF role=text size=30 text=PLAYSTATION
          16 ShaderPanel at=150,292,1620,576 by=clip limit=132,280,1656,588 role=none
            17 Grid at=150,292,792,576 by=clip limit=132,280,1656,588 role=none
              18 FilterBar at=150,292,792,69 by=clip limit=132,280,1656,588 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                19 TextBlock at=150,304,109.5,43.5 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=150,292,792,69 paints=#86868AFF part role=text size=24 text=Category
                20 Dropdown at=271.5,292,270,66 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  21 TextBlock at=289.5,299.5,204,48 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=289.5,299.5,204,48 owner=20 paints=#9C9CA0FF role=text size=26 text=All
                  22 ViewboxContainer at=508.5,320.1,18,9.81 by=clip limit=508.5,320.1,18,9.81 nohit role=none
                23 TextBox at=553.5,292,388.5,69 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  24 ScrollViewer allow="a text box scrolls its own text" at=570,302.5,355.5,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    25 TextBlock allow="a text box scrolls its own text" at=570,302.5,355.5,48 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=24 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              26 GroupedList at=150,373,792,414 by=clip limit=132,280,1656,588 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                27 ScrollViewer at=150,373,792,414 fades=t name=PART_ScrollViewer overflow=561 paints=#1E1E1EFF,#55555AFF,#FFFFFFFF part role=scrollviewer viewport=150,373,768,414
                  28 ListBoxItem at=150,334,768,129 by=scroll drawn focus heard="crt: crt-lottes" in=27 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem seen=150,373,768,90
                    29 GroupedListRow at=150,334,768,127.5 by=clip in=27 limit=150,334,768,129 role=none seen=150,373,768,88.5
                      30 DockPanel at=165,355,738,36 by=clip in=27 limit=150,334,768,129 role=none seen=165,373,738,18
                        31 TextBlock at=889.5,355,13.5,36 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=2 in=27 layout=8.04,24 limit=150,334,768,129 owner=28 paints=#86868AFF role=text seen=889.5,373,13.5,18 size=20 text=2
                        32 TextBlock at=165,355,712.5,36 box=475,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=CRT in=27 layout=26.24,24 limit=150,334,768,129 owner=28 paints=#86868AFF role=text seen=165,373,712.5,18 size=20 text=CRT
                      33 Border at=156,398.5,756,61.5 by=clip in=27 limit=150,334,768,129 role=none
                        34 DockPanel at=171,407.5,726,43.5 by=clip in=27 limit=150,334,768,129 role=none
                          35 StackPanel at=171,407.5,726,43.5 by=clip in=27 limit=150,334,768,129 role=none
                            36 TextBlock at=171,407.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-lottes in=27 layout=76.39,28.8 limit=150,334,768,129 owner=28 paints=#9C9CA0FF role=text size=24 text=crt-lottes
                  37 ListBoxItem at=150,463,768,66 by=scroll drawn focus heard=crt-royale-kurozumi in=27 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    38 GroupedListRow at=150,463,768,64.5 by=clip in=27 limit=150,463,768,66 role=none
                      39 Border at=156,464.5,756,61.5 by=clip in=27 limit=150,463,768,66 role=none
                        40 DockPanel at=171,473.5,726,43.5 by=clip in=27 limit=150,463,768,66 role=none
                          41 StackPanel at=171,473.5,726,43.5 by=clip in=27 limit=150,463,768,66 role=none
                            42 TextBlock at=171,473.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-royale-kurozumi in=27 layout=163.49,28.8 limit=150,463,768,66 owner=37 paints=#9C9CA0FF role=text size=24 text=crt-royale-kurozumi
                  43 ListBoxItem at=150,529,768,129 by=scroll drawn focus heard="handheld: lcd-grid-v2" in=27 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    44 GroupedListRow at=150,529,768,127.5 by=clip in=27 limit=150,529,768,129 role=none
                      45 DockPanel at=165,550,738,36 by=clip in=27 limit=150,529,768,129 role=none
                        46 TextBlock at=894,550,9,36 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=27 layout=5.12,24 limit=150,529,768,129 owner=43 paints=#86868AFF role=text size=20 text=1
                        47 TextBlock at=165,550,717,36 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=HANDHELD in=27 layout=71.9,24 limit=150,529,768,129 owner=43 paints=#86868AFF role=text size=20 text=HANDHELD
                      48 Border at=156,593.5,756,61.5 by=clip in=27 limit=150,529,768,129 role=none
                        49 DockPanel at=171,602.5,726,43.5 by=clip in=27 limit=150,529,768,129 role=none
                          50 StackPanel at=171,602.5,726,43.5 by=clip in=27 limit=150,529,768,129 role=none
                            51 TextBlock at=171,602.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=lcd-grid-v2 in=27 layout=89.64,28.8 limit=150,529,768,129 owner=43 paints=#9C9CA0FF role=text size=24 text=lcd-grid-v2
                  52 ListBoxItem at=150,658,768,129 by=scroll drawn focus heard="long: long-names" in=27 limit=150,373,768,414 op paints=#333337FF,#9C9CA0FF role=listitem
                    53 GroupedListRow at=150,658,768,127.5 by=clip in=27 limit=150,658,768,129 role=none
                      54 DockPanel at=165,679,738,36 by=clip in=27 limit=150,658,768,129 role=none
                        55 TextBlock at=894,679,9,36 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=27 layout=5.12,24 limit=150,658,768,129 owner=52 paints=#86868AFF role=text size=20 text=1
                        56 TextBlock at=165,679,717,36 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=LONG in=27 layout=35.9,24 limit=150,658,768,129 owner=52 paints=#86868AFF role=text size=20 text=LONG
                      57 Border at=156,722.5,756,61.5 by=clip in=27 limit=150,658,768,129 paints=#4C9AE8FF role=none
                        58 DockPanel at=171,731.5,726,43.5 by=clip in=27 limit=150,658,768,129 role=none
                          59 StackPanel at=171,731.5,726,43.5 by=clip in=27 limit=150,658,768,129 role=none
                            60 TextBlock at=171,731.5,726,43.5 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names in=27 layout=96.24,28.8 limit=150,658,768,129 owner=52 paints=#FFFFFFFF role=text size=24 text=long-names
                  61 RepeatButton at=918,373,24,24 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    62 ViewboxContainer at=924,382,12,6.6 by=clip limit=924,382,12,6.6 role=none
                  63 RepeatButton at=918,397,24,211.5 drawn heard="Page up" name=PART_PageUpButton part role=button
                  64 RepeatButton at=918,763,24,24 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    65 ViewboxContainer at=924,772,12,6.6 by=clip limit=924,772,12,6.6 role=none
              66 Border at=150,802,792,66 by=clip limit=132,280,1656,588 role=none
                67 Button at=150,802,226.5,66 by=clip drawn focus heard="UPDATE PACK" limit=132,280,1656,588 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Update Pack"
                  68 TextBlock at=175.5,811,175.5,48 box=117,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="UPDATE PACK" layout=116.04,31.2 limit=150,802,226.5,66 owner=67 paints=#9C9CA0FF role=text size=26 text="UPDATE PACK"
            69 DockPanel at=978,292,792,576 by=clip limit=132,280,1656,588 role=none
              70 StackPanel at=978,292,792,238.5 by=clip limit=132,280,1656,588 role=none
                71 Grid at=978,292,792,66 by=clip limit=132,280,1656,588 role=none
                  72 TextBlock at=978,292,292.5,66 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names layout=96.24,28.8 limit=132,280,1656,588 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=long-names
                  73 StackPanel at=1288.5,292,481.5,66 by=clip limit=132,280,1656,588 role=none
                    74 Button at=1288.5,292,189,66 by=clip drawn heard="RESET ALL" limit=132,280,1656,588 name=SNES.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      75 TextBlock at=1314,301,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=1288.5,292,189,66 off owner=74 paints=#55555AFF role=text size=26 text="RESET ALL"
                    76 Button at=1489.5,292,280.5,66 by=clip drawn focus heard="USE THIS SHADER" limit=132,280,1656,588 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      77 TextBlock at=1515,301,229.5,48 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=1489.5,292,280.5,66 owner=76 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                78 MonoText at=978,367,792,37.5 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard=long/long-names.slangp layout=206.23,24.52 limit=132,280,1656,588 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text=long/long-names.slangp
                79 HintText at=978,413.5,792,36 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="2 parameters." layout=94.54,24 limit=132,280,1656,588 paints=#86868AFF role=text size=20 text="2 parameters."
                80 FilterBar at=978,461.5,771,69 by=clip limit=132,280,1656,588 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  81 TextBox at=978,461.5,771,69 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    82 ScrollViewer allow="a text box scrolls its own text" at=994.5,472,738,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      83 TextBlock allow="a text box scrolls its own text" at=994.5,472,738,48 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=82 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              84 SliderList at=978,542.5,792,325.5 by=clip limit=132,280,1656,588 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                85 ScrollViewer at=978,542.5,792,325.5 fades=b name=PART_ScrollViewer overflow=256.5 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=978,542.5,768,325.5
                  86 ContentPresenter at=978,542.5,747,172.5 by=scroll in=85 limit=978,542.5,768,325.5 role=none
                    87 SectionHeader at=978,557.5,747,151.5 box=498,101 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---" in=85 layout=484.9,100.8 limit=978,542.5,768,325.5 paints=#B4B4B8FF role=text size=28 text="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---"
                  88 ContentPresenter at=978,715,747,183 by=scroll in=85 limit=978,542.5,768,325.5 role=none seen=978,715,747,153
                    89 SliderRow at=978,715,747,177 by=scroll in=85 limit=978,542.5,768,325.5 name=SNES.Parameter.BEAM_WIDTH_AT_THE_BRIGHTEST_PART_OF_THE_PICTURE role=none seen=978,715,747,153
                      90 StackPanel at=978,721,747,165 by=scroll in=85 limit=978,542.5,768,325.5 role=none seen=978,721,747,147
                        91 DockPanel at=978,721,768,87 by=scroll in=85 limit=978,542.5,768,325.5 role=none
                          92 Button at=1648.5,742,97.5,45 by=scroll drawn focus heard="Reset Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=85 limit=978,542.5,768,325.5 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            93 TextBlock at=1665,746.5,64.5,36 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=85 layout=42.66,24 limit=1648.5,742,97.5,45 owner=92 paints=#9C9CA0FF role=text size=20 text=RESET
                          94 TextBlock at=1558.5,742,72,43.5 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.60 in=85 layout=35.71,28.8 limit=978,542.5,768,325.5 paints=#9C9CA0FF role=text size=24 text=0.60
                          95 TextBlock at=978,721,562.5,87 box=375,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=85 layout=373.3,57.6 limit=978,542.5,768,325.5 paints=#9C9CA0FF role=text size=24 text="Horizontal beam width at the brightest part of the picture, in pixels of the source image"
                        96 Slider at=978,811,747,75 by=scroll drawn focus heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=85 limit=978,542.5,768,325.5 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider seen=978,811,747,57 sideways
                          97 RepeatButton at=978,833.5,216,30 drawn in=85 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          98 RepeatButton at=1222.5,833.5,502.5,30 drawn in=85 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  99 RepeatButton at=1746,542.5,24,24 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    100 ViewboxContainer at=1752,551.5,12,6.6 by=clip limit=1752,551.5,12,6.6 role=none
                  101 RepeatButton at=1746,721,24,123 drawn heard="Page down" name=PART_PageDownButton part role=button
                  102 RepeatButton at=1746,844,24,24 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    103 ViewboxContainer at=1752,853,12,6.6 by=clip limit=1752,853,12,6.6 role=none
        104 ButtonBar at=1653,205,135,66 by=panel limit=96,187,1728,698 paints=#9C9CA0FF role=toolbar
          105 Button at=1653,205,135,66 by=clip drawn focus heard=CLOSE limit=1653,205,135,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            106 TextBlock at=1678.5,214,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1653,205,135,66 owner=105 paints=#9C9CA0FF role=text size=26 text=CLOSE
focus 52
edge 8 up same
edge 8 down 20
edge 20 up 8
edge 20 down 28
edge 23 up 8
edge 23 down 28
edge 23 left 20
edge 23 right 76
edge 28 up row:26/5
edge 28 down 37
edge 28 left same
edge 28 right 81
edge 37 up 28
edge 37 down 43
edge 37 left same
edge 37 right 81
edge 43 up 37
edge 43 down 52
edge 43 left same
edge 43 right 81
edge 52 up 43
edge 52 down 67
edge 52 left same
edge 52 right 92
edge 67 up 52
edge 67 down "item:84/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge 67 left same
edge 67 right 96
edge 76 up 105
edge 76 down 81
edge 76 left 23
edge 76 right same
edge 81 up 76
edge 81 down "item:84/1/0/0/0/0/Reset Glow strength"
edge 81 left 37
edge 81 right "item:84/1/0/0/0/0/Reset Glow strength"
edge 92 up 81
edge 92 down 96
edge 92 left 52
edge 92 right same
edge 96 up 92
edge 96 down "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge 105 up same
edge 105 down 76
edge 105 left 8
edge 105 right same
edge "item:84/1/0/0/0/0/Reset Glow strength" up 81
edge "item:84/1/0/0/0/0/Reset Glow strength" down "item:84/1/0/0/1/Glow strength"
edge "item:84/1/0/0/0/0/Reset Glow strength" left 43
edge "item:84/1/0/0/0/0/Reset Glow strength" right same
edge "item:84/1/0/0/1/Glow strength" up "item:84/1/0/0/0/0/Reset Glow strength"
edge "item:84/1/0/0/1/Glow strength" down "item:84/2/0/0/0/0/Reset Scanline weight"
edge "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" up 96
edge "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" down "item:84/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" left 52
edge "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" right same
edge "item:84/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" up "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge "item:84/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" down same
edge row:26/4 up row:26/3
edge row:26/4 down row:26/5
edge row:26/4 left same
edge row:26/4 right 81
edge row:26/5 up row:26/4
edge row:26/5 down 28
edge row:26/5 left same
edge row:26/5 right 81
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
