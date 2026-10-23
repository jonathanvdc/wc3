"""Original two-layer native baked2.5.1 workload author. No reference code.

Envelope/graph authoring follows this repository's original generate_fixtures.py.
Writes only adjacent original benchmark fixtures; encoding is not execution validation.
"""
from pathlib import Path
import argparse
import hashlib
import json
import struct

D = Path(__file__).resolve().parent
W = lambda value: struct.pack('<I', value & 0xffffffff)
A = lambda values: W(len(values)) + b''.join(W(v) for v in values)
F = lambda value: struct.unpack('<I', struct.pack('<f', value))[0]
TYPE = {1: ('bool', 1, 0x02), 25: ('int', 4, 0x1a), 26: ('int2', 8, 0x1b),
        27: ('int3', 12, 0x1c), 31: ('float', 4, 0x20), 33: ('float3', 12, 0x22),
        34: ('float4', 16, 0x23), 0xffffffff: ('context', 8, 0x00)}


class Document:
    def __init__(self):
        self.types = []
        self.strings = []
        self.objects = []
        self.scripts = []

    def string(self, text):
        value = text.encode()
        if value not in self.strings:
            self.strings.append(value)
        return W(self.strings.index(value))

    def obj(self, name, fields=None):
        if name not in self.types:
            self.types.append(name)
            self.string(name)
        self.objects.append([self.types.index(name), fields or []])
        return len(self.objects)

    def put(self, link, fields):
        self.objects[link - 1][1] = fields

    def external(self, name, native_type, access=1, meta=4, type_name=None):
        return self.obj('CCompilerBlobCacheExternal', [
            (0, self.string(name)), (1, self.string(type_name or TYPE[native_type][0])),
            (2, W(native_type)), (3, W(TYPE[native_type][1])), (4, W(meta)),
            (5, W(2 if native_type == 0xffffffff or name == 'scene.time' else 0)),
            (6, W(access))])

    def function(self, name, receiver, args, traits=44):
        links = [self.obj('CCompilerBlobCacheFunctionArg', [
            (0, W(ty)), (1, self.string(text)), (2, W(attributes))])
                 for ty, text, attributes in args]
        return self.obj('CCompilerBlobCacheFunctionDef', [
            (0, self.string(name)), (1, W(receiver)), (2, W(traits)), (3, A(links))])

    def fields(self, rows):
        links, constants = [], []
        for index, (name, ty, words) in enumerate(rows):
            constants.append(list(words) + [0] * (4 - len(words)))
            links.append(self.obj('CLayerCompileCacheField', [
                (0, self.string(name)), (1, W(ty)), (2, W(TYPE[ty][1])),
                (3, W(4)), (4, W(index))]))
        return links, W(len(constants)) + b''.join(W(word) for row in constants for word in row)

    def script(self, stage, code, constants, externals, functions, slots, names):
        pool = b''.join(b''.join(W(word) for word in (list(row) + [0] * (8 - len(row))))
                        for row in constants)
        raw = b''.join(W(word) for word in [0, 0, len(pool), len(code), 0, 0, slots, 0, 0]) + pool + code
        raw += b'\0' * (-len(raw) % 4)
        entry = self.obj('CCompilerBlobCacheEntryPoint', [
            (0, self.string('main')), (1, W(0)), (2, W(len(code)))])
        link = self.obj('CCompilerBlobCache', [(1, W(stage)), (2, W(len(raw) // 4) + raw),
            (3, A(externals)), (4, A(functions)), (5, W(entry))])
        self.scripts.append({'link': link, 'stage': stage, 'code_bytes': len(code),
                             'code_sha256': hashlib.sha256(code).hexdigest(),
                             'instructions': names, 'count': len(names),
                             'constant_words': constants})
        return link

    def encode(self):
        bodies = b''
        for ty, fields in self.objects:
            body = b'\x20' + W(ty) + struct.pack('<H', len(fields))
            body += b''.join(struct.pack('<H', key) + value for key, value in fields)
            bodies += W(len(body)) + body
        header = bytes([0x11, 0x0b, 0, 0xca, 2, 5, 1, 1]) + b''.join(W(value) for value in
            [0, len(self.objects), len(self.types), 28 + 8 * len(self.types) + len(bodies), 0])
        table = b''.join(self.string(name) + W(0) for name in self.types)
        strings = W(len(self.strings))
        for value in self.strings:
            length = len(value)
            assert length < 32768
            strings += (bytes([length]) if length < 128 else bytes([0x80 | (length >> 8), length & 255])) + value
        return header + table + bodies + strings


def reg(ty, slot, scope=1):
    return TYPE[ty][2] << 24 | scope << 16 | slot


def load(ty, slot, external):
    return b'\x43' + W(reg(ty, slot)) + struct.pack('<H', external)


def store(source, external):
    return b'\x44' + W(source) + struct.pack('<H', external)


def binary(operation, ty, destination, left, right):
    return b'\x4e' + bytes([operation]) + W(reg(ty, destination)) + W(left) + W(right)


def call(receiver, function, result, arguments):
    return b'\x53' + bytes([0 if result == 0xffffffff else 2]) + struct.pack('<hHB', receiver, function, len(arguments)) + W(result) + b''.join(bytes([flags]) + W(source) for flags, source in arguments)


def build(count):
    d = Document()
    root = d.obj('CParticleEffect')
    attributes = d.obj('CParticleAttributeList')
    graph = d.obj('CLayerGraphCompileCache')
    entry = d.obj('CLayerGraphCompileCache_EntrySlot')
    root_slot = d.obj('CLayerGraphCompileCache_LayerSlot')
    child_slot = d.obj('CLayerGraphCompileCache_LayerSlot')
    entry_event = d.obj('CLayerGraphCompileCache_EventSlot')
    child_event = d.obj('CLayerGraphCompileCache_EventSlot')
    root_cache = d.obj('CLayerCompileCache')
    child_cache = d.obj('CLayerCompileCache')
    empty_root_event = d.obj('CLayerCompileCacheEvent')
    child_declaration = d.obj('CLayerCompileCacheEvent', [(0, d.string('__e_Child'))])
    d.put(root, [(1, W(graph)), (3, W(attributes))])
    d.put(graph, [(1, A([entry])), (2, A([root_slot, child_slot])),
                  (3, A([entry_event, child_event])), (5, W(0) * 4)])
    d.put(root_slot, [(0, W(root_cache)), (1, A([0])), (2, A([1]))])
    d.put(child_slot, [(0, W(child_cache)), (1, A([1]))])
    d.put(entry_event, [(1, W(-1)), (2, A([0]))])
    d.put(child_event, [(0, d.string('__e_Child')), (1, W(0)), (2, A([1]))])

    root_fields, root_constants = d.fields([
        ('self.lifeRatio', 31, [0]), ('self.invLife', 31, [0]),
        ('remainingBurst', 31, [F(1)]), ('accumulator', 26, [0, 0])])
    root_externals = [d.external('remainingBurst', 31, 3), d.external('accumulator', 26, 3),
        d.external('ParticleContextS', 0xffffffff, 1, 4, 'pCtxS'),
        d.external('RandContext', 0xffffffff, 1, 4, 'RandCtx'),
        d.external('__e_Child', 0xffffffff, 7, 2, 'particleEvent'),
        d.external('self.lifeRatio', 31, 3), d.external('self.invLife', 31, 3)]
    f = (31, 'float', 4)
    generate = d.function('generate', -1, [(26, 'int2', 4), f, f, f, f, (0xffffffff, 'pCtxS', 4)])
    init_payload = d.function('initPayload', 4, [(25, 'int', 0), (27, 'int3', 4), (0xffffffff, 'pCtxS', 4)])
    kick = d.function('kick', 4, [(25, 'int', 4), (0xffffffff, 'RandCtx', 4), (0xffffffff, 'pCtxS', 4)], 16)
    constants = [[0], [F(1)], [F(count)]]
    root_init_code = b''.join(store(reg(31, slot, 0), external) for slot, external in [(1, 0), (0, 5), (0, 6)])
    root_init = d.script(0, root_init_code, constants, root_externals, [], 0,
                         ['store remainingBurst=1', 'store self.lifeRatio=0', 'store self.invLife=0'])
    root_code = load(31, 0, 0) + load(26, 1, 1) + load(0xffffffff, 2, 2) + load(0xffffffff, 3, 3)
    root_code += binary(2, 31, 4, reg(31, 0), reg(31, 2, 0))
    root_code += call(-1, 0, reg(27, 5), [(24, reg(26, 1)), (26, reg(31, 4)), (10, reg(31, 1, 0)),
         (10, reg(31, 0, 0)), (10, reg(31, 1, 0)), (8, reg(0xffffffff, 2))])
    root_code += call(4, 1, reg(25, 6), [(0, reg(25, 0, 0)), (24, reg(27, 5)), (8, reg(0xffffffff, 2))])
    root_code += call(4, 2, 0xffffffff, [(24, reg(25, 6)), (8, reg(0xffffffff, 3)), (8, reg(0xffffffff, 2))])
    root_code += store(reg(31, 0, 0), 0)
    root_evolve = d.script(3, root_code, constants, root_externals, [generate, init_payload, kick], 7,
        ['load remainingBurst', 'load accumulator', 'load ParticleContextS', 'load RandContext',
         'quantity=remainingBurst*N', 'generate', 'initPayload', 'kick', 'store remainingBurst=0'])
    d.put(root_cache, [(0, root_constants), (1, A([])), (2, A(root_fields)),
        (5, A([child_declaration])), (10, A([root_init, root_evolve])), (12, W(empty_root_event))])

    rows = [('self.lifeRatio', 31, [0]), ('self.invLife', 31, [0]),
        ('basePosition', 33, [0, 0, 0]), ('baseSize', 31, [F(1)]),
        ('baseAlpha', 31, [F(.75)]), ('baseColor', 34, [F(v) for v in [.125, .25, .5, 1]]),
        ('baseRotation', 31, [F(.25)]), ('coefficient', 31, [F(.0625)]),
        ('Position', 33, [0, 0, 0]), ('Size', 31, [F(1)]), ('Rotation', 31, [0]),
        ('Enabled', 1, [0xffffffff]), ('Color', 34, [F(1)] * 4), ('TextureID', 31, [0])]
    child_fields, child_constants = d.fields(rows)
    external_rows = [('scene.time', 31)] + [(name, ty) for name, ty, _ in rows]
    child_externals = [d.external(name, ty, 1 if name == 'scene.time' else 3, 2 if name == 'scene.time' else 4)
                       for name, ty in external_rows]
    child_externals.append(d.external('curve', 0xffffffff, 1, 1, 'samplerCurve4C'))
    curve = d.obj('CParticleNodeSamplerData_Curve', [(9, W(4)), (10, W(0)),
        (16, A([F(v) for v in [0, 1, 2]])),
        (17, A([F(v) for v in [.125,.25,.5,1,.5,.375,.25,.75,.75,.5,.125,.5]])),
        (18, A([F(v) for v in [.25,.125,-.125,-.125] * 6]))])
    sampler = d.obj('CLayerCompileCacheSampler', [(0, d.string('curve')), (1, W(curve)), (3, W(3))])
    sample = d.function('sample', len(child_externals) - 1, [f])
    child_init_constants = [words for _, _, words in rows]
    child_init_code = b''.join(store(reg(ty, i, 0), i + 1) for i, (_, ty, _) in enumerate(rows))
    child_init = d.script(0, child_init_code, child_init_constants, child_externals, [], 0,
                         ['store ' + row[0] for row in rows])
    # All registers are invocation-local scope1; no persistent-state override.
    loads = [(31, 0, 0), (33, 1, 3), (31, 2, 4), (31, 3, 5), (34, 4, 6), (31, 5, 7), (31, 6, 8)]
    code = b''.join(load(*item) for item in loads)
    consts = [[F(.25)], [F(.5)], [F(.125)], [0xffffffff], [0]]
    code += binary(2, 31, 7, reg(31, 0), reg(31, 6))
    code += binary(0, 31, 8, reg(31, 2), reg(31, 7))
    code += binary(2, 31, 9, reg(31, 8), reg(31, 0, 0))
    code += binary(0, 31, 10, reg(31, 5), reg(31, 7))
    code += binary(2, 31, 11, reg(31, 3), reg(31, 1, 0))
    code += binary(0, 31, 12, reg(31, 11), reg(31, 1, 0))
    code += binary(2, 31, 13, reg(31, 9), reg(31, 12))
    code += binary(1, 31, 14, reg(31, 10), reg(31, 2, 0))
    code += call(len(child_externals) - 1, 0, reg(34, 15), [(26, reg(31, 7))])
    code += binary(2, 34, 16, reg(34, 15), reg(34, 4))
    code += b''.join(store(source, external) for source, external in [
        (reg(33, 1), 9), (reg(31, 13), 10), (reg(31, 14), 11),
        (reg(1, 3, 0), 12), (reg(34, 16), 13), (reg(31, 4, 0), 14)])
    names = ['load scene.time', 'load basePosition', 'load baseSize', 'load baseAlpha',
        'load baseColor', 'load baseRotation', 'load coefficient', 'multiply time',
        'add size', 'scale size', 'add rotation', 'scale alpha', 'offset alpha',
        'multiply size', 'offset rotation', 'sample curve', 'multiply color',
        'store Position', 'store Size', 'store Rotation', 'store Enabled',
        'store Color', 'store TextureID']
    child_evolve = d.script(3, code, consts, child_externals, [sample], 17, names)
    renderer = d.obj('CLayerCompileCacheRenderer')
    inputs = []
    for semantic, field_index, name, ty in [(0,8,'',33),(1,9,'',31),(2,11,'',1),
            (100,10,'Rotation',31),(100,12,'Color',34),(100,13,'TextureID',31)]:
        fields = [(0, W(semantic)), (1, W(field_index))]
        if name:
            fields.extend([(2, d.string(name)), (3, W(ty))])
        inputs.append(d.obj('CLayerCompileCacheRendererParticleInput', fields))
    properties = []
    for name, value, text in [('GeometryBillboard',1,None),('Diffuse',1,None),
            ('Diffuse.DiffuseMap',0,'sprite.png'),('Transparent',1,None),('Transparent.Type',2,None)]:
        fields = [(0, d.string(name)), (1, W(12 if text else 0)), (2, W(value) + W(0) * 3)]
        if text:
            fields.append((3, d.string(text)))
        properties.append(d.obj('CLayerCompileCacheRendererProperty', fields))
    d.put(renderer, [(1, A(inputs)), (2, A(properties)),
        (3, d.string('Popcorn/Library/PopcornFXCore/Materials/Default_Billboard.pkma'))])
    d.put(child_cache, [(0, child_constants), (1, A([])), (2, A(child_fields)),
        (7, A([sampler])), (8, A([renderer])), (10, A([child_init, child_evolve])),
        (12, W(empty_root_event))])
    blob = d.encode()
    return blob, {'count': count, 'objects': len(d.objects), 'scripts': d.scripts,
        'child_field_order': [name for name, _, _ in rows],
        'child_external_order': [name for name, _ in external_rows] + ['curve'],
        'sha256': hashlib.sha256(blob).hexdigest(), 'bytes': len(blob)}


def shared_contract(info):
    # Every body, field and external is identical across variants. Only the
    # authored one-shot burst constant and resulting container hash differ.
    common = {key: value for key, value in info.items()
              if key not in ['count', 'sha256', 'bytes', 'path']}
    # Script records can share the same authoring constant-list object. Break
    # that Python alias before substituting the human-readable parameter.
    common = json.loads(json.dumps(common))
    for script in common['scripts'][:2]:
        assert script['constant_words'][2] == [F(info['count'])]
        script['constant_words'][2] = ['f32(burst_count)']
    return common


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--check', action='store_true', help='verify deterministic bytes without writing fixtures')
    args = parser.parse_args()
    manifest = {'provenance': 'original synthetic native workload, no acquired bytes',
                'generator_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                'contract': None, 'fixtures': []}
    for count in [1, 8, 128, 1024]:
        blob, info = build(count)
        path = D / f'burst-{count}.pkb'
        if args.check:
            assert path.read_bytes() == blob, ('fixture differs from original generator', path.name)
        else:
            path.write_bytes(blob)
        common = shared_contract(info)
        if manifest['contract'] is None:
            manifest['contract'] = common
        else:
            assert manifest['contract'] == common, 'variant differs beyond its one burst parameter'
        manifest['fixtures'].append({'burst_count': count, 'path': path.name,
            'bytes': len(blob), 'sha256': hashlib.sha256(blob).hexdigest()})
    text = json.dumps(manifest, indent=2) + '\n'
    if args.check:
        assert (D / 'manifest.json').read_text() == text, 'manifest differs from original generator'
        print('Four original fixtures and shared contract match their deterministic generator')
    else:
        (D / 'manifest.json').write_text(text)
