"""Original native unknown-stage copy of a known unselected alternative."""
from pathlib import Path
from generate_ribbon import Writer, a, call, f, load, reg, store, w

def fixture():
    b = Writer()
    root = b.reserve('CParticleEffect')
    attrs = b.obj('CParticleAttributeList', [])
    graph = b.reserve('CLayerGraphCompileCache')
    entry = b.obj('CLayerGraphCompileCache_EntrySlot', [])
    layer = b.reserve('CLayerGraphCompileCache_LayerSlot')
    route = b.reserve('CLayerGraphCompileCache_EventSlot')
    cache = b.reserve('CLayerCompileCache')
    defs = [('self.lifeRatio',31,[0]),('self.invLife',31,[f(.1)]),
            ('position',33,[0,0,0]),('size',31,[f(2)]),
            ('enabled',1,[0xffffffff]),('color',34,[f(1)]*4)]
    fields = [b.obj('CLayerCompileCacheField', [(0,b.st(name)),(1,w(ty)),
        (2,w({31:4,33:12,1:1,34:16}[ty])),(3,w(4)),(4,w(i))])
        for i,(name,ty,values) in enumerate(defs)]
    values = w(len(defs)) + b''.join(b''.join(w(x) for x in (v+[0]*4)[:4]) for _,_,v in defs)
    event = b.obj('CLayerCompileCacheEvent', [])
    scene_reg, point, particle_reg = reg(0,0), reg(0x22,1), reg(0,2)
    code = load(scene_reg,0)+load(particle_reg,1)
    code += call(0,point,[reg(0x1a,0,0),scene_reg])
    code += call(1,point,[point,reg(0x1a,1,0),particle_reg])+store(point,2)
    # Known init/fixed/varying have different visible counter increments.
    # The unknown record is an exact copy of varying with fresh native links.
    def script(stage, increment):
        scene = b.obj('CCompilerBlobCacheExternal', [(0,b.st('SceneContext')),
            (1,b.st('SceneCtx')),(2,w(0xffffffff)),(3,w(8)),(4,w(2)),(5,w(2)),(6,w(1))])
        particle = b.ext('ParticleContextS',0xffffffff,'pCtxS',8)
        position = b.ext('position',33,'float3',12)
        size = b.ext('size',31,'float',4)
        functions = [b.fn('view.position',[(25,'int'),(0xffffffff,'SceneCtx')]),
                     b.fn('xform_w2l_f_masked',[(33,'float3'),(25,'int'),(0xffffffff,'pCtxS')])]
        value = reg(0x20,3)
        add = b'\x4e\x00'+w(value)+w(value)+w(reg(0x20,2,0))
        body = code + load(value,3) + add + store(value,3)
        entry=b.obj('CCompilerBlobCacheEntryPoint',[(0,b.st('Root')),(1,w(0)),(2,w(len(body)))])
        blob=b.blob(body,[[0],[67],[f(increment)]],[scene,particle,position,size],functions,4)
        b.objects[blob-1][1].extend([(1,w(stage)),(5,w(entry))])
        return blob
    init=script(0,1);fixed=script(4,2);varying=script(5,3);extra=script(97,3)
    inputs = []
    for semantic,index in [(0,2),(1,3),(2,4),(100,5)]:
        fs = [(0,w(semantic)),(1,w(index))]
        if semantic == 100: fs += [(2,b.st('Color')),(3,w(34))]
        inputs.append(b.obj('CLayerCompileCacheRendererParticleInput', fs))
    props = []
    for name,value,text in [('GeometryBillboard',1,None),('Diffuse',1,None),
            ('Diffuse.DiffuseMap',0,'sprite.png'),('Transparent',1,None),('Transparent.Type',2,None)]:
        fs=[(0,b.st(name)),(1,w(12 if text else 0)),(2,w(value)+w(0)*3)]
        if text: fs.append((3,b.st(text)))
        props.append(b.obj('CLayerCompileCacheRendererProperty',fs))
    renderer=b.obj('CLayerCompileCacheRenderer',[(1,a(inputs)),(2,a(props)),
        (3,b.st('Popcorn/Library/PopcornFXCore/Materials/Default_Billboard.pkma'))])
    b.set(cache,[(0,values),(1,a([])),(2,a(fields)),(8,a([renderer])),(9,w(varying)),(10,a([init,extra,fixed])),(12,w(event))])
    b.set(root,[(1,w(graph)),(3,w(attrs))])
    b.set(graph,[(1,a([entry])),(2,a([layer])),(3,a([route])),(5,w(0)*4)])
    b.set(layer,[(0,w(cache)),(1,a([0]))])
    b.set(route,[(1,w(0xffffffff)),(2,a([0]))])
    return b.encode()

if __name__ == '__main__':
    (Path(__file__).parent/'redundant-stage.pkb').write_bytes(fixture())
