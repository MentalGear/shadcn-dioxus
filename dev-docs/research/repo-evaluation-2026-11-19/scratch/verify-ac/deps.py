import tomllib,glob,subprocess
fs=subprocess.run("git ls-files '*Cargo.toml'",shell=True,capture_output=True,text=True,cwd='/home/user/shadcn-dioxus').stdout.split()
for f in fs:
    d=tomllib.load(open('/home/user/shadcn-dioxus/'+f,'rb'))
    def names(sec):
        s=d.get(sec,{});return sorted(s)
    tg=d.get('target',{})
    tnames=sorted({n for v in tg.values() for k in('dependencies',) for n in v.get(k,{})})
    print(f, 'deps',len(names('dependencies')),names('dependencies'),'| target-deps',tnames,'| build',len(names('build-dependencies')),'| dev',len(names('dev-dependencies')))
