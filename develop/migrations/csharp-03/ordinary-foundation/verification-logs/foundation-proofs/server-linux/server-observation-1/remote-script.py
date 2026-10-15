import json,pathlib,subprocess
p=pathlib.Path('/root/mpk-w09-scoped-construction-2c4613aa-linux/status.json')
print(p.read_text())
print(subprocess.run(['ps','-o','pid,ppid,etime,state,args','-p','3711165,3717706,3717713'],text=True,capture_output=True).stdout)
