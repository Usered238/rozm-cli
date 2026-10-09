"""Development-only original PE32 procedure execution; never used by the CLI."""
from pathlib import Path
import hashlib
import struct, sys, pefile
from unicorn import Uc,UC_ARCH_X86,UC_MODE_32,UC_HOOK_CODE
from unicorn.x86_const import *
class Oracle:
 def __init__(self):
  if hashlib.sha256(Path('orig/Rozm.exe').read_bytes()).hexdigest() != '90e26b39a4ca518df3b1ebcf4bbaf90a8543ed90a31273241d580d22b39e1a54':
   raise RuntimeError('Unexpected original Rozm.exe version')
  self.u=Uc(UC_ARCH_X86,UC_MODE_32); u=self.u
  u.mem_map(0,0x1000); u.mem_map(0x400000,0x2000000); u.mem_map(0x3000000,0x4000000); u.mem_map(0x8000000,0x100000)
  p=pefile.PE('orig/Rozm.exe'); u.mem_write(0x400000,p.get_memory_mapped_image()); self.heap=0x3000000
  pth=Path('orig'); d=pth.joinpath('BSNbn').read_bytes(); u.mem_write(0x7ee2f4,d); starts=[0]+[i+1 for i,b in enumerate(d) if 37<b<81]; self.put(0x1b00ff8,len(d))
  for i,v in enumerate(starts[1:],2): self.put(0x1b00ff8+4*i,v)
  self.put(0x1b00ffc,0); self.put(0x4ec204,len(starts)-1)
  for addr,name in [(0x67c860,'skfs/skf'),(0x770aa4,'skfs/wif'),(0x7d2528,'skfs/wlf')]:u.mem_write(addr,pth.joinpath(name).read_bytes())
  self.put(0x4ec200,self.get(0x770aa4)); 
  for addr in [0x404dfc,0x40494c,0x404970,0x4049a0,0x4049e4,0x404b34,0x404bb0,0x404c0c,0x404c14,0x404c58,0x404ccc,0x404d58,0x404e64,0x404e6c,0x404eac,0x404f50,0x4036b8,0x408ea8,0x407684,0x4674f2]:u.hook_add(UC_HOOK_CODE,self.hook,begin=addr,end=addr)
 def get(self,a):return struct.unpack('<I',self.u.mem_read(a,4))[0]
 def put(self,a,b):self.u.mem_write(a,struct.pack('<I',b&0xffffffff))
 def string(self,a):return bytes(self.u.mem_read(a,self.get(a-4))) if a else b''
 def alloc(self,s):
  if not s:return 0
  a=self.heap+8;self.heap+=len(s)+16;self.put(a-8,1);self.put(a-4,len(s));self.u.mem_write(a,s+b'\0');return a
 def assign(self,a,s):self.put(a,self.alloc(s))
 def ret(self,extra=0):
  sp=self.u.reg_read(UC_X86_REG_ESP);self.u.reg_write(UC_X86_REG_EIP,self.get(sp));self.u.reg_write(UC_X86_REG_ESP,sp+4+extra)
 def hook(self,u,addr,size,data):
  a=u.reg_read(UC_X86_REG_EAX);b=u.reg_read(UC_X86_REG_EDX);c=u.reg_read(UC_X86_REG_ECX);sp=u.reg_read(UC_X86_REG_ESP)
  extra=0
  if addr==0x404dfc:pass
  elif addr==0x40494c:self.put(a,0)
  elif addr==0x404970:
   for i in range(b):self.put(a+4*i,0)
  elif addr in [0x4049a0,0x4049e4]:self.assign(a,self.string(b))
  elif addr==0x404b34:self.assign(a,bytes([b&255]))
  elif addr==0x404bb0:self.assign(a,bytes(u.mem_read(b+1,bytes(u.mem_read(b,1))[0])))
  elif addr==0x404c0c:u.reg_write(UC_X86_REG_EAX,len(self.string(a)))
  elif addr==0x404c14:self.assign(a,self.string(self.get(a))+self.string(b))
  elif addr==0x404c58:self.assign(a,self.string(b)+self.string(c))
  elif addr==0x404ccc:
   self.assign(a,b''.join(self.string(self.get(sp+4+4*i)) for i in range(b-1,-1,-1)));extra=4*b
  elif addr==0x404d58:
   x,y=self.string(a),self.string(b);flags=u.reg_read(UC_X86_REG_EFLAGS)&~(1|64);u.reg_write(UC_X86_REG_EFLAGS,flags|(64 if x==y else 1 if x<y else 0))
  elif addr==0x404e64:u.reg_write(UC_X86_REG_EAX,self.get(a))
  elif addr==0x404e6c:self.assign(self.get(sp+4),self.string(a)[max(0,b-1):max(0,b-1)+c]);extra=4
  elif addr==0x404eac:
   s=self.string(self.get(a));self.assign(a,s[:b-1]+s[b-1+c:])
  elif addr==0x404f50:
   x=self.string(a);u.reg_write(UC_X86_REG_EAX,(self.string(b).find(x)+1) if x else 0)
  elif addr==0x4036b8:
   u.mem_write(a,bytes(x|y for x,y in zip(u.mem_read(a,c&255),u.mem_read(b,c&255))))
  elif addr==0x408ea8:u.reg_write(UC_X86_REG_EAX,int(self.string(a)))
  elif addr==0x407684:u.mem_write(a,bytes([c&255])*b)
  elif addr==0x4674f2:
   self.pcm=bytes(u.mem_read(0x46fca4,self.get(u.reg_read(UC_X86_REG_EBP)-12)));u.emu_stop();return
  else:return
  self.ret(extra)
 def run(self,addr,s,reference=False):
  u=self.u; self.put(0x800ff00,0x800fe00);self.put(0x800fd00,0)
  u.reg_write(UC_X86_REG_ESP,0x800ff00);u.reg_write(UC_X86_REG_EAX,self.alloc(s));u.reg_write(UC_X86_REG_EDX,0x800fd00)
  if reference:self.put(0x800fd04,u.reg_read(UC_X86_REG_EAX));u.reg_write(UC_X86_REG_EAX,0x800fd04)
  u.emu_start(addr,0x800fe00,count=20000000)
  if u.reg_read(UC_X86_REG_EIP) not in (0x800fe00,0x4674f2):raise RuntimeError('Original procedure exceeded instruction limit')
  return self.string(self.get(0x800fd04 if reference else 0x800fd00))
 def voice(self,n):
  for a,f in [(0x7eabd4,'ip2f'),(0x7ec764,'lp2f'),(0x217d27c,'Sd2f')]:self.u.mem_write(a,Path(f'orig/snf{n}/{f}').read_bytes())
  self.u.mem_write(0x46e6e8,bytes([48+n]));self.put(0x46e6e0,42)
  for i in range(17):self.u.mem_write(0x4ec0f0+8*i,struct.pack('<d',i/16))
if __name__=='__main__':
 o=Oracle();s=(sys.argv[1] if len(sys.argv)>1 else 'Привіт, світе!').encode('cp1251')+b' '
 for addr in [0x465888,0x4659dc,0x465e9c,0x46621c,0x465210,0x46662c]:
  s=o.run(addr,s,addr==0x465210);print(hex(addr),s.hex(),s.decode('cp1251'))
 o.voice(1);o.put(0x46e6e4,5);o.run(0x466e7c,s);print('PCM',len(o.pcm));Path('target/oracle.pcm').write_bytes(o.pcm)
