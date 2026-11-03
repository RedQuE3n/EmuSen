# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsShaderLevers-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsShaderLevers
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=tab
          4 SectionTab at=64,40,104,44 by=panel drawn focus heard=Display limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=DISPLAY
            5 AccessText at=80,48,72,25 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=64,40,104,44 owner=4 paints=#D4D4D4FF role=none size=20 text=DISPLAY
          6 SectionTab at=176,40,176,44 by=panel drawn focus heard="Shader levers" limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="SHADER LEVERS"
            7 AccessText at=192,48,144,25 box=144,25 by=clip caps=13.999 drawn face=Barlow layout=143.58,24 limit=176,40,176,44 owner=6 paints=#EEEEF0FF role=none size=20 text="SHADER LEVERS"
        8 SelectionStack at=64,108,704,540 by=panel heard="Shader levers" limit=0,0,1280,800 name=SettingsRows role=list
          9 MenuRow at=64,108,704,54 by=panel focus heard="Signal, Composite" limit=0,0,1280,800 name=SettingsRow.signal role=listitem
          10 MenuRow at=64,162,704,54 by=panel focus heard="Television standard, The game's" limit=0,0,1280,800 name=SettingsRow.standard role=listitem
          11 MenuRow at=64,216,704,54 by=panel focus heard="PAL colour decoding, Delay line" limit=0,0,1280,800 name=SettingsRow.palDecoder role=listitem
          12 MenuRow at=64,270,704,54 by=panel focus heard="Screen, Consumer TV, 20-inch" limit=0,0,1280,800 name=SettingsRow.screen role=listitem
          13 MenuRow at=64,324,704,54 by=panel focus heard="Colour, North America" limit=0,0,1280,800 name=SettingsRow.colour role=listitem
          14 MenuRow at=64,378,704,54 by=panel focus heard="Curvature, 1" limit=0,0,1280,800 name=SettingsRow.curvature role=listitem
          15 MenuRow at=64,432,704,54 by=panel focus heard="Picture brightness, Bright" limit=0,0,1280,800 name=SettingsRow.level role=listitem
          16 MenuRow at=64,486,704,54 by=panel focus heard="Display brightness, cd/m², 300" limit=0,0,1280,800 name=SettingsRow.displayNits role=listitem
          17 MenuRow at=64,540,704,54 by=panel focus heard="Display subpixels, RGB" limit=0,0,1280,800 name=SettingsRow.subpixels role=listitem
          18 MenuRow at=64,594,704,54 by=panel focus heard="Display colours, sRGB" limit=0,0,1280,800 name=SettingsRow.gamut role=listitem
        19 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          20 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            21 TextBlock at=824,132,368,27 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Signal, for the NES." layout=176.55,26.4 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Signal, for the NES."
        22 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          23 HintBar at=64,716,1152,30 by=panel heard="Change, Value, Back, Home" limit=0,0,1280,800 role=group
focus none
press down focus="MenuRow SettingsRow.standard" settings="display Display › Shader levers standard" top=none
press up focus="MenuRow SettingsRow.signal" settings="display Display › Shader levers signal" top=none
press back focus="MenuRow SettingsRow.levers" settings="display  levers" top=none
