"""Project-bound local theorem interfaces for every existing operation recipe."""
from shared_project import Project,ProjectModel
from shared_vir import ProjectBody
from shared_vir_vectors import project_spec
from vir_body_build import Builder
from operation_signature_vectors import parameters,enclosing
from go_obligations import local_requirements,body_regions


def fragments():
    p=Project(project_spec(False));decl=p.resolve({'package':'fixture/policy','function':'Scopes','receiver':None});m=ProjectModel(p,decl);params=parameters(m);rows=[]
    for n in range(1,57):
        stack=(enclosing(m),) if n in (54,55) else ();b=Builder(m);args=params[n]
        if n==18:
            call=m.derive('GP-OP-19',{'callee':args['callee']});r=b.region(call['inputs'],stack);inputs=call['inputs']
            product,state=r.apply(19,{'callee':args['callee']},[x['id'] for x in r.r['parameters']]);r.yield_(r.apply(18,args,[product,state]))
        else:
            d=m.derive('GP-OP-'+str(n).zfill(2),args);inputs=d['inputs'];r=b.region(inputs,stack)
            r.yield_(r.apply(n,args,[x['id'] for x in r.r['parameters']]))
        d=m.derive('GP-OP-'+str(n).zfill(2),args);v=ProjectBody(p,decl,b.definitions());v.region(r.r,inputs,d['outputs'],stack=stack)
        nodes={x['id']:x for region in body_regions(r.r) for x in region['nodes']}
        rows.append({'op':d['op'],'inputs':inputs,'outputs':d['outputs'],'definitions':b.definitions(),'region':r.r,
            'entry_control_stack':list(stack),'sites':v.sites,
            'local_requirements':[{'node_id':s['node_id'],'requirements':local_requirements(s,nodes[s['node_id']])} for s in v.sites],
            'scope':'local typed theorem-interface fragment,not a full function,ordinary proposition,or source proof'})
    return rows
