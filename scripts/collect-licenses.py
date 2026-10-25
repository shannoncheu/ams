"""Collect notices from the locked Windows dependency graph and bundled font."""
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output([
    'cargo', 'metadata', '--locked', '--format-version', '1',
    '--filter-platform', 'x86_64-pc-windows-msvc',
], cwd=root))
selected = {node['id'] for node in metadata['resolve']['nodes']}
out = root / 'dist'
out.mkdir(exist_ok=True)
parts = ['THIRD-PARTY NOTICES\n\nGPUI Music Player retains upstream MIT and Apache-2.0 attribution.\n']
inventory = []
for package in sorted(metadata['packages'], key=lambda item: (item['name'], item['version'])):
    if package['id'] not in selected:
        continue
    directory = Path(package['manifest_path']).parent
    notices = []
    # Workspace packages can inherit a repository-root LICENSE/NOTICE.
    for parent in [directory, *list(directory.parents)[:3]]:
        candidates = sorted(p for p in parent.iterdir() if p.is_file()
                            and p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'NOTICE', 'COPYRIGHT')))
        notices.extend(candidates)
        if candidates:
            break
    if package.get('license_file'):
        candidate = directory / package['license_file']
        if candidate.is_file():
            notices.append(candidate)
    parts.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\nSPDX: {package.get('license') or 'see license file'}\nSource: {package.get('repository') or package.get('source') or 'project source'}\n")
    for notice in sorted(set(notices)):
        parts.append(f'\n--- {notice.name} ---\n' + notice.read_text(encoding='utf-8', errors='replace'))
    inventory.append({'name': package['name'], 'version': package['version'], 'license': package.get('license'), 'notices': [p.name for p in sorted(set(notices))]})
parts.append('\n\nNoto Sans SC variable font\n' + (root / 'assets/fonts/OFL.txt').read_text(encoding='utf-8'))
(out / 'THIRD_PARTY_NOTICES.txt').write_text('\n'.join(parts), encoding='utf-8')
(out / 'license-inventory.json').write_text(json.dumps(inventory, indent=2), encoding='utf-8')
print(f'Collected attribution for {len(inventory)} packages and the bundled OFL font')
