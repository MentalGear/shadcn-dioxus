import glob,os,zlib,subprocess,sys
W='/home/user/shadcn-dioxus/docs/assets/preview_bg-dxh92ee26ea6afb8ff3.wasm'
b=open(W,'rb').read()
root='/home/user/shadcn-dioxus/preview/src/components'
fs=glob.glob(root+'/*/component.rs')+glob.glob(root+'/*/variants/*/mod.rs')+glob.glob(root+'/*/style.css')+glob.glob(root+'/*/variants/*/demo.css')+['/home/user/shadcn-dioxus/preview/assets/dx-components-theme.css']
rustb=cssb=0
z=b
for f in sorted(set(fs),key=lambda f:-os.path.getsize(f)):
    if not os.path.exists(f):continue
    t=open(f,'rb').read()
    if len(t)<20:continue
    n=z.count(t)
    if n:
        z=z.replace(t,b'\0'*len(t))
        if f.endswith('.css'):cssb+=len(t)*n
        else:rustb+=len(t)*n
print('zeroed rust bytes',rustb,'css bytes',cssb,'same length',len(z)==len(b))
open('/tmp/zeroed.wasm','wb').write(z)
g0=len(zlib.compress(b,9));g1=len(zlib.compress(z,9))
print('zlib9 original',g0,'zeroed',g1,'delta',g0-g1)
