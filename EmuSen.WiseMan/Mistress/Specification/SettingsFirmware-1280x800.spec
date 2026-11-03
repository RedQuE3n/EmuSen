# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsFirmware-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsFirmware
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=none
          4 ToggleButton at=64,40,125,44 by=panel drawn focus heard=Consoles limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=CONSOLES
            5 AccessText at=80,48,93,25 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=64,40,125,44 owner=4 paints=#D4D4D4FF role=none size=20 text=CONSOLES
          6 ToggleButton at=197,40,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PLAYSTATION
            7 AccessText at=213,48,114,25 box=114,25 by=clip caps=13.999 drawn face=Barlow layout=113.56,24 limit=197,40,146,44 owner=6 paints=#D4D4D4FF role=none size=20 text=PLAYSTATION
          8 ToggleButton at=351,40,127,44 by=panel drawn focus heard=Firmware limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=FIRMWARE
            9 AccessText at=367,48,95,25 box=95,25 by=clip caps=13.999 drawn face=Barlow layout=94.76,24 limit=351,40,127,44 owner=8 paints=#EEEEF0FF role=none size=20 text=FIRMWARE
        10 StackPanel at=64,108,704,270 by=panel heard=Firmware limit=0,0,1280,800 name=SettingsRows role=none
          11 MenuRow at=64,108,704,54 by=panel heard="The Japanese PlayStation's BIOS, Not needed" limit=0,0,1280,800 name=SettingsRow.fw:scph5500.bin role=listitem
          12 MenuRow at=64,162,704,54 by=panel heard="The American PlayStation's BIOS, Not needed" limit=0,0,1280,800 name=SettingsRow.fw:scph5501.bin role=listitem
          13 MenuRow at=64,216,704,54 by=panel heard="The European PlayStation's BIOS, Not needed" limit=0,0,1280,800 name=SettingsRow.fw:scph5502.bin role=listitem
          14 MenuRow at=64,270,704,54 by=panel heard="Folder, EmuSen's folder" limit=0,0,1280,800 name=SettingsRow.fwfolder role=listitem
          15 MenuRow at=64,324,704,54 by=panel heard="Check again" limit=0,0,1280,800 name=SettingsRow.fwcheck role=listitem
        16 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          17 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            18 TextBlock at=824,132,368,80 box=368,80 by=clip caps=15.3989 drawn face=Barlow heard="The Japanese PlayStation's BIOS: none of your own is needed; the open replacement runs most games." layout=360.21,79.2 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="The Japanese PlayStation's BIOS: none of your own is needed; the open replacement runs most games."
            19 TextBlock at=824,220,368,53 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="Your own copy (scph5500.bin) is used when it is found." layout=347.69,52.8 limit=800,108,416,578 name=SettingsLine paints=#8D8D91FF role=text size=22 text="Your own copy (scph5500.bin) is used when it is found."
        20 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          21 HintBar at=64,716,1152,30 by=panel heard="Change, Back, Home" limit=0,0,1280,800 role=group
focus none
press down focus="SettingsLayout SettingsScreen" settings="consoles Consoles › PlayStation › Firmware fw:scph5501.bin" top=none
press up focus="SettingsLayout SettingsScreen" settings="consoles Consoles › PlayStation › Firmware fw:scph5500.bin" top=none
press back focus="SettingsLayout SettingsScreen" settings="consoles Consoles › PlayStation firmware" top=none
