import subprocess,collections
for c in ['primitives','cli','decision-kernel','preview','labs']:
    fs=subprocess.run(f"git ls-files '{c}/**/*.rs' '{c}/*.rs'",shell=True,capture_output=True,text=True,cwd='/home/user/shadcn-dioxus').stdout.split()
    code=com=0
    for f in fs:
        for l in open('/home/user/shadcn-dioxus/'+f,errors='replace').read().split('\n'):
            if len(l)>100:
                if l.lstrip().startswith('//'): com+=1
                else: code+=1
    print(c,'code>100',code,'comment>100',com)
