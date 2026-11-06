import os
import sys
import tomllib
from pathlib import Path

with Path('Cargo.toml').open('rb') as manifest:
    version = tomllib.load(manifest)['package']['version']
tag = os.environ['GITHUB_REF_NAME']
if tag != f'v{version}':
    print(f'::error::Tag {tag} does not match Cargo.toml version {version}.')
    sys.exit(1)
print(f'Tag {tag} matches Cargo.toml version {version}.')
