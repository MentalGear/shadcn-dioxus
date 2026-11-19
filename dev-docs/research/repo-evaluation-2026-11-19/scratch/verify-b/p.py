import json,collections,sys
seen=set();rows=[];pk=collections.Counter();targets=collections.Counter();cargo=set()
for l in open(sys.argv[1]):
    try:m=json.loads(l)
    except:continue
    if m.get("reason")=="compiler-artifact":
        t=m["target"];targets[(m["package_id"].split("#")[0].rsplit("/",1)[-1],tuple(t["kind"]),t["name"])]+=1
    if m.get("reason")!="compiler-message":continue
    g=m["message"];c=(g.get("code") or {}).get("code")
    if not c or not c.startswith("clippy::"):continue
    sp=[s for s in g["spans"] if s["is_primary"]]
    if not sp:continue
    s=sp[0];k=(c,s["file_name"],s["line_start"],s["column_start"])
    if k in seen:continue
    seen.add(k);rows.append(k)
cnt=collections.Counter(r[0] for r in rows)
for c in ["clippy::expect_fun_call","clippy::redundant_clone","clippy::or_fun_call"]:print(c,cnt[c])
print("perf-ish",[ (c,n) for c,n in cnt.items() if c in("clippy::large_enum_variant","clippy::box_collection","clippy::rc_buffer","clippy::redundant_allocation","clippy::vec_init_then_push","clippy::iter_overeager_cloned","clippy::unnecessary_to_owned","clippy::to_string_in_format_args","clippy::useless_format")])
def istest(f):return "/tests/" in f or "test" in f.split("/")[-1] 
rc=[r for r in rows if r[0]=="clippy::redundant_clone"]
print(collections.Counter(r[1].split("/")[0] for r in rc))
json.dump(rows,open(sys.argv[2],"w"))
print(len(targets));
for k,v in sorted(targets.items()):print(k,v)
