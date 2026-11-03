# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsDesktopDisplay-1920x1200
size 1920 1200
kind window
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 0
frame 0,0,960,640
case SettingsDesktopDisplay
palette #0000008C #000000AA #000000FF #005FB8FF #0A5A96FF #0E0E10FF #1B1B1EFF #1B6E3CFF #1B7A1BFF #1C1C1CE6 #1F1F1FFF #26262BFF #2A2A2AFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #5F5F5FFF #707070FF #7A5B00FF #8A6300FF #8C8C8CFF #8D8D91FF #9CDCFEFF #A34B1EFF #B3261EFF #B32D12FF #D08770FF #D37070FF #D4D4D4FF #D6D6D6FF #DAA520FF #DCDCDCFF #EEEEF0FF #F3F3F3FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
window 960 640
tree
  0 Panel at=0,0,960,640 by=panel limit=0,0,960,640 role=none
    1 DockPanel at=0,0,960,640 by=panel limit=0,0,960,640 paints=#F3F3F3FF role=none
      2 Border at=0,0,201,640 by=panel limit=0,0,960,640 paints=#D6D6D6FF role=none
        3 SourceList at=0,12,200,628 by=panel focus limit=0,0,960,640 name=SettingsSourceList paints=#000000FF,#F3F3F3FF role=list
          4 ScrollViewer at=0,12,200,628 name=PART_Scroll overflow=0 paints=#000000FF part role=scrollviewer viewport=0,12,200,628
            5 SourceListGroupView at=0,12,200,144 by=fixed-scroll in=4 limit=0,12,200,628 role=group
              6 SourceListRow at=6,12,188,24 by=fixed-scroll heard=Controllers in=4 limit=0,12,200,628 role=listitem
                7 Grid at=26,12,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  8 TextBlock at=26,16,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Controllers in=4 layout=59.77,15.6 limit=0,12,200,628 paints=#1F1F1FFF role=text size=13 text=Controllers
              9 SourceListRow at=6,36,188,24 by=fixed-scroll heard=Display in=4 limit=0,12,200,628 paints=#005FB8FF role=listitem
                10 Grid at=26,36,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  11 TextBlock at=26,40,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Display in=4 layout=39.94,15.6 limit=0,12,200,628 paints=#FFFFFFFF role=text size=13 text=Display
              12 SourceListRow at=6,60,188,24 by=fixed-scroll heard=Sound in=4 limit=0,12,200,628 role=listitem
                13 Grid at=26,60,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  14 TextBlock at=26,64,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Sound in=4 layout=34.92,15.6 limit=0,12,200,628 paints=#1F1F1FFF role=text size=13 text=Sound
              15 SourceListRow at=6,84,188,24 by=fixed-scroll heard=Consoles in=4 limit=0,12,200,628 role=listitem
                16 Grid at=26,84,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  17 TextBlock at=26,88,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Consoles in=4 layout=49.32,15.6 limit=0,12,200,628 paints=#1F1F1FFF role=text size=13 text=Consoles
              18 SourceListRow at=6,108,188,24 by=fixed-scroll heard=Library in=4 limit=0,12,200,628 role=listitem
                19 Grid at=26,108,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  20 TextBlock at=26,112,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Library in=4 layout=39.23,15.6 limit=0,12,200,628 paints=#1F1F1FFF role=text size=13 text=Library
              21 SourceListRow at=6,132,188,24 by=fixed-scroll heard=EmuSen in=4 limit=0,12,200,628 role=listitem
                22 Grid at=26,132,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  23 TextBlock at=26,136,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=EmuSen in=4 layout=45.84,15.6 limit=0,12,200,628 paints=#1F1F1FFF role=text size=13 text=EmuSen
      24 DockPanel at=225,8,711,608 by=panel limit=0,0,960,640 role=none
        25 DockPanel at=225,8,711,48 by=panel limit=0,0,960,640 name=SettingsHeader role=none
          26 TextBox at=716,16,220,32 by=panel drawn focus heard="Search settings" limit=0,0,960,640 name=SettingsSearch op paints=#1F1F1FFF,#8C8C8CFF,#FFFFFFFF role=edit
            27 ScrollViewer at=727,23,202,19 name=PART_ScrollViewer paints=#1F1F1FFF part role=scrollviewer
              28 TextBlock at=727,23,202,19 box=202,19 by=fixed-scroll caps=9.0994 drawn face=Barlow heard="Search settings" in=27 layout=85.19,15.6 limit=727,23,202,19 name=PART_Placeholder paints=#5F5F5FFF part role=text size=13 text="Search settings"
          29 TextBlock at=225,22,491,20 box=491,20 by=panel caps=11.1992 drawn face="Barlow Condensed" heard=DISPLAY layout=41.25,19.2 limit=0,0,960,640 name=SettingsTitle paints=#000000FF role=text size=16 text=DISPLAY
        30 Border at=660,64,276,238 by=panel limit=0,0,960,640 name=SettingsPane paints=#FFFFFFFF role=none
          31 StackPanel at=676,80,244,206 by=panel limit=0,0,960,640 role=none
            32 Panel at=676,80,244,162 by=panel limit=0,0,960,640 name=SettingsPicture role=none
              33 Border at=676,80,244,162 by=panel limit=0,0,960,640 paints=#F3F3F3FF role=none
              34 TextBlock at=716,151,165,20 box=165,20 by=panel caps=11.1992 drawn face="Barlow Condensed" heard="Aurora Drift (Synthetic) · Off CRT" layout=164.02,19.2 limit=0,0,960,640 name=SettingsCaption nohit paints=#000000FF role=text size=16 text="Aurora Drift (Synthetic) · Off CRT"
            35 TextBlock at=676,254,244,32 box=244,32 by=panel caps=9.0994 drawn face=Barlow heard="Off: the console's pixels as they are, with no television." layout=236.05,31.2 limit=0,0,960,640 name=SettingsLine paints=#1F1F1FFF role=text size=13 text="Off: the console's pixels as they are, with no television."
        36 ScrollViewer at=225,56,419,560 by=panel fades=b limit=0,0,960,640 overflow=83 paints=#000000FF,#8C8C8CFF role=scrollviewer viewport=225,56,419,560
          37 StackPanel at=225,56,419,643 by=scroll heard=Display in=36 limit=225,56,419,560 name=SettingsRows paints=#000000FF,#005FB8FF,#1F1F1FFF,#5F5F5FFF,#D6D6D6FF role=none seen=225,56,419,560
            38 Border at=225,56,419,65 by=scroll focus heard="Picture shape, TV shape" in=36 limit=225,56,419,560 name=SettingsRow.shape paints=#D6D6D6FF role=none
              39 DockPanel at=233,60,403,56 by=scroll in=36 limit=225,56,419,560 role=none
                40 ComboBox at=508,72,120,32 by=scroll drawn focus heard="Picture shape" in=36 limit=225,56,419,560 op paints=#1F1F1FFF,#5F5F5FFF,#8C8C8CFF,#FFFFFFFF role=combobox sideways
                  41 TextBlock at=520,79,76,16 box=76,16 by=clip caps=9.0994 drawn face=Barlow heard="TV shape" in=36 layout=49.8,15.6 limit=520,77,76,20 owner=40 paints=#1F1F1FFF role=text size=13 text="TV shape"
                  42 ViewboxContainer at=606,84.73,12,6.54 by=clip in=36 limit=606,84.73,12,6.54 nohit role=none
                43 StackPanel at=233,60,259,56 by=scroll in=36 limit=225,56,419,560 role=none
                  44 TextBlock at=233,60,259,16 box=259,16 by=scroll caps=9.0994 drawn face=Barlow heard="Picture shape" in=36 layout=75.76,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text="Picture shape"
                  45 TextBlock at=233,76,259,40 box=259,40 by=scroll caps=7.6995 drawn face=Barlow heard="The screen each console was made for, or each of the frame's pixels square. Console's leaves each console its own." in=36 layout=254.81,39.6 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="The screen each console was made for, or each of the frame's pixels square. Console's leaves each console its own."
            46 Border at=225,121,419,48 by=scroll focus heard="CRT shader, Off" in=36 limit=225,56,419,560 name=SettingsRow.crt paints=#D6D6D6FF,#FFFFFFFF role=none
              47 DockPanel at=233,125,403,39 by=scroll in=36 limit=225,56,419,560 role=none
                48 ComboBox at=508,129,120,32 by=scroll drawn focus heard="CRT shader" in=36 limit=225,56,419,560 op paints=#1F1F1FFF,#5F5F5FFF,#8C8C8CFF,#FFFFFFFF role=combobox sideways
                  49 TextBlock at=520,136,76,16 box=76,16 by=clip caps=9.0994 drawn face=Barlow heard=Off in=36 layout=16.16,15.6 limit=520,134,76,20 owner=48 paints=#1F1F1FFF role=text size=13 text=Off
                  50 ViewboxContainer at=606,141.73,12,6.54 by=clip in=36 limit=606,141.73,12,6.54 nohit role=none
                51 StackPanel at=233,129,259,30 by=scroll in=36 limit=225,56,419,560 role=none
                  52 TextBlock at=233,129,259,16 box=259,16 by=scroll caps=9.0994 drawn face=Barlow heard="CRT shader" in=36 layout=62.32,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text="CRT shader"
                  53 TextBlock at=233,145,259,14 box=259,14 by=scroll caps=7.6995 drawn face=Barlow heard="The television's picture for every console that had one." in=36 layout=250.1,13.2 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="The television's picture for every console that had one."
            54 Border at=225,169,419,52 by=scroll focus heard="Shader levers" in=36 limit=225,56,419,560 name=SettingsRow.levers paints=#D6D6D6FF role=none
              55 DockPanel at=233,173,403,43 by=scroll in=36 limit=225,56,419,560 role=none
                56 Button at=571,180,57,29 by=scroll drawn focus heard="Shader levers" in=36 limit=225,56,419,560 op paints=#1F1F1FFF,#8C8C8CFF,#FFFFFFFF role=button text="Open…"
                  57 AccessText at=580,186,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=36 layout=38.73,15.6 limit=571,180,57,29 owner=56 paints=#1F1F1FFF role=none size=13 text="Open…"
                58 StackPanel at=233,173,322,43 by=scroll in=36 limit=225,56,419,560 role=none
                  59 TextBlock at=233,173,322,16 box=322,16 by=scroll caps=9.0994 drawn face=Barlow heard="Shader levers" in=36 layout=74.2,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text="Shader levers"
                  60 TextBlock at=233,189,322,27 box=322,27 by=scroll caps=7.6995 drawn face=Barlow heard="The CRT's own levers for one console: the mask, the bloom, the scanlines and the glass." in=36 layout=287.39,26.4 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="The CRT's own levers for one console: the mask, the bloom, the scanlines and the glass."
            61 Border at=225,221,419,65 by=scroll focus heard=Shaders in=36 limit=225,56,419,560 name=SettingsRow.shaders paints=#D6D6D6FF role=none
              62 DockPanel at=233,225,403,56 by=scroll in=36 limit=225,56,419,560 role=none
                63 Button at=571,239,57,29 by=scroll drawn focus heard=Shaders in=36 limit=225,56,419,560 op paints=#1F1F1FFF,#8C8C8CFF,#FFFFFFFF role=button text="Open…"
                  64 AccessText at=580,245,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=36 layout=38.73,15.6 limit=571,239,57,29 owner=63 paints=#1F1F1FFF role=none size=13 text="Open…"
                65 StackPanel at=233,225,322,56 by=scroll in=36 limit=225,56,419,560 role=none
                  66 TextBlock at=233,225,322,16 box=322,16 by=scroll caps=9.0994 drawn face=Barlow heard=Shaders in=36 layout=44.97,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text=Shaders
                  67 TextBlock at=233,241,322,40 box=322,40 by=scroll caps=7.6995 drawn face=Barlow heard="Each console's screen filter from every one EmuSen has: the built-in filters and the slang catalogue's presets, their parameters, and the catalogue's download." in=36 layout=314.72,39.6 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="Each console's screen filter from every one EmuSen has: the built-in filters and the slang catalogue's presets, their parameters, and the catalogue's download."
            68 Border at=225,286,419,65 by=scroll focus heard="Crop overscan, Whole picture" in=36 limit=225,56,419,560 name=SettingsRow.crop paints=#D6D6D6FF role=none
              69 DockPanel at=233,290,403,56 by=scroll in=36 limit=225,56,419,560 role=none
                70 ComboBox at=507,302,121,32 by=scroll drawn focus heard="Crop overscan" in=36 limit=225,56,419,560 op paints=#1F1F1FFF,#5F5F5FFF,#8C8C8CFF,#FFFFFFFF role=combobox sideways
                  71 TextBlock at=519,309,77,16 box=77,16 by=clip caps=9.0994 drawn face=Barlow heard="Whole picture" in=36 layout=76.52,15.6 limit=519,307,77,20 owner=70 paints=#1F1F1FFF role=text size=13 text="Whole picture"
                  72 ViewboxContainer at=606,314.73,12,6.54 by=clip in=36 limit=606,314.73,12,6.54 nohit role=none
                73 StackPanel at=233,290,258,56 by=scroll in=36 limit=225,56,419,560 role=none
                  74 TextBlock at=233,290,258,16 box=258,16 by=scroll caps=9.0994 drawn face=Barlow heard="Crop overscan" in=36 layout=78.57,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text="Crop overscan"
                  75 TextBlock at=233,306,258,40 box=258,40 by=scroll caps=7.6995 drawn face=Barlow heard="What is hidden at the picture's edges on a console that had a television: the whole picture, or the part a television showed. Console's leaves each its own." in=36 layout=250.71,39.6 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="What is hidden at the picture's edges on a console that had a television: the whole picture, or the part a television showed. Console's leaves each its own."
            76 Border at=225,351,419,52 by=scroll focus heard="Light or dark, Dark" in=36 limit=225,56,419,560 name=SettingsRow.variant paints=#D6D6D6FF role=none
              77 DockPanel at=233,355,403,43 by=scroll in=36 limit=225,56,419,560 role=none
                78 ComboBox at=508,361,120,32 by=scroll drawn focus heard="Light or dark" in=36 limit=225,56,419,560 op paints=#1F1F1FFF,#5F5F5FFF,#8C8C8CFF,#FFFFFFFF role=combobox sideways
                  79 TextBlock at=520,368,76,16 box=76,16 by=clip caps=9.0994 drawn face=Barlow heard=Dark in=36 layout=25.64,15.6 limit=520,366,76,20 owner=78 paints=#1F1F1FFF role=text size=13 text=Dark
                  80 ViewboxContainer at=606,373.73,12,6.54 by=clip in=36 limit=606,373.73,12,6.54 nohit role=none
                81 StackPanel at=233,355,259,43 by=scroll in=36 limit=225,56,419,560 role=none
                  82 TextBlock at=233,355,259,16 box=259,16 by=scroll caps=9.0994 drawn face=Barlow heard="Light or dark" in=36 layout=69.97,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text="Light or dark"
                  83 TextBlock at=233,371,259,27 box=259,27 by=scroll caps=7.6995 drawn face=Barlow heard="Dark is the big screen's default; Light, or the desktop's own choice." in=36 layout=247.63,26.4 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="Dark is the big screen's default; Light, or the desktop's own choice."
            84 Border at=225,403,419,48 by=scroll focus heard="Background art on Home, On" in=36 limit=225,56,419,560 name=SettingsRow.art paints=#D6D6D6FF role=none
              85 DockPanel at=233,407,403,39 by=scroll in=36 limit=225,56,419,560 role=none
                86 LunaSwitch at=570,411,58,32 by=scroll drawn focus heard="Background art on Home" in=36 limit=225,56,419,560 op paints=#000000FF,#005FB8FF role=button
                87 StackPanel at=233,411,321,30 by=scroll in=36 limit=225,56,419,560 role=none
                  88 TextBlock at=233,411,321,16 box=321,16 by=scroll caps=9.0994 drawn face=Barlow heard="Background art on Home" in=36 layout=135.95,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text="Background art on Home"
                  89 TextBlock at=233,427,321,14 box=321,14 by=scroll caps=7.6995 drawn face=Barlow heard="The focused game's art behind Home, dimmed and blurred." in=36 layout=272.34,13.2 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="The focused game's art behind Home, dimmed and blurred."
            90 Border at=225,451,419,52 by=scroll focus heard="Play videos, On" in=36 limit=225,56,419,560 name=SettingsRow.videos paints=#D6D6D6FF role=none
              91 DockPanel at=233,455,403,43 by=scroll in=36 limit=225,56,419,560 role=none
                92 LunaSwitch at=570,461,58,32 by=scroll drawn focus heard="Play videos" in=36 limit=225,56,419,560 op paints=#000000FF,#005FB8FF role=button
                93 StackPanel at=233,455,321,43 by=scroll in=36 limit=225,56,419,560 role=none
                  94 TextBlock at=233,455,321,16 box=321,16 by=scroll caps=9.0994 drawn face=Barlow heard="Play videos" in=36 layout=61.59,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text="Play videos"
                  95 TextBlock at=233,471,321,27 box=321,27 by=scroll caps=7.6995 drawn face=Barlow heard="A game's clip plays in its picture once the focus rests on it, where it has one." in=36 layout=306.57,26.4 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="A game's clip plays in its picture once the focus rests on it, where it has one."
            96 Border at=225,503,419,52 by=scroll focus heard="Smooth motion, On" in=36 limit=225,56,419,560 name=SettingsRow.smooth paints=#D6D6D6FF role=none
              97 DockPanel at=233,507,403,43 by=scroll in=36 limit=225,56,419,560 role=none
                98 LunaSwitch at=570,513,58,32 by=scroll drawn focus heard="Smooth motion" in=36 limit=225,56,419,560 op paints=#000000FF,#005FB8FF role=button
                99 StackPanel at=233,507,321,43 by=scroll in=36 limit=225,56,419,560 role=none
                  100 TextBlock at=233,507,321,16 box=321,16 by=scroll caps=9.0994 drawn face=Barlow heard="Smooth motion" in=36 layout=83.9,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text="Smooth motion"
                  101 TextBlock at=233,523,321,27 box=321,27 by=scroll caps=7.6995 drawn face=Barlow heard="One frame per refresh when the display's rate is within 1% of the console's, the sound resampled by the difference." in=36 layout=292.27,26.4 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="One frame per refresh when the display's rate is within 1% of the console's, the sound resampled by the difference."
            102 Border at=225,555,419,48 by=scroll focus heard="Status bar, On" in=36 limit=225,56,419,560 name=SettingsRow.statusbar paints=#D6D6D6FF role=none
              103 DockPanel at=233,559,403,39 by=scroll in=36 limit=225,56,419,560 role=none
                104 LunaSwitch at=570,563,58,32 by=scroll drawn focus heard="Status bar" in=36 limit=225,56,419,560 op paints=#000000FF,#005FB8FF role=button
                105 StackPanel at=233,563,321,30 by=scroll in=36 limit=225,56,419,560 role=none
                  106 TextBlock at=233,563,321,16 box=321,16 by=scroll caps=9.0994 drawn face=Barlow heard="Status bar" in=36 layout=55.84,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text size=13 text="Status bar"
                  107 TextBlock at=233,579,321,14 box=321,14 by=scroll caps=7.6995 drawn face=Barlow heard="The desktop's bar along the bottom of the window." in=36 layout=232.06,13.2 limit=225,56,419,560 paints=#5F5F5FFF role=text size=11 text="The desktop's bar along the bottom of the window."
            108 Border at=225,603,419,48 by=scroll focus heard="Messages, On" in=36 limit=225,56,419,560 name=SettingsRow.messages paints=#D6D6D6FF role=none seen=225,603,419,13
              109 DockPanel at=233,607,403,39 by=scroll in=36 limit=225,56,419,560 role=none seen=233,607,403,9
                110 LunaSwitch at=570,611,58,32 by=scroll drawn focus heard=Messages in=36 limit=225,56,419,560 op paints=#000000FF,#005FB8FF role=button seen=570,611,58,5
                111 StackPanel at=233,611,321,30 by=scroll in=36 limit=225,56,419,560 paints=#5F5F5FFF role=none seen=233,611,321,5
                  112 TextBlock at=233,611,321,16 box=321,16 by=scroll caps=9.0994 drawn face=Barlow heard=Messages in=36 layout=53.6,15.6 limit=225,56,419,560 paints=#1F1F1FFF role=text seen=233,611,321,5 size=13 text=Messages
focus none
