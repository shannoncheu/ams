"""Collect locked Windows notices and redistribute unchanged MPL-covered sources."""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import re
import subprocess
from urllib.error import HTTPError, URLError
from urllib.request import urlopen
import zipfile

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output([
    'cargo', 'metadata', '--locked', '--format-version', '1',
    '--filter-platform', 'x86_64-pc-windows-msvc',
], cwd=root))
selected = {node['id'] for node in metadata['resolve']['nodes']}
packages = sorted((p for p in metadata['packages'] if p['id'] in selected), key=lambda p: (p['name'], p['version']))
out = root / 'dist'
out.mkdir(exist_ok=True)
supplement = out / 'license-supplement'
supplement.mkdir(exist_ok=True)

def local_notices(package):
    directory = Path(package['manifest_path']).parent
    notices = []
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
    return [(p.name, p.read_text(encoding='utf-8', errors='replace'), None) for p in sorted(set(notices))]

def remote_key(package):
    repo = re.fullmatch(r'https://github.com/([\w.-]+)/([\w.-]+?)(?:\.git)?/?', package.get('repository') or '')
    vcs_path = Path(package['manifest_path']).parent / '.cargo_vcs_info.json'
    if not repo or not vcs_path.is_file():
        return None
    vcs = json.loads(vcs_path.read_text(encoding='utf-8'))
    sha = vcs.get('git', {}).get('sha1', '')
    if not re.fullmatch(r'[a-f0-9]{40}', sha):
        return None
    return repo.group(1), repo.group(2), sha

def fetch_notices(key):
    owner, repo, sha = key
    found = []
    # Some published crates omit workspace-root license files. Retrieve the exact
    # published VCS commit, never an unpinned default branch or another license.
    for name in ['LICENSE', 'LICENSE-MIT', 'LICENSE-APACHE', 'LICENSE.md', 'LICENSE.txt', 'COPYING', 'COPYRIGHT', 'LICENCE', 'LICENSE-APACHE.txt', 'LICENSE-MIT.txt']:
        url = f'https://raw.githubusercontent.com/{owner}/{repo}/{sha}/{name}'
        cache = supplement / (hashlib.sha256(url.encode()).hexdigest() + '.txt')
        try:
            if cache.is_file():
                content = cache.read_text(encoding='utf-8')
            else:
                with urlopen(url, timeout=20) as response:
                    content = response.read().decode('utf-8', errors='replace')
                cache.write_text(content, encoding='utf-8')
            found.append((name, content, url))
        except (HTTPError, URLError, TimeoutError):
            continue
    return key, found

notices = {p['id']: local_notices(p) for p in packages}
keys = {remote_key(p) for p in packages if not notices[p['id']]}
keys.discard(None)
with ThreadPoolExecutor(max_workers=8) as executor:
    remote = dict(executor.map(fetch_notices, sorted(keys)))
for p in packages:
    if not notices[p['id']]:
        notices[p['id']] = remote.get(remote_key(p), [])
clarifications = {
    ('dwrote', '0.11.5'): 'MPL-2.0',
    ('pathfinder_geometry', '0.5.1'): 'Apache-2.0',
    ('seahash', '4.1.0'): 'MIT-TERMS',
    ('simd_helpers', '0.1.0'): 'MIT-TERMS',
    ('zune-inflate', '0.2.54'): 'Apache-2.0',
}
clarified = set()
for package in packages:
    choice = clarifications.get((package['name'], package['version']))
    if not notices[package['id']] and choice:
        content = (root / 'licenses' / (choice + '.txt')).read_text(encoding='utf-8')
        evidence = (root / 'licenses/CLARIFICATIONS.md').read_text(encoding='utf-8')
        notices[package['id']] = [('DECLARED-LICENSE-' + choice, content, None), ('CLARIFICATIONS.md', evidence, None)]
        clarified.add(package['id'])
missing = [f"{p['name']} {p['version']}" for p in packages if not notices[p['id']]]
if missing:
    raise SystemExit('Missing license text; do not publish: ' + ', '.join(missing))

parts = ['THIRD-PARTY NOTICES\n\nGPUI Music Player retains upstream MIT and Apache-2.0 attribution.\nDual Apache-2.0/GPL packages are used under Apache-2.0. MPL-covered crates are unchanged; corresponding source is included in THIRD_PARTY_SOURCES.zip.\n']
inventory = []
with zipfile.ZipFile(out / 'THIRD_PARTY_SOURCES.zip', 'w', zipfile.ZIP_DEFLATED, strict_timestamps=False) as sources:
    source_index = []
    for package in packages:
        items = notices[package['id']]
        directory = Path(package['manifest_path']).parent
        parts.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\nSPDX: {package.get('license') or 'see license file'}\nAuthors: {', '.join(package.get('authors', []))}\nSource: {package.get('repository') or package.get('source') or 'project source'}\n")
        for name, content, url in items:
            parts.append(f'\n--- {name} ---\n' + (f'Exact source: {url}\n' if url else '') + content)
        inventory.append({'name': package['name'], 'version': package['version'], 'license': package.get('license'), 'notices': [n for n, _, _ in items]})
        if 'MPL-2.0' in (package.get('license') or '') or package['id'] in clarified:
            prefix = f"{package['name']}-{package['version']}"
            for path in sorted(directory.rglob('*')):
                if path.is_file():
                    sources.write(path, prefix + '/' + path.relative_to(directory).as_posix())
            for name, content, url in items:
                sources.writestr(prefix + '/UPSTREAM-' + name, content)
            source_index.append({'name': package['name'], 'version': package['version'], 'repository': package.get('repository'), 'modified': False})
    sources.writestr('SOURCES.json', json.dumps(source_index, indent=2))
    sources.writestr('README.txt', 'Unmodified published source of MPL-2.0 covered dependencies and explicitly clarified packages. Rebuild instructions and dependencies are in each package Cargo.toml. The application Cargo.lock pins their versions. These licenses apply to their covered files, not to all application code.\n')
parts.append('\n\nNoto Sans SC variable font\n' + (root / 'assets/fonts/OFL.txt').read_text(encoding='utf-8'))
(out / 'THIRD_PARTY_NOTICES.txt').write_text('\n'.join(parts), encoding='utf-8')
(out / 'license-inventory.json').write_text(json.dumps(inventory, indent=2), encoding='utf-8')
print(f'Collected complete notice files for {len(inventory)} packages, bundled OFL font and unchanged MPL sources')
