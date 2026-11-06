"""Development only: subset downloaded OFL font sources without runtime networking.
Install fonttools and brotli, put upstream TTF/OFL files in target/font-sources,
then run python scripts/subset-fonts.py. Provenance is in public/fonts/SOURCES.md.
"""
from pathlib import Path
from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
source=Path('target/font-sources')
output=Path('public/fonts');output.mkdir(parents=True,exist_ok=True)
for name,prefix in [('NotoSansKhmer','notosanskhmer'),('KantumruyPro','kantumruypro')]:
    path=next(p for p in source.glob(prefix+'-*.ttf') if 'Italic' not in p.name)
    for weight in [400,600]:
        font=TTFont(path)
        font=instantiateVariableFont(font,{'wght':weight,**({'wdth':100} if name=='NotoSansKhmer' else {})},inplace=True)
        options=subset.Options();options.flavor='woff2';options.layout_features=['*'];options.name_IDs=['*'];options.name_legacy=True
        worker=subset.Subsetter(options=options)
        worker.populate(unicodes=list(range(0x20,0x100))+list(range(0x1780,0x1800))+list(range(0x19e0,0x1a00))+list(range(0x2000,0x2070))+[0x25cc])
        worker.subset(font);font.flavor='woff2';font.save(output/(name+('' if weight==400 else '-SemiBold')+'.woff2'))
    (output/(name+'-OFL.txt')).write_bytes((source/(prefix+'-OFL.txt')).read_bytes())
    print(name,(output/(name+'.woff2')).stat().st_size)
