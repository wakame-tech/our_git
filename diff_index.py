from pathlib import Path
from subprocess import run
import sys

git_index = Path('.git/index')
our_git_index = Path('.git/our_index')

res = run(f'cargo run -- add -i {our_git_index} {sys.argv[1]}', shell=True, capture_output=True)
print(res.stdout)

res = run(f'diff -u <(hexyl --color never {git_index}) <(hexyl --color never {our_git_index})', shell=True, capture_output=True)
print(res.stdout)
