#!/usr/bin/env python3
"""Prepare a synchronized workspace release; preview unless --execute is given."""

from argparse import ArgumentParser
from dataclasses import dataclass
from difflib import unified_diff
from json import loads
from pathlib import Path
from re import escape, fullmatch, sub, subn
from shlex import join
from subprocess import CalledProcessError, run
from sys import executable


ROOT = Path(__file__).resolve().parents[1]


def command(*args, capture=False):
    print(f"+ {join(str(arg) for arg in args)}", flush=True)
    return run(args, cwd=ROOT, check=True, text=True, capture_output=capture)


def release_manifest(source, version):
    """Change only the shared package version and local dependency requirements."""
    section = ''
    local_dependencies = set()
    changed_dependencies = set()
    changed_package = False
    lines = []
    for line in source.splitlines(keepends=True):
        stripped = line.strip()
        if stripped.startswith('['):
            section = stripped
        if section == '[workspace.package]' and fullmatch(r'version\s*=.*', stripped):
            line = sub(r'"[^"]*"', f'"{version}"', line, count=1)
            changed_package = True
        elif section == '[workspace.dependencies]':
            name = line.partition('=')[0].strip()
            if '=' in line and 'path' in line:
                local_dependencies.add(name)
                line, count = subn(
                    r'(\bversion\s*=\s*)"[^"]*"',
                    lambda match: f'{match[1]}"{version}"', line, count=1,
                )
                if count:
                    changed_dependencies.add(name)
        lines.append(line)
    if not changed_package or changed_dependencies != local_dependencies:
        raise ValueError('Expected a shared version and inline local dependencies with versions')
    return ''.join(lines)



@dataclass(frozen=True)
class FileChange:
    path: Path
    before: str
    after: str

    def preview(self, label):
        name = self.path.name
        diff = ''.join(unified_diff(
            self.before.splitlines(keepends=True), self.after.splitlines(keepends=True),
            fromfile=f'{name} (current)', tofile=f'{name} ({label})',
        ))
        print(diff or f'{name} already has the requested versions.')


@dataclass(frozen=True)
class ReleaseStep:
    args: tuple
    requires_staged_changes: bool = False


@dataclass(frozen=True)
class ReleasePlan:
    version: str
    manifest: FileChange
    lockfile: FileChange
    steps: tuple

    @property
    def tag(self):
        return f'v{self.version}'


def parse_version(value):
    number = r'(?:0|[1-9][0-9]*)'
    if not fullmatch(rf'{number}\.{number}\.{number}(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?', value):
        raise ValueError('Expected X.Y.Z or X.Y.Z-prerelease, without build metadata')
    if '-' in value:
        identifiers = value.split('-', 1)[1].split('.')
        if any(part.isdigit() and len(part) > 1 and part.startswith('0') for part in identifiers):
            raise ValueError('Numeric prerelease identifiers cannot have leading zeros')
    return value


def build_plan(version):
    path = ROOT / 'Cargo.toml'
    source = path.read_text()
    manifest = FileChange(path, source, release_manifest(source, version))
    metadata = loads(command(
        'cargo', 'metadata', '--offline', '--no-deps', '--format-version', '1',
        capture=True,
    ).stdout)
    members = set(metadata['workspace_members'])
    names = [
        escape(package['name']) for package in metadata['packages']
        if package['id'] in members
    ]
    lock_path = ROOT / 'Cargo.lock'
    lock_source = lock_path.read_text()
    lock_updated = sub(
        rf'(\[\[package\]\]\nname = "(?:{"|".join(names)})"\nversion = )"[^"]*"',
        lambda match: f'{match[1]}"{version}"', lock_source,
    )
    helper = Path.home() / '.codex/skills/coarse-commit-series/scripts/commit_with_date.py'
    commit = (executable, str(helper), '--') if helper.is_file() else ('git', 'commit')
    message = f'Release {version}'
    steps = (
        ReleaseStep(('cargo', 'check', '--workspace')),
        ReleaseStep(('cargo', 'publish', '--workspace', '--dry-run', '--locked')),
        ReleaseStep(('git', 'diff', '--check')),
        ReleaseStep(('git', 'add', '--', 'Cargo.toml', 'Cargo.lock')),
        ReleaseStep((*commit, '-m', message), requires_staged_changes=True),
        ReleaseStep(('git', 'tag', '-a', f'v{version}', '-m', message)),
    )
    return ReleasePlan(version, manifest, FileChange(lock_path, lock_source, lock_updated), steps)


def release_blockers(plan):
    blockers = []
    status = command('git', 'status', '--porcelain', '--untracked-files=all', capture=True).stdout
    if status:
        blockers.append(f'Commit or stash existing changes:\n{status.rstrip()}')
    if command('git', 'tag', '--list', plan.tag, capture=True).stdout.strip():
        blockers.append(f'Tag {plan.tag} already exists')
    branch = run(
        ['git', 'symbolic-ref', '--quiet', '--short', 'HEAD'],
        cwd=ROOT, text=True, capture_output=True,
    )
    if branch.returncode == 1:
        blockers.append('HEAD is detached')
    else:
        branch.check_returncode()
        print(f'Release branch: {branch.stdout.strip()}')
    return blockers


def preview(plan, blockers):
    plan.manifest.preview('proposed')
    plan.lockfile.preview('expected')
    print('Cargo will resolve the actual lockfile during execution.')
    for blocker in blockers:
        print(f'Execution blocked: {blocker}')
    print('Planned commands:')
    for step in plan.steps:
        print(f'  {join(step.args)}')
    print('The commit is skipped if no files are staged.')
    print('Preview only: no files, commits, or tags changed. Add --execute to prepare the release.')


def execute(plan):
    plan.manifest.path.write_text(plan.manifest.after)
    # Keep failed preparation changes available for inspection.
    for step in plan.steps:
        if step.requires_staged_changes:
            staged = command('git', 'diff', '--cached', '--name-only', capture=True).stdout
            if not staged.strip():
                continue
        command(*step.args)
    print(f'Created local tag {plan.tag}. Push the release commit and tag when ready.')


def main():
    parser = ArgumentParser(description=__doc__)
    parser.add_argument('version', type=parse_version, help='Release version, such as 0.3.0 or 0.3.0-alpha.1')
    parser.add_argument('--execute', action='store_true', help='Update, validate, commit, and tag')
    args = parser.parse_args()
    plan = build_plan(args.version)
    blockers = release_blockers(plan)
    print(f'Workspace package and local dependency versions: {plan.version}')
    print('Nothing will be pushed or published.')
    if not args.execute:
        preview(plan, blockers)
    elif blockers:
        parser.error('\n'.join(blockers))
    else:
        execute(plan)


if __name__ == '__main__':
    try:
        main()
    except (CalledProcessError, ValueError, OSError) as error:
        raise SystemExit(f'Release stopped: {error}. Inspect git status before retrying.')
