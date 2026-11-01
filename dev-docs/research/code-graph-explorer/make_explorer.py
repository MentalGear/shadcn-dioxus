import sys
t=open(sys.argv[1]).read();g=open(sys.argv[2]).read().replace('</','<\\/')
open(sys.argv[3],'w').write(t.replace('__GRAPH__',g))
