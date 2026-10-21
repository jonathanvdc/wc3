"""Original three-layer keyed lookup: one writer, two aliased snapshot readers."""
from pathlib import Path
from generate_ribbon import Writer, a, call, f, load, reg, store, w

def fixture(reverse=False):
    b=Writer(); root=b.reserve('CParticleEffect'); attrs=b.obj('CParticleAttributeList',[])
    graph=b.reserve('CLayerGraphCompileCache'); entry=b.obj('CLayerGraphCompileCache_EntrySlot',[])
    layers=[b.reserve('CLayerGraphCompileCache_LayerSlot') for _ in range(3)]
    route=b.reserve('CLayerGraphCompileCache_EventSlot')
    caches=[b.reserve('CLayerCompileCache') for _ in range(3)]
    ctx=(0xffffffff,'pCtxS'); I=(25,'int'); F=(31,'float'); F3=(33,'float3'); SC=(0xffffffff,'SceneCtx')
    C=lambda bank,slot:reg(bank,slot,0)
    for index in range(3):
        writer=index==0; alias=['write','readA','readB'][index]
        definitions=[('self.lifeRatio',31,[0]),('self.invLife',31,[f(50 if writer else 0)]),
            ('position',33,[0,0,0]),('size',31,[f(.01)]),('enabled',1,[0xffffffff]),('color',34,[f(1)]*4)]
        fields=[b.obj('CLayerCompileCacheField',[(0,b.st(name)),(1,w(ty)),
            (2,w({31:4,33:12,1:1,34:16}[ty])),(3,w(4)),(4,w(i))])
            for i,(name,ty,value) in enumerate(definitions)]
        values=w(len(fields))+b''.join(b''.join(w(x) for x in (v+[0]*4)[:4]) for _,_,v in definitions)
        event=b.obj('CLayerCompileCacheEvent',[])
        payload=b.obj('CLayerCompileCacheSpatialLayerPayload',[(0,b.st('Position')),(1,w(33))]+([(2,w(0x1c00))] if writer else []))
        resource=b.obj('CLayerCompileCacheSpatialLayer',[(0,b.st('Original/Keyed')),(1,b.st(alias)),(2,w(1)),(3,w(f(.75))),(4,a([payload]))])
        resource_ext=b.ext(alias,0xffffffff,'SpatialLayer',8)
        S=reg(0,0); P=reg(0x22,1); K=reg(0x1a,1)
        if writer:
            externals=[resource_ext,b.ext('ParticleContextS',0xffffffff,'pCtxS',8)]
            functions=[b.fn('allocatePayload',[ctx],0),b.fn('appendPayload',[I,F3,I,ctx],0),b.fn('insert',[I,F3,ctx],0)]
            constants=[[f(200),0,0],[0],[f(7)]*3]
            code=load(S,1)+call(0,K,[S])+call(1,K,[K,C(0x22,0),C(0x1a,1),S])+call(2,0xffffffff,[K,C(0x22,2),S])
        else:
            scene=b.obj('CCompilerBlobCacheExternal',[(0,b.st('SceneContext')),(1,b.st('SceneCtx')),
                (2,w(0xffffffff)),(3,w(8)),(4,w(2)),(5,w(2)),(6,w(1))])
            externals=[resource_ext,scene,b.ext('position',33,'float3',12)]
            functions=[b.fn('closestF3',[F3,F,I,I,I,I,SC],0)]
            constants=[[f(7)]*3,[f(.0001)],[0]]
            code=load(S,1)+call(0,P,[C(0x22,0),C(0x20,1),C(0x1a,2),C(0x1a,2),C(0x1a,2),C(0x1a,2),S])+store(P,2)
        init=b.blob(code,constants,externals,functions,2)
        physics=b.blob(code,constants,externals,functions,2);b.objects[physics-1][1].insert(0,(1,w(3)))
        renderer=[]
        if not writer:
            inputs=[]
            for semantic,slot in [(0,2),(1,3),(2,4),(100,5)]:
                fs=[(0,w(semantic)),(1,w(slot))]
                if semantic==100:fs.extend([(2,b.st('Color')),(3,w(34))])
                inputs.append(b.obj('CLayerCompileCacheRendererParticleInput',fs))
            properties=[]
            for name,value,text in [('GeometryBillboard',1,None),('Diffuse',1,None),('Diffuse.DiffuseMap',0,'sprite.png'),('Transparent',1,None),('Transparent.Type',2,None)]:
                fs=[(0,b.st(name)),(1,w(12 if text else 0)),(2,w(value)+w(0)*3)]
                if text:fs.append((3,b.st(text)))
                properties.append(b.obj('CLayerCompileCacheRendererProperty',fs))
            renderer=[b.obj('CLayerCompileCacheRenderer',[(1,a(inputs)),(2,a(properties)),(3,b.st('Popcorn/Library/PopcornFXCore/Materials/Default_Billboard.pkma'))])]
        b.set(caches[index],[(0,values),(1,a([])),(2,a(fields)),(6,a([resource])),(8,a(renderer)),(10,a([init,physics])),(12,w(event))])
        b.set(layers[index],[(0,w(caches[index])),(1,a([0]))])
    b.set(root,[(1,w(graph)),(3,w(attrs))])
    b.set(graph,[(1,a([entry])),(2,a(layers)),(3,a([route])),(5,w(0)*4)])
    b.set(route,[(1,w(0xffffffff)),(2,a([1,0,2] if reverse else [0,1,2]))])
    return b.encode()

if __name__=='__main__':
    path=Path(__file__).parent
    (path/'spatial.pkb').write_bytes(fixture())
    (path/'spatial-reordered.pkb').write_bytes(fixture(True))
