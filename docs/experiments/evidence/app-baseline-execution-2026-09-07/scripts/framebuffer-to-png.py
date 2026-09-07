"""Bounded conversion of a preserved Hyper-V RGB565 capture; no guest execution."""
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image

source, target = map(Path, sys.argv[1:3])
width, height = map(int, sys.argv[3:5])
assert 0 < width <= 4096 and 0 < height <= 4096
raw = source.read_bytes()
size = width * height * 2
assert size <= len(raw) <= size + 4, (size, len(raw))
image = Image.frombytes("RGB", (width, height), raw[:size], "raw", "BGR;16")
with target.open("xb") as stream:
    image.save(stream, format="PNG")
print(json.dumps({"raw_bytes": len(raw), "pixel_bytes": size,
                  "unused_trailing_bytes_hex": raw[size:].hex(),
                  "conversion": "Pillow RGB565 to RGB PNG, bounded to requested dimensions",
                  "raw_sha256": hashlib.sha256(raw).hexdigest(),
                  "png_sha256": hashlib.sha256(target.read_bytes()).hexdigest()}))
