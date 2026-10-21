"""Original native graph: a controller emits three payload-position ribbon points."""
from pathlib import Path
import json,struct,zlib
ROOT=Path(__file__).resolve().parent
w=lambda x:struct.pack('<I',x&0xffffffff)
a=lambda v:w(len(v))+b''.join(w(x) for x in v)
f=lambda x:struct.unpack('<I',struct.pack('<f',x))[0]
class Writer:
 def __init__(self):self.strings=[];self.types=[];self.objects=[]
 def st(self,text):
  text=text.encode() if isinstance(text,str) else text
  if text not in self.strings:self.strings.append(text)
  return w(self.strings.index(text))
 def reserve(self,ty):
  ty=self.st(ty)
  if ty not in self.types:self.types.append(ty)
  self.objects.append([self.types.index(ty),[]]);return len(self.objects)
 def set(self,i,fs):self.objects[i-1][1]=fs
 def obj(self,ty,fs):i=self.reserve(ty);self.set(i,fs);return i
 def encode(self):
  bodies=b''
  for ty,fs in self.objects:
   body=b'\x20'+w(ty)+struct.pack('<H',len(fs))+b''.join(struct.pack('<H',i)+v for i,v in fs);bodies+=w(len(body))+body
  header=bytes([0x11,0x0b,0,0xca,2,5,1,1])+b''.join(w(x) for x in [0,len(self.objects),len(self.types),28+8*len(self.types)+len(bodies),0])
  assert all(len(x)<128 for x in self.strings)
  return header+b''.join(x+w(0) for x in self.types)+bodies+w(len(self.strings))+b''.join(bytes([len(s)])+s for s in self.strings)
 def arg(self,ty,name):return self.obj('CCompilerBlobCacheFunctionArg',[(0,w(ty)),(1,self.st(name))])
 def fn(self,name,args,bound=0xffffffff):
  links=[self.arg(t,n) for t,n in args]
  return self.obj('CCompilerBlobCacheFunctionDef',[(0,self.st(name)),(1,w(bound)),(2,w(0)),(3,a(links))])
 def ext(self,name,ty,typename,size):return self.obj('CCompilerBlobCacheExternal',[(0,self.st(name)),(1,self.st(typename)),(2,w(ty)),(3,w(size)),(4,w(0)),(6,w(1))])
 def blob(self,code,constants,externals,functions,slots):
  constants=b''.join(b''.join(w(x) for x in (c+[0]*8)[:8]) for c in constants)
  body=b''.join(w(x) for x in [0,0,len(constants),len(code),0,0,slots,0,0])+constants+code
  body+=b'\0'*((-len(body))%4)
  return self.obj('CCompilerBlobCache',[(2,w(len(body)//4)+body),(3,a(externals)),(4,a(functions))])
def reg(bank,slot,scope=1):return (bank<<24)|(scope<<16)|slot
def load(dst,ext):return b'\x43'+w(dst)+struct.pack('<H',ext)
def store(src,ext):return b'\x44'+w(src)+struct.pack('<H',ext)
def call(fn,dst,args):return b'\x53\x00'+struct.pack('<hHB',-1,fn,len(args))+w(dst)+b''.join(b'\x00'+w(x) for x in args)
def native(p):return [p[0],-p[2],p[1]]
def fixture(mode=0,image='quadrants.png',groups=1):
 b=Writer();root=b.reserve('CParticleEffect');attrs=b.obj('CParticleAttributeList',[]);graph=b.reserve('CLayerGraphCompileCache');entry=b.obj('CLayerGraphCompileCache_EntrySlot',[])
 layers=[b.reserve('CLayerGraphCompileCache_LayerSlot') for _ in range(2)]
 routes=[b.reserve('CLayerGraphCompileCache_EventSlot') for _ in range(2)]
 caches=[b.reserve('CLayerCompileCache') for _ in range(2)]
 payload=b.obj('CLayerCompileCacheEventPayload',[(0,b.st('Position')),(1,w(33)),(2,w(12)),(4,w(1))])
 outgoing=b.obj('CLayerCompileCacheEvent',[(0,b.st('Spawn')),(2,a([payload])),(3,w(12))])
 controller_root=b.obj('CLayerCompileCacheEvent',[])
 child_root=b.obj('CLayerCompileCacheEvent',[(2,a([payload])),(3,w(12))])
 def fields(defs):
  links=[]
  for i,(name,ty,value) in enumerate(defs):links.append(b.obj('CLayerCompileCacheField',[(0,b.st(name)),(1,w(ty)),(2,w({31:4,33:12,34:16,1:1}[ty])),(3,w(4)),(4,w(i))]))
  values=w(len(defs))+b''.join(b''.join(w(x) for x in (vals+[0]*4)[:4]) for _,_,vals in defs)
  return values,links
 cvalues,cfields=fields([('self.lifeRatio',31,[0]),('self.invLife',31,[0])])
 defs=[('self.lifeRatio',31,[0]),('self.invLife',31,[f(.2)]),('position',33,[0,0,0]),('size',31,[f(.25)]),('enabled',1,[0xffffffff]),('color',34,[f(1)]*4),('axis0',33,[f(v) for v in native((0,0,1))])]
 pvalues,pfields=fields(defs)
 ctx=(0xffffffff,'pCtxS');rand=(0xffffffff,'RandCtx');I=(25,'int');I3=(27,'int3');F=(31,'float');F3=(33,'float3')
 externals=[b.ext('ParticleContextS',0xffffffff,'pCtxS',8),b.ext('RandContext',0xffffffff,'RandCtx',8),b.ext('Spawn',0xffffffff,'Event',8)]
 functions=[b.fn('trigger',[(1,'bool'),F,ctx]),b.fn('buildPayloadElement',[I3,F3,F3,I,ctx]),b.fn('initPayload',[I,I3,ctx],2),b.fn('appendPayload',[I,I,I,ctx],2),b.fn('kick',[I,rand,ctx],2)]
 C=lambda bank,slot:reg(bank,slot,0)
 S=reg(0,0);R=reg(0,1);B=reg(0x1c,2);E=reg(0x1a,3);D=reg(0x1a,4)
 constants=[[0xffffffff],[f(1)],[0],[0x100]]+[[f(v) for v in native(p)] for p in [(-2,0,0),(0,0,0),(0,2,0)]]
 code=load(S,0)+load(R,1)
 for i in range(3):
  code+=call(0,B,[C(2,0),C(0x20,1),S])+call(1,E,[B,C(0x22,4+i),C(0x22,4+i),C(0x1a,3),S])+call(2,D,[C(0x1a,2),B,S])+call(3,D,[D,C(0x1a,2),E,S])+call(4,0xffffffff,[D,R,S])
 init=b.blob(code,constants,externals,functions,5)
 child_ext=[b.ext('ParticleContextS',0xffffffff,'pCtxS',8),b.ext('position',33,'float3',12)]
 child_functions=[b.fn('xform_l2w_f_masked',[F3,I,ctx])]
 code=load(S,0)+call(0,reg(0x22,1),[C(0x22,0),C(0x1a,1),S])+store(reg(0x22,1),1)
 child_init=b.blob(code,[[0,0,0],[17]],child_ext,child_functions,2)
 streams=[]
 for semantic,index,name,ty in [(0,2,None,33),(1,3,None,31),(2,4,None,1),(100,5,'Color',34),(4,6,None,33)]:
  fs=[(0,w(semantic)),(1,w(index))]
  if name:fs +=[(2,b.st(name)),(3,w(ty))]
  streams.append(b.obj('CLayerCompileCacheRendererParticleInput',fs))
 props=[]
 for name,num,text,ty in [('GeometryRibbon',1,None,0),('Diffuse',1,None,0),('EnableRendering',1,None,0),('Diffuse.DiffuseMap',0,image,12),('Transparent',1,None,0),('Transparent.Type',2,None,3),('BillboardingMode',mode,None,3)]:
  fs=[(0,b.st(name)),(1,w(ty)),(2,w(num)+w(0)*3)]
  if text:fs.append((3,b.st(text)))
  props.append(b.obj('CLayerCompileCacheRendererProperty',fs))
 renderer=b.obj('CLayerCompileCacheRenderer',[(0,w(1)),(1,a(streams)),(2,a(props)),(3,b.st('Popcorn/Library/PopcornFXCore/Materials/Default_Ribbon.pkma'))])
 b.set(caches[0],[(0,cvalues),(1,a([])),(2,a(cfields)),(5,a([outgoing])),(10,a([init])),(12,w(controller_root))])
 b.set(caches[1],[(0,pvalues),(1,a([])),(2,a(pfields)),(8,a([renderer])),(10,a([child_init])),(12,w(child_root))])
 b.set(root,[(1,w(graph)),(3,w(attrs))]);b.set(graph,[(1,a([entry])),(2,a(layers)),(3,a(routes)),(5,b''.join(w(x) for x in [1,5,3,1]))])
 b.set(layers[0],[(0,w(caches[0])),(1,a([0])),(2,a([1]))]);b.set(layers[1],[(0,w(caches[1])),(1,a([1]))])
 b.set(routes[0],[(1,w(0xffffffff)),(2,a([0]*groups))]);b.set(routes[1],[(0,b.st('Spawn')),(1,w(0)),(2,a([1]))])
 return b.encode()

if __name__ == "__main__":
 (ROOT/"ribbon.pkb").write_bytes(fixture(image="sprite.png"))
