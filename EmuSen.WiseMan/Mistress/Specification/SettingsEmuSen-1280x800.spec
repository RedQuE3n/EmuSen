# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsEmuSen-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsEmuSen
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=tab
          4 SectionTab at=64,40,159,44 by=panel drawn focus heard=Controllers limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=CONTROLLERS
            5 AccessText at=80,48,127,25 box=127,25 by=clip caps=13.999 drawn face=Barlow layout=126.94,24 limit=64,40,159,44 owner=4 paints=#D4D4D4FF role=none size=20 text=CONTROLLERS
          6 SectionTab at=231,40,104,44 by=panel drawn focus heard=Display limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=DISPLAY
            7 AccessText at=247,48,72,25 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=231,40,104,44 owner=6 paints=#D4D4D4FF role=none size=20 text=DISPLAY
          8 SectionTab at=343,40,93,44 by=panel drawn focus heard=Sound limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=SOUND
            9 AccessText at=359,48,61,25 box=61,25 by=clip caps=13.999 drawn face=Barlow layout=60.84,24 limit=343,40,93,44 owner=8 paints=#D4D4D4FF role=none size=20 text=SOUND
          10 SectionTab at=444,40,125,44 by=panel drawn focus heard=Consoles limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=CONSOLES
            11 AccessText at=460,48,93,25 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=444,40,125,44 owner=10 paints=#D4D4D4FF role=none size=20 text=CONSOLES
          12 SectionTab at=577,40,107,44 by=panel drawn focus heard=Library limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=LIBRARY
            13 AccessText at=593,48,75,25 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.16,24 limit=577,40,107,44 owner=12 paints=#D4D4D4FF role=none size=20 text=LIBRARY
          14 SectionTab at=692,40,107,44 by=panel drawn focus heard=EmuSen limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=EMUSEN
            15 AccessText at=708,48,75,25 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.04,24 limit=692,40,107,44 owner=14 paints=#EEEEF0FF role=none size=20 text=EMUSEN
        16 RowList at=64,108,704,324 by=panel heard=EmuSen limit=0,0,1280,800 name=SettingsRows role=list
          17 MenuRow at=64,108,704,54 by=panel focus heard="Saves and states" limit=0,0,1280,800 name=SettingsRow.saves role=listitem
          18 MenuRow at=64,162,704,54 by=panel focus heard="Big picture, On startup" limit=0,0,1280,800 name=SettingsRow.bigpicture role=listitem
          19 MenuRow at=64,216,704,54 by=panel focus heard="Pause in the background, On" limit=0,0,1280,800 name=SettingsRow.background role=listitem
          20 MenuRow at=64,270,704,54 by=panel focus heard="Logging, Off" limit=0,0,1280,800 name=SettingsRow.logging role=listitem
          21 MenuRow at=64,324,704,54 by=panel focus heard=Shell limit=0,0,1280,800 name=SettingsRow.shell role=listitem
          22 MenuRow at=64,378,704,54 by=panel focus heard="About, EmuSen 0.9.1" limit=0,0,1280,800 name=SettingsRow.about role=listitem
        23 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          24 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            25 TextBlock at=824,132,368,53 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="States and saves live beside the games, with one resume point kept per game." layout=362.43,52.8 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="States and saves live beside the games, with one resume point kept per game."
            26 TextBlock at=824,193,368,27 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Resume · Always start again" layout=257.84,26.4 limit=800,108,416,578 name=SettingsLine paints=#8D8D91FF role=text size=22 text="Resume · Always start again"
        27 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          28 HintBar at=64,716,1152,30 by=panel heard="Open, Value, Section, Back, Home" limit=0,0,1280,800 role=group
focus none
press down focus="MenuRow SettingsRow.bigpicture" settings="emusen  bigpicture" top=none
press up focus="MenuRow SettingsRow.saves" settings="emusen  saves" top=none
press back focus="HubLayout HomeScreen" top=none
