from pathlib import Path
import datetime,json,os,signal,subprocess
OUT=Path(__file__).resolve().parent
pid=27590;parent=27589
snapshot=subprocess.check_output(['/bin/ps','-p',str(pid),'-o','pid=,ppid=,command='],text=True)
assert snapshot.split()[:2]==[str(pid),str(parent)] and '/Users/kazuyoshitoshiya/ff/mpk/target/release/deps/go_practical_array_equality-c2fda9ef323cc6a5 'in snapshot
cargo=subprocess.check_output(['/bin/ps','-p',str(parent),'-o','pid=,ppid=,command='],text=True)
assert cargo.split()[:2]==[str(parent),'27584'] and '--test go_practical_array_equality 'in cargo
record={'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'reason':'The typed-definition root has already failed: ByteNotEqual mutation is accepted by both cores because a direct concrete complement law is missing. Preserve the original partial run, stop only its exact Rust child, add that law, then run the complete affected suite again. No unrelated process or passing suite is stopped.','process_before_signal':snapshot,'parent_before_signal':cargo,'target_pid':pid,'signal':signal.SIGTERM,'signal_sent':False,'original_outputs_reconstructed':False}
result=OUT/'result.json';result.write_text(json.dumps(record,indent=2)+'\n')
os.kill(pid,signal.SIGTERM)
record['signal_sent']=True;result.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'target_pid':pid,'signal_sent':True}))
