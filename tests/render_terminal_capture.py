#!/usr/bin/env python3
"""Render an actual asciicast terminal buffer (optional dev deps: pyte, Pillow)."""
import json
import sys
from pathlib import Path
import pyte
from PIL import Image, ImageDraw, ImageFont

def screenshot(output, columns, rows, path):
    screen = pyte.Screen(columns, rows)
    pyte.Stream(screen).feed(output)
    image = Image.new('RGB', (columns*8, rows*16))
    draw = ImageDraw.Draw(image)
    font = ImageFont.load_default(size=12)
    named = {'default': '0d1420', 'black': '000000', 'white': 'ffffff',
             'red': 'ff0000', 'green': '00ff00', 'blue': '0000ff',
             'brown': 'aaaa00', 'magenta': 'ff00ff', 'cyan': '00ffff'}
    def color(value):
        value = named.get(value, value)
        return '#' + value if len(value) == 6 else '#cfddea'
    for y in range(rows):
        for x in range(columns):
            cell = screen.buffer[y][x]
            fg, bg = color(cell.fg), color(cell.bg)
            if cell.reverse:
                fg, bg = bg, fg
            draw.rectangle((x*8,y*16,x*8+7,y*16+15),fill=bg)
            if cell.data == '▀':
                draw.rectangle((x*8,y*16,x*8+7,y*16+7),fill=fg)
            elif cell.data == '▄':
                draw.rectangle((x*8,y*16+8,x*8+7,y*16+15),fill=fg)
            elif cell.data != ' ':
                draw.text((x*8,y*16),cell.data,font=font,fill=fg)
    image.save(path)

if __name__ == '__main__':
    capture, seconds, output_path = sys.argv[1:]
    lines=Path(capture).read_text().splitlines()
    header=json.loads(lines[0]); chunks=[]
    for line in lines[1:]:
        stamp, kind, value=json.loads(line)
        if stamp > float(seconds): break
        if kind=='o': chunks.append(value)
    screenshot(''.join(chunks),header['width'],header['height'],output_path)
