# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsShaders-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsShaders
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=tab
          4 SectionTab at=64,40,104,44 by=panel drawn focus heard=Display limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=DISPLAY
            5 AccessText at=80,48,72,25 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=64,40,104,44 owner=4 paints=#D4D4D4FF role=none size=20 text=DISPLAY
          6 SectionTab at=176,40,115,44 by=panel drawn focus heard=Shaders limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=SHADERS
            7 AccessText at=192,48,83,25 box=83,25 by=clip caps=13.999 drawn face=Barlow layout=82.2,24 limit=176,40,115,44 owner=6 paints=#D4D4D4FF role=none size=20 text=SHADERS
          8 SectionTab at=299,40,186,44 by=panel drawn focus heard="Super Nintendo" limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="SUPER NINTENDO"
            9 AccessText at=315,48,154,25 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=299,40,186,44 owner=8 paints=#EEEEF0FF role=none size=20 text="SUPER NINTENDO"
        10 SelectionStack at=64,108,704,378 by=panel heard="Super Nintendo" limit=0,0,1280,800 name=SettingsRows role=list
          11 MenuRow at=64,108,704,54 by=panel focus heard="Parameters, None" limit=0,0,1280,800 name=SettingsRow.params role=listitem
          12 MenuRow at=64,162,704,54 by=panel focus heard="Search, …" limit=0,0,1280,800 name=SettingsRow.find role=listitem
          13 MenuRow at=64,216,704,54 by=panel focus heard="None, ● In use" limit=0,0,1280,800 name=SettingsRow.use:None role=listitem
          14 MenuRow at=64,270,704,54 by=panel focus heard=CRT limit=0,0,1280,800 name=SettingsRow.use:CRT role=listitem
          15 MenuRow at=64,324,704,54 by=panel focus heard="CRT (Lottes)" limit=0,0,1280,800 name="SettingsRow.use:CRT (Lottes)" role=listitem
          16 MenuRow at=64,378,704,54 by=panel focus heard=Scanlines limit=0,0,1280,800 name=SettingsRow.use:Scanlines role=listitem
          17 MenuRow at=64,432,704,54 by=panel focus heard="Simple CRT" limit=0,0,1280,800 name="SettingsRow.use:Simple CRT" role=listitem
        18 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          19 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            20 TextBlock at=824,132,368,53 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="The parameters of None, for the Super Nintendo." layout=351.03,52.8 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="The parameters of None, for the Super Nintendo."
        21 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          22 HintBar at=64,716,1152,30 by=panel heard="Open, Back, Home" limit=0,0,1280,800 role=group
focus none
press down focus="MenuRow SettingsRow.find" settings="display Display › Shaders › Super Nintendo find" top=none
press up focus="MenuRow SettingsRow.params" settings="display Display › Shaders › Super Nintendo params" top=none
press back focus="MenuRow SettingsRow.shader:SNES" settings="display Display › Shaders shader:SNES" top=none
