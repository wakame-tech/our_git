from pathlib import Path
import zlib
import sys

path = Path(sys.argv[1]).read_bytes()
bin = zlib.decompress(path)
Path(sys.argv[2]).write_bytes(bin)
