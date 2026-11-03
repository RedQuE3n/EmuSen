# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsConsoles-1920x1200
size 1920 1200
kind menu
root SettingsLayout#SettingsScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case SettingsConsoles
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 WholeRow at=96,60,1728,66 by=panel heard=Sections limit=0,0,1920,1200 name=SettingsSections role=none
          4 ToggleButton at=96,60,238.5,66 by=panel drawn focus heard=Controllers limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=CONTROLLERS
            5 AccessText at=120,72,190.5,37.5 box=127,25 by=clip caps=13.999 drawn face=Barlow layout=126.94,24 limit=96,60,238.5,66 owner=4 paints=#D4D4D4FF role=none size=20 text=CONTROLLERS
          6 ToggleButton at=346.5,60,156,66 by=panel drawn focus heard=Display limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=DISPLAY
            7 AccessText at=370.5,72,108,37.5 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=346.5,60,156,66 owner=6 paints=#D4D4D4FF role=none size=20 text=DISPLAY
          8 ToggleButton at=514.5,60,139.5,66 by=panel drawn focus heard=Sound limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=SOUND
            9 AccessText at=538.5,72,91.5,37.5 box=61,25 by=clip caps=13.999 drawn face=Barlow layout=60.84,24 limit=514.5,60,139.5,66 owner=8 paints=#D4D4D4FF role=none size=20 text=SOUND
          10 ToggleButton at=666,60,187.5,66 by=panel drawn focus heard=Consoles limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=CONSOLES
            11 AccessText at=690,72,139.5,37.5 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=666,60,187.5,66 owner=10 paints=#EEEEF0FF role=none size=20 text=CONSOLES
          12 ToggleButton at=865.5,60,160.5,66 by=panel drawn focus heard=Library limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=LIBRARY
            13 AccessText at=889.5,72,112.5,37.5 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.16,24 limit=865.5,60,160.5,66 owner=12 paints=#D4D4D4FF role=none size=20 text=LIBRARY
          14 ToggleButton at=1038,60,160.5,66 by=panel drawn focus heard=EmuSen limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=EMUSEN
            15 AccessText at=1062,72,112.5,37.5 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.04,24 limit=1038,60,160.5,66 owner=14 paints=#D4D4D4FF role=none size=20 text=EMUSEN
        16 StackPanel at=96,162,1056,810 by=panel heard=Consoles limit=0,0,1920,1200 name=SettingsRows role=none
          17 MenuRow at=96,162,1056,81 by=panel heard=NES limit=0,0,1920,1200 name=SettingsRow.console:NES role=listitem
          18 MenuRow at=96,243,1056,81 by=panel heard=SG-1000 limit=0,0,1920,1200 name=SettingsRow.console:SG-1000 role=listitem
          19 MenuRow at=96,324,1056,81 by=panel heard="Master System" limit=0,0,1920,1200 name="SettingsRow.console:Master System" role=listitem
          20 MenuRow at=96,405,1056,81 by=panel heard=Genesis limit=0,0,1920,1200 name=SettingsRow.console:Genesis role=listitem
          21 MenuRow at=96,486,1056,81 by=panel heard="Game Boy" limit=0,0,1920,1200 name=SettingsRow.console:GB role=listitem
          22 MenuRow at=96,567,1056,81 by=panel heard="Super Nintendo" limit=0,0,1920,1200 name=SettingsRow.console:SNES role=listitem
          23 MenuRow at=96,648,1056,81 by=panel heard="Sega CD" limit=0,0,1920,1200 name="SettingsRow.console:Sega CD" role=listitem
          24 MenuRow at=96,729,1056,81 by=panel heard=PlayStation limit=0,0,1920,1200 name=SettingsRow.console:PlayStation role=listitem
          25 MenuRow at=96,810,1056,81 by=panel heard="Nintendo 64" limit=0,0,1920,1200 name=SettingsRow.console:N64 role=listitem
          26 MenuRow at=96,891,1056,81 by=panel heard="Game Boy Color" limit=0,0,1920,1200 name=SettingsRow.console:GBC role=listitem
        27 Border at=1200,162,624,867 by=panel limit=0,0,1920,1200 name=SettingsPane paints=#26262BFF role=none
          28 StackPanel at=1236,198,552,795 by=clip limit=1200,162,624,867 role=none
            29 TextBlock at=1236,198,552,79.5 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="Super Nintendo · 5 games · engine VenusRT (Rust)" layout=312.33,52.8 limit=1200,162,624,867 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Super Nintendo · 5 games · engine VenusRT (Rust)"
            30 TextBlock at=1236,289.5,552,43.5 by=clip drawn heard="✓ No firmware needed" limit=1200,162,624,867 name=SettingsLine paints=#FFFFFFFF role=text
            31 TextBlock at=1236,345,552,79.5 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="CRT profile Off · SA-1 catch-up After every S-CPU instruction (exact)" layout=327.23,52.8 limit=1200,162,624,867 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="CRT profile Off · SA-1 catch-up After every S-CPU instruction (exact)"
            32 TextBlock at=1236,436.5,552,120 box=368,80 by=clip caps=15.3989 drawn face=Barlow heard="What every Super Nintendo game shares, unless the game's own options say otherwise." layout=353.36,79.2 limit=1200,162,624,867 name=SettingsLine paints=#8D8D91FF role=text size=22 text="What every Super Nintendo game shares, unless the game's own options say otherwise."
        33 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          34 HintBar at=96,1074,1728,45 by=panel heard="Open, Section, Back, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="SettingsLayout SettingsScreen" settings="consoles  console:Sega CD" top=none
press up focus="SettingsLayout SettingsScreen" settings="consoles  console:SNES" top=none
press back focus="HubLayout HomeScreen" top=none
