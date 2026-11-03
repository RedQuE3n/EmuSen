# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsConsolePage-1920x1200
size 1920 1200
kind menu
root SettingsLayout#SettingsScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case SettingsConsolePage
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 WholeRow at=96,60,1728,66 by=panel heard=Sections limit=0,0,1920,1200 name=SettingsSections role=none
          4 ToggleButton at=96,60,187.5,66 by=panel drawn focus heard=Consoles limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=CONSOLES
            5 AccessText at=120,72,139.5,37.5 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=96,60,187.5,66 owner=4 paints=#D4D4D4FF role=none size=20 text=CONSOLES
          6 ToggleButton at=295.5,60,279,66 by=panel drawn focus heard="Super Nintendo" limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text="SUPER NINTENDO"
            7 AccessText at=319.5,72,231,37.5 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=295.5,60,279,66 owner=6 paints=#EEEEF0FF role=none size=20 text="SUPER NINTENDO"
        8 StackPanel at=96,162,1056,648 by=panel heard="Super Nintendo" limit=0,0,1920,1200 name=SettingsRows role=none
          9 MenuRow at=96,162,1056,81 by=panel heard="Engine, VenusRT (Rust)" limit=0,0,1920,1200 name=SettingsRow.engine role=listitem
          10 MenuRow at=96,243,1056,81 by=panel heard="Firmware, ⚠ None found" limit=0,0,1920,1200 name=SettingsRow.firmware role=listitem
          11 MenuRow at=96,324,1056,81 by=panel heard="Memory cards" limit=0,0,1920,1200 name=SettingsRow.cards role=listitem
          12 MenuRow at=96,405,1056,81 by=panel heard="SA-1 catch-up, After every S-CPU instruction (exact)" limit=0,0,1920,1200 name=SettingsRow.sa1_catch_up role=listitem
          13 MenuRow at=96,486,1056,81 by=panel heard="CRT profile, Off" limit=0,0,1920,1200 name=SettingsRow.crtprofile role=listitem
          14 MenuRow at=96,567,1056,81 by=panel heard="Shader levers" limit=0,0,1920,1200 name=SettingsRow.consolelevers role=listitem
          15 MenuRow at=96,648,1056,81 by=panel heard="Crop overscan, Whole picture" limit=0,0,1920,1200 name=SettingsRow.consolecrop role=listitem
          16 MenuRow at=96,729,1056,81 by=panel heard="Picture shape, TV shape" limit=0,0,1920,1200 name=SettingsRow.consoleshape role=listitem
        17 Border at=1200,162,624,867 by=panel limit=0,0,1920,1200 name=SettingsPane paints=#26262BFF role=none
          18 StackPanel at=1236,198,552,795 by=clip limit=1200,162,624,867 role=none
            19 TextBlock at=1236,198,552,159 box=368,106 by=clip caps=15.3989 drawn face=Barlow heard="VenusRT (Rust), the Super Nintendo's engine: every Super Nintendo game opens on it unless the game's own options choose another." layout=340.3,105.6 limit=1200,162,624,867 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="VenusRT (Rust), the Super Nintendo's engine: every Super Nintendo game opens on it unless the game's own options choose another."
        20 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          21 HintBar at=96,1074,1728,45 by=panel heard="Change, Value, Back, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="SettingsLayout SettingsScreen" settings="consoles Consoles › Super Nintendo firmware" top=none
press up focus="SettingsLayout SettingsScreen" settings="consoles Consoles › Super Nintendo engine" top=none
press back focus="SettingsLayout SettingsScreen" settings="consoles  console:SNES" top=none
