# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsEmuSen-1920x1200
size 1920 1200
kind menu
root SettingsLayout#SettingsScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case SettingsEmuSen
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 WholeRow at=96,60,1728,66 by=panel heard=Sections limit=0,0,1920,1200 name=SettingsSections role=tab
          4 SectionTab at=96,60,238.5,66 by=panel drawn focus heard=Controllers limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=CONTROLLERS
            5 AccessText at=120,72,190.5,37.5 box=127,25 by=clip caps=13.999 drawn face=Barlow layout=126.94,24 limit=96,60,238.5,66 owner=4 paints=#D4D4D4FF role=none size=20 text=CONTROLLERS
          6 SectionTab at=346.5,60,156,66 by=panel drawn focus heard=Display limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=DISPLAY
            7 AccessText at=370.5,72,108,37.5 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=346.5,60,156,66 owner=6 paints=#D4D4D4FF role=none size=20 text=DISPLAY
          8 SectionTab at=514.5,60,139.5,66 by=panel drawn focus heard=Sound limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=SOUND
            9 AccessText at=538.5,72,91.5,37.5 box=61,25 by=clip caps=13.999 drawn face=Barlow layout=60.84,24 limit=514.5,60,139.5,66 owner=8 paints=#D4D4D4FF role=none size=20 text=SOUND
          10 SectionTab at=666,60,187.5,66 by=panel drawn focus heard=Consoles limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=CONSOLES
            11 AccessText at=690,72,139.5,37.5 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=666,60,187.5,66 owner=10 paints=#D4D4D4FF role=none size=20 text=CONSOLES
          12 SectionTab at=865.5,60,160.5,66 by=panel drawn focus heard=Library limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=LIBRARY
            13 AccessText at=889.5,72,112.5,37.5 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.16,24 limit=865.5,60,160.5,66 owner=12 paints=#D4D4D4FF role=none size=20 text=LIBRARY
          14 SectionTab at=1038,60,160.5,66 by=panel drawn focus heard=EmuSen limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=EMUSEN
            15 AccessText at=1062,72,112.5,37.5 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.04,24 limit=1038,60,160.5,66 owner=14 paints=#EEEEF0FF role=none size=20 text=EMUSEN
        16 SelectionStack at=96,162,1056,486 by=panel heard=EmuSen limit=0,0,1920,1200 name=SettingsRows role=list
          17 MenuRow at=96,162,1056,81 by=panel focus heard="Saves and states" limit=0,0,1920,1200 name=SettingsRow.saves role=listitem
          18 MenuRow at=96,243,1056,81 by=panel focus heard="Big picture, On startup" limit=0,0,1920,1200 name=SettingsRow.bigpicture role=listitem
          19 MenuRow at=96,324,1056,81 by=panel focus heard="Pause in the background, On" limit=0,0,1920,1200 name=SettingsRow.background role=listitem
          20 MenuRow at=96,405,1056,81 by=panel focus heard="Logging, Off" limit=0,0,1920,1200 name=SettingsRow.logging role=listitem
          21 MenuRow at=96,486,1056,81 by=panel focus heard=Shell limit=0,0,1920,1200 name=SettingsRow.shell role=listitem
          22 MenuRow at=96,567,1056,81 by=panel focus heard="About, EmuSen 0.9.1" limit=0,0,1920,1200 name=SettingsRow.about role=listitem
        23 Border at=1200,162,624,867 by=panel limit=0,0,1920,1200 name=SettingsPane paints=#26262BFF role=none
          24 StackPanel at=1236,198,552,795 by=clip limit=1200,162,624,867 role=none
            25 TextBlock at=1236,198,552,79.5 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="States and saves live beside the games, with one resume point kept per game." layout=362.43,52.8 limit=1200,162,624,867 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="States and saves live beside the games, with one resume point kept per game."
            26 TextBlock at=1236,289.5,552,40.5 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Resume · Always start again" layout=257.84,26.4 limit=1200,162,624,867 name=SettingsLine paints=#8D8D91FF role=text size=22 text="Resume · Always start again"
        27 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          28 HintBar at=96,1074,1728,45 by=panel heard="Open, Value, Section, Back, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="MenuRow SettingsRow.bigpicture" settings="emusen  bigpicture" top=none
press up focus="MenuRow SettingsRow.saves" settings="emusen  saves" top=none
press back focus="HubLayout HomeScreen" top=none
