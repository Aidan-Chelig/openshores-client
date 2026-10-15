//! Read-only mounted torso topology diagnostics, independent of texture alpha.
use openshores_client::{asset_mount::AssetMount, model::Face, world::{native_body::NativeDpAvatarGeometry, native_dna::NativeDhDna}};
use std::collections::BTreeMap;

fn topology(points: &[[f64;3]], faces: &[Face]) -> (usize,usize,usize) {
    let mut canonical=BTreeMap::new();
    let ids: Vec<_>=points.iter().enumerate().map(|(i,p)| {
        let key=p.map(|v| {let v=v as f32; if v==0. {0} else {v.to_bits()}});
        *canonical.entry(key).or_insert(i)
    }).collect();
    let mut edges=BTreeMap::new();
    let mut collapsed=0;
    for face in faces {
        let v=face.vertices.map(|i| ids[usize::from(i)]);
        if v[0]==v[1] || v[0]==v[2] || v[1]==v[2] {collapsed+=1;}
        for (a,b) in [(v[0],v[1]),(v[1],v[2]),(v[2],v[0])] {
            *edges.entry((a.min(b),a.max(b))).or_insert(0usize)+=1;
        }
    }
    let mut adjacency: BTreeMap<usize,Vec<usize>>=BTreeMap::new();
    for (&(a,b),&count) in &edges {if count==1 {adjacency.entry(a).or_default().push(b);adjacency.entry(b).or_default().push(a);}}
    while let Some((&seed,_))=adjacency.first_key_value() {
        let mut pending=vec![seed];
        let mut component=Vec::new();
        while let Some(i)=pending.pop() {if let Some(neighbors)=adjacency.remove(&i) {component.push(i);pending.extend(neighbors);}}
        let bounds: [[f64;2];3]=std::array::from_fn(|axis| [component.iter().map(|i| points[*i][axis]).fold(f64::INFINITY,f64::min),component.iter().map(|i| points[*i][axis]).fold(f64::NEG_INFINITY,f64::max)]);
        println!("  boundary component {} vertices bounds {bounds:?}",component.len());
    }
    (edges.values().filter(|n| **n==1).count(),edges.values().filter(|n| **n>2).count(),collapsed)
}

#[test]
#[ignore = "requires OPENSHORES_HAZERON_INSTALLATION; no GPU or original executable"]
fn mounted_torso_topology_diagnostics() {
    let mount=AssetMount::open(std::env::var_os("OPENSHORES_HAZERON_INSTALLATION").expect("installation")).unwrap();
    for many in [false,true] {
        let mut dna=NativeDhDna([0;24]);
        dna.0[1]=1; dna.0[12]=4; dna.0[14]=0x51; dna.0[20]=0x60;
        if many {dna.0[1]=2;dna.0[12]=8;dna.0[14]=0x91;dna.0[20]=0x80;dna.0[5]=2;dna.0[13]=0x60;dna.0[15]=0x40;}
        let avatar=NativeDpAvatarGeometry::generate(&mount,&dna,0,0).unwrap();
        let torso=&avatar.surfaces[0];
        println!("fixture many={many}, merged torso {:?} (boundary, nonmanifold, collapsed)",topology(&torso.surface.geometry.positions,&torso.surface.geometry.faces));
        for source in &torso.sources {
            let points: Vec<_>=source.mesh.vertices.iter().map(|p| p.map(f64::from)).collect();
            println!("{}: {} vertices {} faces topology {:?}",source.resource,points.len(),source.mesh.faces.len(),topology(&points,&source.mesh.faces));
        }
    }
}
