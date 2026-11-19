cd /home/user/shadcn-dioxus
python3 -I - <<'PY'
import json,os,statistics,glob
m=[os.path.basename(x) for x in json.load(open('component.json'))['members']]
sizes={}
for c in m:
    d='preview/src/components/'+c;ex=set(json.load(open(d+'/component.json')).get('exclude',[]))
    t=0
    for dp,dn,fn in os.walk(d):
        rel=os.path.relpath(dp,d)
        if rel!='.' and rel.split('/')[0] in ex: continue
        for f in fn:
            r=os.path.normpath(os.path.join(rel,f))
            if r.split('/')[0] in ex: continue
            t+=os.path.getsize(os.path.join(dp,f))
    sizes[c]=t
v=sorted(sizes.values());print('members',len(m),'min',v[0],min(sizes,key=sizes.get),'median',statistics.median(v),'max',v[-1],max(sizes,key=sizes.get),'sum',sum(v))
dep=0;pinned=0;nopr=[]
for c in m:
    j=json.load(open('preview/src/components/'+c+'/component.json'))
    ds=[d for d in j.get('cargoDependencies',[]) if isinstance(d,dict) and d.get('name')=='dioxus-primitives']
    if ds:
        dep+=1
        if any(k in ds[0] for k in('rev','tag','branch','version')):pinned+=1
    else:nopr.append(c)
print('components with dioxus-primitives dep',dep,'pinned (rev/tag/branch/version)',pinned,'without it',len(nopr),nopr[:6])
j=json.load(open('playwright/durations.json'));s=sum(v['seconds'] for v in j['files'].values());print('durations sum',s,'meta total',j['meta']['total_seconds'],'files',len(j['files']),'max file',max(v['seconds'] for v in j['files'].values()))
PY
