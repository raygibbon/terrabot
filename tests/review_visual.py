#!/usr/bin/env python3
"""Build a contact sheet from saved terminal-cell references; requires Pillow."""
from pathlib import Path
import sys
from PIL import Image, ImageDraw
source=Path(sys.argv[1]);output=Path(sys.argv[2])
names=[name for name in ['Idle','Walk','Run','Curious','Bored','Jump','Sit','Sleep','Wake','Think','Talk','Happy','Celebrate','Play'] if (source/f'{name}-0.cells').exists()];times=([0,520,1200,2000,3200,4300] if (source/'Idle-4300.cells').exists() else [0,240,520,800,1200,1800])
sheet=Image.new('RGB',(6*160,len(names)*116),'#0d1420');draw=ImageDraw.Draw(sheet)
for row,name in enumerate(names):
    for col,ms in enumerate(times):
        lines=(source/f'{name}-{ms}.cells').read_text().splitlines()
        palette=dict(item.split('=') for item in lines[1].split())
        image=Image.new('RGB',(80,50));image.putdata([tuple(bytes.fromhex(palette[c])) for line in lines[2:] for c in line])
        sheet.paste(image.resize((160,100),Image.Resampling.NEAREST),(col*160,row*116+16))
        draw.text((col*160,row*116),f'{name} {ms}ms',fill='white')
sheet.save(output)
