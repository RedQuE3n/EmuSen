# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsTextSize-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsTextSize
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=tab
          4 SectionTab at=64,40,159,44 by=panel drawn focus heard=Controllers limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=CONTROLLERS
            5 AccessText at=80,48,127,25 box=127,25 by=clip caps=13.999 drawn face=Barlow layout=126.94,24 limit=64,40,159,44 owner=4 paints=#D4D4D4FF role=none size=20 text=CONTROLLERS
          6 SectionTab at=231,40,104,44 by=panel drawn focus heard=Display limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=DISPLAY
            7 AccessText at=247,48,72,25 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=231,40,104,44 owner=6 paints=#EEEEF0FF role=none size=20 text=DISPLAY
          8 SectionTab at=343,40,93,44 by=panel drawn focus heard=Sound limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=SOUND
            9 AccessText at=359,48,61,25 box=61,25 by=clip caps=13.999 drawn face=Barlow layout=60.84,24 limit=343,40,93,44 owner=8 paints=#D4D4D4FF role=none size=20 text=SOUND
          10 SectionTab at=444,40,125,44 by=panel drawn focus heard=Consoles limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=CONSOLES
            11 AccessText at=460,48,93,25 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=444,40,125,44 owner=10 paints=#D4D4D4FF role=none size=20 text=CONSOLES
          12 SectionTab at=577,40,107,44 by=panel drawn focus heard=Library limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=LIBRARY
            13 AccessText at=593,48,75,25 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.16,24 limit=577,40,107,44 owner=12 paints=#D4D4D4FF role=none size=20 text=LIBRARY
          14 SectionTab at=692,40,107,44 by=panel drawn focus heard=EmuSen limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=EMUSEN
            15 AccessText at=708,48,75,25 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.04,24 limit=692,40,107,44 owner=14 paints=#D4D4D4FF role=none size=20 text=EMUSEN
        16 SelectionStack at=64,108,704,540 by=panel heard=Display limit=0,0,1280,800 name=SettingsRows role=list
          17 MenuRow at=64,108,704,54 by=panel focus heard="Picture shape, TV shape" limit=0,0,1280,800 name=SettingsRow.shape role=listitem
          18 MenuRow at=64,162,704,54 by=panel focus heard="CRT shader, Off" limit=0,0,1280,800 name=SettingsRow.crt role=listitem
          19 MenuRow at=64,216,704,54 by=panel focus heard="Shader levers" limit=0,0,1280,800 name=SettingsRow.levers role=listitem
          20 MenuRow at=64,270,704,54 by=panel focus heard=Shaders limit=0,0,1280,800 name=SettingsRow.shaders role=listitem
          21 MenuRow at=64,324,704,54 by=panel focus heard="Crop overscan, Whole picture" limit=0,0,1280,800 name=SettingsRow.crop role=listitem
          22 MenuRow at=64,378,704,54 by=panel focus heard="Text size, Default" limit=0,0,1280,800 name=SettingsRow.textsize role=listitem
          23 MenuRow at=64,432,704,54 by=panel focus heard="Background art on Home, On" limit=0,0,1280,800 name=SettingsRow.art role=listitem
          24 MenuRow at=64,486,704,54 by=panel focus heard="Reduced motion, System" limit=0,0,1280,800 name=SettingsRow.motion role=listitem
          25 MenuRow at=64,540,704,54 by=panel focus heard="Light or dark, Dark" limit=0,0,1280,800 name=SettingsRow.variant role=listitem
          26 MenuRow at=64,594,704,54 by=panel focus heard="Play videos, On" limit=0,0,1280,800 name=SettingsRow.videos role=listitem
        27 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          28 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            29 TextBlock at=824,132,368,27 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Default · The quick brown fox" layout=265.5,26.4 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Default · The quick brown fox"
            30 TextBlock at=824,167,368,31 box=368,31 by=clip caps=17.8488 drawn face=Barlow heard="Large · The quick brown fox" layout=292.77,30.6 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=25.5 text="Large · The quick brown fox"
            31 TextBlock at=824,206,368,35 box=368,35 by=clip caps=19.9486 drawn face=Barlow heard="Larger · The quick brown fox" layout=337.55,34.2 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=28.5 text="Larger · The quick brown fox"
            32 TextBlock at=824,249,368,53 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="Text size scales every word on the big screen; nothing is cut off at any size." layout=345.77,52.8 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Text size scales every word on the big screen; nothing is cut off at any size."
        33 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          34 HintBar at=64,716,1152,30 by=panel heard="Change, Value, Section, Back, Home" limit=0,0,1280,800 role=group
focus none
press down focus="MenuRow SettingsRow.art" settings="display  art" top=none
press up focus="MenuRow SettingsRow.textsize" settings="display  textsize" top=none
press back focus="HubLayout HomeScreen" top=none
