# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsShaderParameters-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsShaderParameters
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
          8 SectionTab at=299,40,186,44 by=panel drawn focus heard="Super Nintendo" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="SUPER NINTENDO"
            9 AccessText at=315,48,154,25 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=299,40,186,44 owner=8 paints=#D4D4D4FF role=none size=20 text="SUPER NINTENDO"
          10 SectionTab at=493,40,149,44 by=panel drawn focus heard=Parameters limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=PARAMETERS
            11 AccessText at=509,48,117,25 box=117,25 by=clip caps=13.999 drawn face=Barlow layout=116.52,24 limit=493,40,149,44 owner=10 paints=#EEEEF0FF role=none size=20 text=PARAMETERS
        12 RowList at=64,108,704,540 by=panel heard=Parameters limit=0,0,1280,800 name=SettingsRows role=list
          13 MenuRow at=64,108,704,54 by=panel focus heard="Search parameters, …" limit=0,0,1280,800 name=SettingsRow.paramsearch role=listitem
          14 MenuRow at=64,162,704,54 by=panel focus heard="Reset all" limit=0,0,1280,800 name=SettingsRow.paramreset role=listitem
          15 MenuRow at=64,216,704,54 by=panel focus heard="Quality, Balanced" limit=0,0,1280,800 name=SettingsRow.p:quality role=listitem
          16 MenuRow at=64,270,704,54 by=panel focus heard="Signal, Composite" limit=0,0,1280,800 name=SettingsRow.p:signal role=listitem
          17 MenuRow at=64,324,704,54 by=panel focus heard="Television standard, The game's" limit=0,0,1280,800 name=SettingsRow.p:standard role=listitem
          18 MenuRow at=64,378,704,54 by=panel focus heard="PAL colour decoding, Delay line" limit=0,0,1280,800 name=SettingsRow.p:palDecoder role=listitem
          19 MenuRow at=64,432,704,54 by=panel focus heard="Screen, Consumer TV, 20-inch" limit=0,0,1280,800 name=SettingsRow.p:screen role=listitem
          20 MenuRow at=64,486,704,54 by=panel focus heard="Colour, North America" limit=0,0,1280,800 name=SettingsRow.p:colour role=listitem
          21 MenuRow at=64,540,704,54 by=panel focus heard="Curvature, 1" limit=0,0,1280,800 name=SettingsRow.p:curvature role=listitem
          22 MenuRow at=64,594,704,54 by=panel focus heard="Picture brightness, Bright" limit=0,0,1280,800 name=SettingsRow.p:level role=listitem
        23 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          24 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            25 TextBlock at=824,132,368,27 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Words in a parameter's name." layout=277,26.4 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Words in a parameter's name."
        26 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          27 HintBar at=64,716,1152,30 by=panel heard="Change, Value, Back, Home" limit=0,0,1280,800 role=group
focus none
press down focus="MenuRow SettingsRow.paramreset" settings="display Display › Shaders › Super Nintendo › Parameters paramreset" top=none
press up focus="MenuRow SettingsRow.paramsearch" settings="display Display › Shaders › Super Nintendo › Parameters paramsearch" top=none
press back focus="MenuRow SettingsRow.params" settings="display Display › Shaders › Super Nintendo params" top=none
