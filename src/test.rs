use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use obj::*;

#[test]
fn load() {
    let mut expected = ObjData::new();
    expected.vertices = vec![(1.,-1.,-1.,1.),
    (1.,-1.,1.,1.),
    (-1.,-1.,1.,1.),
    (-1.,-1.,-1.,1.),
    (1.,1.,-1.,1.),
    (1.,1.,1.,1.),
    (-1.,1.,1.,1.),
    (-1.,1.,-1.,1.)];
    expected.normals = vec![(0.,-1.,0.),
    (0.,1.,0.),
    (1.,0.,0.),
    (0.,0.,1.),
    (-1.,0.,0.),
    (0.,0.,-1.)];
    expected.faces = vec![ vec![(1,None,Some(0)), (3,None,Some(0)), (0,None,Some(0))],
    vec![(7,None,Some(1)), (5,None,Some(1)), (4,None,Some(1))],
    vec![(4,None,Some(2)), (1,None,Some(2)), (0,None,Some(2))],
    vec![(5,None,Some(3)), (2,None,Some(3)), (1,None,Some(3))],
    vec![(2,None,Some(4)), (7,None,Some(4)), (3,None,Some(4))],
    vec![(0,None,Some(5)), (7,None,Some(5)), (4,None,Some(5))],
    vec![(1,None,Some(0)), (2,None,Some(0)), (3,None,Some(0))],
    vec![(7,None,Some(1)), (6,None,Some(1)), (5,None,Some(1))],
    vec![(4,None,Some(2)), (5,None,Some(2)), (1,None,Some(2))],
    vec![(5,None,Some(3)), (6,None,Some(3)), (2,None,Some(3))],
    vec![(2,None,Some(4)), (6,None,Some(4)), (7,None,Some(4))],
    vec![(0,None,Some(5)), (3,None,Some(5)), (7,None,Some(5))],
    ];
    let obj = Object {
        name : String::from("Cube"),
        primitives : vec![0,1,2,3,4,5,6,7,8,9,10,11]
    };
    expected.objects = vec![obj];
    let f = File::open("cube.obj").unwrap();
    let mut input = BufReader::new(f);
    let data = ObjData::load(&mut input).ok().unwrap();
    assert_eq!(expected.vertices,data.vertices);
    assert_eq!(expected.normals,data.normals);
    assert_eq!(expected.texcoords,data.texcoords);
    assert_eq!(expected.faces,data.faces);
    assert_eq!(expected.objects,data.objects);
}

#[test]
fn write() {
    let mut expected = ObjData::new();
    expected.vertices = vec![(1.,-1.,-1.,1.),
    (1.,-1.,1.,1.),
    (-1.,-1.,1.,1.),
    (-1.,-1.,-1.,1.),
    (1.,1.,-1.,1.),
    (1.,1.,1.,1.),
    (-1.,1.,1.,1.),
    (-1.,1.,-1.,1.)];
    expected.normals = vec![(0.,-1.,0.),
    (0.,1.,0.),
    (1.,0.,0.),
    (0.,0.,1.),
    (-1.,0.,0.),
    (0.,0.,-1.)];
    expected.faces = vec![ vec![(1,None,Some(0)), (3,None,Some(0)), (0,None,Some(0))],
    vec![(7,None,Some(1)), (5,None,Some(1)), (4,None,Some(1))],
    vec![(4,None,Some(2)), (1,None,Some(2)), (0,None,Some(2))],
    vec![(5,None,Some(3)), (2,None,Some(3)), (1,None,Some(3))],
    vec![(2,None,Some(4)), (7,None,Some(4)), (3,None,Some(4))],
    vec![(0,None,Some(5)), (7,None,Some(5)), (4,None,Some(5))],
    vec![(1,None,Some(0)), (2,None,Some(0)), (3,None,Some(0))],
    vec![(7,None,Some(1)), (6,None,Some(1)), (5,None,Some(1))],
    vec![(4,None,Some(2)), (5,None,Some(2)), (1,None,Some(2))],
    vec![(5,None,Some(3)), (6,None,Some(3)), (2,None,Some(3))],
    vec![(2,None,Some(4)), (6,None,Some(4)), (7,None,Some(4))],
    vec![(0,None,Some(5)), (3,None,Some(5)), (7,None,Some(5))],
    ];
    let obj = Object {
        name : String::from("Cube"),
        primitives : vec![0,1,2,3,4,5,6,7,8,9,10,11]
    };
    expected.objects = vec![obj];
    {
        let f2 = File::create("tmp.obj").unwrap();
        let mut output = BufWriter::new(f2);
        assert!(expected.write(&mut output).is_ok());
    }
    let f1 = File::open("tmp.obj").unwrap();
    let mut input = BufReader::new(f1);
    let data = ObjData::load(&mut input).ok().unwrap();
    assert_eq!(expected.vertices,data.vertices);
    assert_eq!(expected.normals,data.normals);
    assert_eq!(expected.texcoords,data.texcoords);
    assert_eq!(expected.faces,data.faces);
}

#[test]
fn read_write_read() {
    let f = File::open("cube.obj").unwrap();
    let mut input = BufReader::new(f);
    let data = ObjData::load(&mut input).ok().unwrap();
    {
        let f2 = File::create("rwr.obj").unwrap();
        let mut output = BufWriter::new(f2);
        assert!(data.write(&mut output).is_ok());
    }
    let f1 = File::open("rwr.obj").unwrap();
    let mut input = BufReader::new(f1);
    let reload = ObjData::load(&mut input).ok().unwrap();
    assert_eq!(reload.vertices,data.vertices);
    assert_eq!(reload.normals,data.normals);
    assert_eq!(reload.texcoords,data.texcoords);
    assert_eq!(reload.faces,data.faces);
    assert_eq!(reload.objects,data.objects);
    assert_eq!(reload.groups,data.groups);
}
