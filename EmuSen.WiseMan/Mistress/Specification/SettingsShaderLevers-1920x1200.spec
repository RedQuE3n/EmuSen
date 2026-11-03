# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsShaderLevers-1920x1200
size 1920 1200
kind menu
root SettingsLayout#SettingsScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case SettingsShaderLevers
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF role=none
        3 WholeRow at=96,60,1728,66 by=panel heard=Sections limit=0,0,1920,1200 name=SettingsSections role=tab
          4 SectionTab at=96,60,156,66 by=panel drawn focus heard=Display limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=DISPLAY
            5 AccessText at=120,72,108,37.5 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=96,60,156,66 owner=4 paints=#D4D4D4FF role=none size=20 text=DISPLAY
          6 SectionTab at=264,60,264,66 by=panel drawn focus heard="Shader levers" limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="SHADER LEVERS"
            7 AccessText at=288,72,216,37.5 box=144,25 by=clip caps=13.999 drawn face=Barlow layout=143.58,24 limit=264,60,264,66 owner=6 paints=#EEEEF0FF role=none size=20 text="SHADER LEVERS"
        8 SelectionStack at=96,162,1056,810 by=panel heard="Shader levers" limit=0,0,1920,1200 name=SettingsRows role=list
          9 MenuRow at=96,162,1056,81 by=panel focus heard="Signal, Composite" limit=0,0,1920,1200 name=SettingsRow.signal role=listitem
          10 MenuRow at=96,243,1056,81 by=panel focus heard="Television standard, The game's" limit=0,0,1920,1200 name=SettingsRow.standard role=listitem
          11 MenuRow at=96,324,1056,81 by=panel focus heard="PAL colour decoding, Delay line" limit=0,0,1920,1200 name=SettingsRow.palDecoder role=listitem
          12 MenuRow at=96,405,1056,81 by=panel focus heard="Screen, Consumer TV, 20-inch" limit=0,0,1920,1200 name=SettingsRow.screen role=listitem
          13 MenuRow at=96,486,1056,81 by=panel focus heard="Colour, North America" limit=0,0,1920,1200 name=SettingsRow.colour role=listitem
          14 MenuRow at=96,567,1056,81 by=panel focus heard="Curvature, 1" limit=0,0,1920,1200 name=SettingsRow.curvature role=listitem
          15 MenuRow at=96,648,1056,81 by=panel focus heard="Picture brightness, Bright" limit=0,0,1920,1200 name=SettingsRow.level role=listitem
          16 MenuRow at=96,729,1056,81 by=panel focus heard="Display brightness, cd/m², 300" limit=0,0,1920,1200 name=SettingsRow.displayNits role=listitem
          17 MenuRow at=96,810,1056,81 by=panel focus heard="Display subpixels, RGB" limit=0,0,1920,1200 name=SettingsRow.subpixels role=listitem
          18 MenuRow at=96,891,1056,81 by=panel focus heard="Display colours, sRGB" limit=0,0,1920,1200 name=SettingsRow.gamut role=listitem
        19 Border at=1200,162,624,867 by=panel limit=0,0,1920,1200 name=SettingsPane paints=#26262BFF role=none
          20 StackPanel at=1236,198,552,795 by=clip limit=1200,162,624,867 role=none
            21 TextBlock at=1236,198,552,40.5 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Signal, for the NES." layout=176.55,26.4 limit=1200,162,624,867 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Signal, for the NES."
        22 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          23 HintBar at=96,1074,1728,45 by=panel heard="Change, Value, Back, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="MenuRow SettingsRow.standard" settings="display Display › Shader levers standard" top=none
press up focus="MenuRow SettingsRow.signal" settings="display Display › Shader levers signal" top=none
press back focus="MenuRow SettingsRow.levers" settings="display  levers" top=none
