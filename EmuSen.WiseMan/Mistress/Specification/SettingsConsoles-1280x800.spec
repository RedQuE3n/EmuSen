# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsConsoles-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsConsoles
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=none
          4 ToggleButton at=64,40,159,44 by=panel drawn focus heard=Controllers limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=CONTROLLERS
            5 AccessText at=80,48,127,25 box=127,25 by=clip caps=13.999 drawn face=Barlow layout=126.94,24 limit=64,40,159,44 owner=4 paints=#D4D4D4FF role=none size=20 text=CONTROLLERS
          6 ToggleButton at=231,40,104,44 by=panel drawn focus heard=Display limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=DISPLAY
            7 AccessText at=247,48,72,25 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=231,40,104,44 owner=6 paints=#D4D4D4FF role=none size=20 text=DISPLAY
          8 ToggleButton at=343,40,93,44 by=panel drawn focus heard=Sound limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SOUND
            9 AccessText at=359,48,61,25 box=61,25 by=clip caps=13.999 drawn face=Barlow layout=60.84,24 limit=343,40,93,44 owner=8 paints=#D4D4D4FF role=none size=20 text=SOUND
          10 ToggleButton at=444,40,125,44 by=panel drawn focus heard=Consoles limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=CONSOLES
            11 AccessText at=460,48,93,25 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=444,40,125,44 owner=10 paints=#EEEEF0FF role=none size=20 text=CONSOLES
          12 ToggleButton at=577,40,107,44 by=panel drawn focus heard=Library limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=LIBRARY
            13 AccessText at=593,48,75,25 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.16,24 limit=577,40,107,44 owner=12 paints=#D4D4D4FF role=none size=20 text=LIBRARY
          14 ToggleButton at=692,40,107,44 by=panel drawn focus heard=EmuSen limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=EMUSEN
            15 AccessText at=708,48,75,25 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.04,24 limit=692,40,107,44 owner=14 paints=#D4D4D4FF role=none size=20 text=EMUSEN
        16 StackPanel at=64,108,704,540 by=panel heard=Consoles limit=0,0,1280,800 name=SettingsRows role=none
          17 MenuRow at=64,108,704,54 by=panel heard=NES limit=0,0,1280,800 name=SettingsRow.console:NES role=listitem
          18 MenuRow at=64,162,704,54 by=panel heard=SG-1000 limit=0,0,1280,800 name=SettingsRow.console:SG-1000 role=listitem
          19 MenuRow at=64,216,704,54 by=panel heard="Master System" limit=0,0,1280,800 name="SettingsRow.console:Master System" role=listitem
          20 MenuRow at=64,270,704,54 by=panel heard=Genesis limit=0,0,1280,800 name=SettingsRow.console:Genesis role=listitem
          21 MenuRow at=64,324,704,54 by=panel heard="Game Boy" limit=0,0,1280,800 name=SettingsRow.console:GB role=listitem
          22 MenuRow at=64,378,704,54 by=panel heard="Super Nintendo" limit=0,0,1280,800 name=SettingsRow.console:SNES role=listitem
          23 MenuRow at=64,432,704,54 by=panel heard="Sega CD" limit=0,0,1280,800 name="SettingsRow.console:Sega CD" role=listitem
          24 MenuRow at=64,486,704,54 by=panel heard=PlayStation limit=0,0,1280,800 name=SettingsRow.console:PlayStation role=listitem
          25 MenuRow at=64,540,704,54 by=panel heard="Nintendo 64" limit=0,0,1280,800 name=SettingsRow.console:N64 role=listitem
          26 MenuRow at=64,594,704,54 by=panel heard="Game Boy Color" limit=0,0,1280,800 name=SettingsRow.console:GBC role=listitem
        27 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          28 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            29 TextBlock at=824,132,368,53 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="Super Nintendo · 5 games · engine VenusRT (Rust)" layout=312.33,52.8 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Super Nintendo · 5 games · engine VenusRT (Rust)"
            30 TextBlock at=824,193,368,29 by=clip drawn heard="✓ No firmware needed" limit=800,108,416,578 name=SettingsLine paints=#FFFFFFFF role=text
            31 TextBlock at=824,230,368,53 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="CRT profile Off · SA-1 catch-up After every S-CPU instruction (exact)" layout=327.23,52.8 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="CRT profile Off · SA-1 catch-up After every S-CPU instruction (exact)"
            32 TextBlock at=824,291,368,80 box=368,80 by=clip caps=15.3989 drawn face=Barlow heard="What every Super Nintendo game shares, unless the game's own options say otherwise." layout=353.36,79.2 limit=800,108,416,578 name=SettingsLine paints=#8D8D91FF role=text size=22 text="What every Super Nintendo game shares, unless the game's own options say otherwise."
        33 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          34 HintBar at=64,716,1152,30 by=panel heard="Open, Section, Back, Home" limit=0,0,1280,800 role=group
focus none
press down focus="SettingsLayout SettingsScreen" settings="consoles  console:Sega CD" top=none
press up focus="SettingsLayout SettingsScreen" settings="consoles  console:SNES" top=none
press back focus="HubLayout HomeScreen" top=none
