"""Generate original minimal native fixtures from the repository root."""
import struct
from pathlib import Path

w = lambda n: struct.pack("<I", n)
a = lambda ns: w(len(ns)) + b"".join(w(n) for n in ns)

def fixture(with_curve=False, with_controls=False):
    types=[b'CParticleEffect',b'CParticleAttributeList',b'CLayerGraphCompileCache',b'CLayerGraphCompileCache_EntrySlot',b'CLayerGraphCompileCache_LayerSlot',b'CLayerGraphCompileCache_EventSlot',b'CLayerCompileCache',b'CLayerCompileCacheField',b'CLayerCompileCacheEvent',b'CCompilerBlobCache',b'CLayerCompileCacheRenderer',b'CLayerCompileCacheRendererParticleInput',b'CLayerCompileCacheRendererProperty']
    strings=types.copy()
    def st(s):
     s=s.encode() if isinstance(s,str) else s
     if s not in strings:strings.append(s)
     return w(strings.index(s))
    objects=[]
    def obj(t,fields):
     objects.append((types.index(t.encode()),fields));return len(objects)
    # All content is original: one initialized sprite with two-second lifetime.
    obj('CParticleEffect',[(1,w(3)),(3,w(2))])
    obj('CParticleAttributeList',[])
    obj('CLayerGraphCompileCache',[(1,a([4])),(2,a([5])),(3,a([6])),(5,w(0)*4)])
    obj('CLayerGraphCompileCache_EntrySlot',[])
    obj('CLayerGraphCompileCache_LayerSlot',[(0,w(7)),(1,a([0]))])
    obj('CLayerGraphCompileCache_EventSlot',[(1,w(0xffffffff)),(2,a([0]))])
    fields=[('self.lifeRatio',31,[0,0,0,0]),('self.invLife',31,[0x3f000000,0,0,0]),('position',33,[0,0,0,0]),('size',31,[0x40000000,0,0,0]),('enabled',1,[0xffffffff,0,0,0]),('color',34,[0x3f800000]*4)]
    constants=w(len(fields))+b''.join(b''.join(w(n) for n in vals) for _,_,vals in fields)
    obj('CLayerCompileCache',[(0,constants),(1,a([])),(2,a(list(range(8,14)))),(8,a([16])),(10,a([15])),(12,w(14))])
    for i,(name,ty,vals) in enumerate(fields):obj('CLayerCompileCacheField',[(0,st(name)),(1,w(ty)),(2,w({31:4,33:12,1:1,34:16}[ty])),(3,w(4)),(4,w(i))])
    obj('CLayerCompileCacheEvent',[])
    script=w(0)*9
    obj('CCompilerBlobCache',[(2,w(len(script)//4)+script)])
    # Renderer16; streams17..20; properties21..25.
    obj('CLayerCompileCacheRenderer',[(1,a([17,18,19,20])),(2,a([21,22,23,24,25])),(3,st('Popcorn/Library/PopcornFXCore/Materials/Default_Billboard.pkma'))])
    for semantic,index in [(0,2),(1,3),(2,4),(100,5)]:
     fs=[(0,w(semantic)),(1,w(index))]
     if index==5:fs += [(2,st('Color')),(3,w(34))]
     obj('CLayerCompileCacheRendererParticleInput',fs)
    for name,num,text in [('GeometryBillboard',1,None),('Diffuse',1,None),('Diffuse.DiffuseMap',0,'sprite.png'),('Transparent',1,None),('Transparent.Type',2,None)]:
     fs=[(0,st(name)),(1,w(12 if text else 0)),(2,w(num)+w(0)*3)]
     if text:fs.append((3,st(text)))
     obj('CLayerCompileCacheRendererProperty',fs)
    if with_curve:
        for name in [b'CLayerCompileCacheSampler',b'CParticleNodeSamplerData_Curve',b'CCompilerBlobCacheExternal',b'CCompilerBlobCacheFunctionDef',b'CCompilerBlobCacheFunctionArg']:
         types.append(name)
         st(name)
        objects[6][1].append((7,a([26])))
        obj('CLayerCompileCacheSampler',[(0,st('curve')),(1,w(27)),(3,w(3))])
        obj('CParticleNodeSamplerData_Curve',[(16,a([0,0x3f800000])),(17,a([0,0x3f800000]))])
        obj('CCompilerBlobCacheExternal',[(0,st('curve')),(1,st('CurveSampler')),(2,w(0xffffffff)),(3,w(4)),(4,w(0)),(6,w(1))])
        obj('CCompilerBlobCacheFunctionDef',[(0,st('sample')),(1,w(0)),(2,w(0)),(3,a([30]))])
        obj('CCompilerBlobCacheFunctionArg',[(0,w(31)),(1,st('float'))])
        call=b'\x53\x00'+struct.pack('<hHB',-1,0,1)+w(0x20010000)+b'\x00'+w(0x20000000)
        code=call*2
        script=b''.join(w(n) for n in [0,0,32,len(code),0,0,1,0,0])+w(0)*8+code
        script+=b'\x00'*((-len(script))%4)
        objects[14]=(types.index(b'CCompilerBlobCache'),[(2,w(len(script)//4)+script),(3,a([28])),(4,a([29]))])
    if with_controls:
        assert not with_curve
        for name in [b'CLayerCompileCacheAttrib', b'CCompilerBlobCacheExternal',
                     b'CLayerCompileCacheSampler', b'CParticleNodeSamplerData_Curve',
                     b'CCompilerBlobCacheFunctionDef', b'CCompilerBlobCacheFunctionArg']:
            types.append(name)
            st(name)
        # Attribute slots26/27; external declarations28..31; sampler32/33;
        # function34/35; physics36. All content is original.
        obj('CLayerCompileCacheAttrib',[(0,st('__a_Game.EmissionRateMultiplier')),(2,w(31)),(10,w(0x3f800000)+w(0)*3)])
        obj('CLayerCompileCacheAttrib',[(0,st('__a_Custom.SizeMultiplier')),(2,w(31)),(10,w(0x40000000)+w(0)*3)])
        for name in ['__a_Game.EmissionRateMultiplier','__a_Custom.SizeMultiplier','size']:
            obj('CCompilerBlobCacheExternal',[(0,st(name)),(1,st('float')),(2,w(31)),(3,w(4)),(4,w(2)),(6,w(1 if name!='size' else 2))])
        obj('CCompilerBlobCacheExternal',[(0,st('validityCurve')),(1,st('CurveSampler')),(2,w(0xffffffff)),(3,w(4)),(4,w(0)),(6,w(1))])
        obj('CLayerCompileCacheSampler',[(0,st('validityCurve')),(1,w(33)),(3,w(3))])
        obj('CParticleNodeSamplerData_Curve',[(9,w(1)),(16,a([0,0x3f800000])),(17,a([0x3f800000,0x3f800000]))])
        obj('CCompilerBlobCacheFunctionDef',[(0,st('sample')),(1,w(3)),(2,w(0)),(3,a([35]))])
        obj('CCompilerBlobCacheFunctionArg',[(0,w(31)),(1,st('float'))])
        r=lambda slot: 0x20010000+slot
        code=b'\x43'+w(r(0))+struct.pack('<H',0)+b'\x43'+w(r(1))+struct.pack('<H',1)
        code+=b'\x4e\x02'+w(r(2))+w(r(0))+w(r(1))
        code+=b'\x44'+w(r(2))+struct.pack('<H',2)
        code+=b'\x53\x00'+struct.pack('<hHB',3,0,1)+w(r(3))+b'\x00'+w(r(1))
        script=b''.join(w(n) for n in [0,0,0,len(code),0,0,4,0,0])+code
        script+=b'\x00'*((-len(script))%4)
        fs=[(2,w(len(script)//4)+script),(3,a([28,29,30,31])),(4,a([34]))]
        objects[14]=(types.index(b'CCompilerBlobCache'),fs)
        obj('CCompilerBlobCache',[(1,w(3))]+fs)
        fields=objects[6][1]
        fields[:]=[(key,a([15,36]) if key==10 else value) for key,value in fields]
        fields.extend([(3,a([26,27])),(7,a([32]))])
    bodies=b''
    for ty,fs in objects:
     body=b'\x20'+w(ty)+struct.pack('<H',len(fs))+b''.join(struct.pack('<H',f)+v for f,v in fs)
     bodies+=w(len(body))+body
    header=bytes([0x11,0x0b,0,0xca,2,5,1,1])+b''.join(w(n) for n in [0,len(objects),len(types),28+8*len(types)+len(bodies),0])
    blob=header+b''.join(w(strings.index(ty))+w(0) for ty in types)+bodies+w(len(strings))+b''.join(bytes([len(s)])+s for s in strings)
    return blob

if __name__ == "__main__":
    directory = Path("crates/wc3/tests/fixtures/pkb")
    (directory / "sprite.pkb").write_bytes(fixture())
    (directory / "sparse_curve.pkb").write_bytes(fixture(with_curve=True))
    (directory / "controls.pkb").write_bytes(fixture(with_controls=True))
