use std::io::BufRead;
use std::io::Write;
use std::io;
use std::str::FromStr;
use std::collections::HashSet;

#[derive(Debug)]
pub enum LoadingError {
    InvalidLine(usize),
    WrongNumberOfArguments(usize),
    Parse(usize),
    Io(io::Error),
}

#[derive(PartialEq, Debug)]
pub struct Group {
    pub name : String,
    pub indexes : HashSet<usize>,
}

#[derive(PartialEq, PartialOrd,Debug)]
pub struct Object {
    pub name : String,
    pub primitives : Vec<usize>
}

/// A struct containing all data store by wavefront.
pub struct ObjData {
    /// List of vertices `(x,y,z,w)`.
    /// Its coordinates are (x,y,z) and w is the weight for rational curves and surfaces.
    pub vertices : Vec<(f32,f32,f32,f32)>,
    /// List of normal vector with componetns `(x,y,z)`.
    pub normals : Vec<(f32,f32,f32)>,
    /// List of texture coordinates `(u,v,w)`.
    /// u and v are the value for the horizontal and vertical direction.
    /// w is the value for the depth of the texture.
    pub texcoords : Vec<(f32,f32,f32)>,
    /// List of faces.
    /// Each Face is a list of `(v,vt,vn)`.
    /// v is the index of vertex.
    /// vt is the index of its texture coordinate if it has one.
    /// vn is the index of its normal vector if it has one.
    pub faces : Vec<Vec<(usize,Option<usize>,Option<usize>)>>,
    /// List of Objects
    pub objects : Vec<Object>,
    /// List of groups
    pub groups : Vec<Group>
}

impl From<io::Error> for LoadingError {
    fn from(err : io::Error) -> LoadingError {
        LoadingError::Io(err)
    }
}

fn parse<T : FromStr>(it : Vec<&str>, nb : usize) -> Result<Vec<T>, LoadingError> {
    let mut vec : Vec<T> = Vec::new();
    for s in it {
        let val = match s.parse::<T>() {
            Ok(v) => v,
            Err(_) => return Err(LoadingError::Parse(nb)),
        };
        vec.push(val);
    }
    return Ok(vec);
}

impl Group {
    pub fn new(n : String) -> Group {
        Group {
            name : n,
            indexes : HashSet::new()
        }
    }
}

impl Object {
    pub fn new(n : String) -> Object {
        Object {
            name : n,
            primitives : Vec::new()
        }
    }
}


impl ObjData {
    /// Constructs a new empty `ObjData`.
    ///
    /// # Examples
    ///
    /// ```
    /// use lwobj::ObjData;
    ///
    /// let data = ObjData::new();
    /// ```
    pub fn new() -> ObjData {
        ObjData {
            vertices : Vec::new(),
            normals : Vec::new(),
            texcoords : Vec::new(),
            faces : Vec::new(),
            objects : Vec::new(),
            groups : Vec::new(),
        }
    }

    /// Load an `ObjData` from a `BufReader`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use std::io::BufReader;
    /// use lwobj::ObjData;
    ///
    /// let f = File::open("cube.obj").unwrap();
    /// let mut input = BufReader::new(f);
    /// let data = ObjData::load(&mut input).ok().unwrap();
    /// ```
    pub fn load<R : io::Read>(input : &mut io::BufReader<R>) -> Result<ObjData,LoadingError> {
        let mut data = ObjData::new();
        let mut buf = String::new();
        let mut nb : usize = 0;
        let mut actif_groups : Vec<usize> = Vec::new();
        let mut obj : Option<usize> = None;
        while try!(input.read_line(&mut buf)) > 0 {
            // Skip comment
            if buf.chars().next().unwrap() != '#' {
                let mut iter = buf.split_whitespace();
                let identifier = iter.next();
                let args : Vec<_> = iter.collect();
                if identifier.is_none() {continue;}
                match identifier.unwrap() {
                    "v" => {
                        let values = try!(parse::<f32>(args,nb));
                        if values.len() == 4 {
                            data.vertices.push((values[0],values[1],values[2],values[3]));
                        } else if values.len() == 3 {
                            data.vertices.push((values[0],values[1],values[2],1.0));
                        } else {
                            return Err(LoadingError::WrongNumberOfArguments(nb));
                        }
                    },
                    "vn" => {
                        let values = try!(parse::<f32>(args,nb));
                        if values.len() == 3 {
                            data.normals.push((values[0],values[1],values[2]));
                        } else {
                            return Err(LoadingError::WrongNumberOfArguments(nb));
                        }
                    },
                    "vt" => {
                        let values = try!(parse::<f32>(args,nb));
                        if values.len() == 3 {
                            data.texcoords.push((values[0],values[1],values[2]));
                        } else if values.len() == 2 {
                            data.texcoords.push((values[0],values[1],0.));
                        } else if values.len() == 1 {
                            data.texcoords.push((values[0],0.,0.));
                        } else {
                            return Err(LoadingError::WrongNumberOfArguments(nb));
                        }
                    },
                    "s" => {
                        // Not supported
                    },
                    "f" => {
                        let mut vec : Vec<(usize,Option<usize>,Option<usize>)> = Vec::new();
                        if args.len() < 3 {return Err(LoadingError::WrongNumberOfArguments(nb))}
                        for arg in args {
                            let index : Vec<_> = arg.split('/').collect();
                            if index.len() == 0 || index.len() > 3 {
                                return Err(LoadingError::WrongNumberOfArguments(nb));
                            }
                            let v = match index[0].parse::<usize>() {
                                Ok(val) => val-1,
                                Err(_) => return Err(LoadingError::Parse(nb)),
                            };
                            let mut vt = None;
                            if index.len() >= 2 {
                                vt = match index[1].parse::<usize>().ok() {
                                    Some(val) => Some(val-1),
                                    None => None,
                                };
                            }
                            let mut vn = None;
                            if index.len() == 3 {
                                vn = match index[2].parse::<usize>().ok() {
                                    Some(val) => Some(val-1),
                                    None => None,
                                };
                            }
                            vec.push((v,vt,vn));
                        }
                        data.faces.push(vec);
                        if obj.is_none() {
                            data.objects.push(Object::new(String::new()));
                            obj = Some(data.objects.len()-1);
                        }
                        data.objects[obj.unwrap()].primitives.push(data.faces.len()-1);
                        for g in actif_groups.iter() {
                            data.groups[*g].indexes.insert(data.faces.len()-1);
                        }
                    },
                    "o" => {
                        if args.len() == 0 {
                            return Err(LoadingError::WrongNumberOfArguments(nb));
                        }
                        let mut name = String::new();
                        let mut args_it = args.iter();
                        name += args_it.next().unwrap();
                        for arg in args_it {
                            name += " ";
                            name += arg;
                        }
                        data.objects.push(Object::new(String::from(name)));
                        obj = Some(data.objects.len()-1);
                    },
                    "g" => {
                        actif_groups.clear();
                        for arg in args {
                            let mut found = false;
                            for (i,g) in data.groups.iter().enumerate() {
                                if g.name == arg {
                                    actif_groups.push(i);
                                    found = true;
                                }
                            }
                            if !found {
                                data.groups.push(Group::new(String::from(arg)));
                                actif_groups.push(data.groups.len()-1);
                            }
                        }
                    },
                    _ => return Err(LoadingError::InvalidLine(nb)),
                }
            }
            nb += 1;
            buf.clear();
        }
        return Ok(data);
    }

    /// Write in wavefront format in file.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use std::io::BufWriter;
    /// use std::io::BufReader;
    /// use lwobj::ObjData;
    ///
    /// let f1 = File::open("cube.obj").unwrap();
    /// let mut input = BufReader::new(f1);
    /// let data = ObjData::load(&mut input).ok().unwrap();
    /// let f2 = File::create("tmp.obj").unwrap();
    /// let mut output = BufWriter::new(f2);
    /// assert!(data.write(&mut output).is_ok());
    /// ```
    pub fn write<W : io::Write>(&self, output : &mut io::BufWriter<W>) -> Result<(),LoadingError> {
        // Write vertices
        for &(x,y,z,w) in &self.vertices {
            let line : String = format!("v {} {} {} {}\n",x,y,z,w);
            try!(output.write_all(line.as_bytes()));
        }

        // Write normals
        for &(x,y,z) in &self.normals {
            let line : String = format!("vn {} {} {}\n",x,y,z);
            try!(output.write_all(line.as_bytes()));
        }

        // Write texcoords
        for &(u,v,w) in &self.texcoords {
            let line : String = format!("vt {} {} {}\n",u,v,w);
            try!(output.write_all(line.as_bytes()));
        }

        // Write faces
        let mut actif_groups : Vec<usize> = Vec::new();
        for o in &self.objects {
            if o.name != String::new() {
                let line : String = format!("o {}\n",o.name);
                try!(output.write_all(line.as_bytes()));
            }
            for i in &o.primitives {
                let mut groups : Vec<usize> = Vec::new();
                for (j,g) in self.groups.iter().enumerate() {
                    if g.indexes.contains(i) {
                        groups.push(j);
                    }
                }
                if actif_groups != groups {
                    actif_groups = groups;
                    try!(output.write_all("g".as_bytes()));
                    for g in &actif_groups {
                        try!(output.write_all(" ".as_bytes()));
                        try!(output.write_all(&self.groups[*g].name.as_bytes()));
                    }
                    try!(output.write_all("\n".as_bytes()));
                }

                try!(output.write_all("f".as_bytes()));
                for &(v,vt,vn) in &self.faces[*i] {
                    let vt_str = match vt {
                        Some(val) => (val+1).to_string(),
                        None => "".to_string(),
                    };
                    let vn_str = match vn {
                        Some(val) => (val+1).to_string(),
                        None => "".to_string(),
                    };
                    let arg : String = format!(" {}/{}/{}",v+1,vt_str,vn_str);
                    try!(output.write_all(arg.as_bytes()));
                }
                try!(output.write_all("\n".as_bytes()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;
    use std::io::BufWriter;
    use std::str;
    use obj::*;

    #[test]
    fn load_invalid_line() {
        let obj_str =
        r#"o Test
        az 1. -2.00 -3.5
        v 1 -1 3.
        v -1 -1d 1 0.5
        v -1 -1.000000 -1.000000"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        match ObjData::load(&mut input).err().unwrap() {
            LoadingError::InvalidLine(line) => assert!(line == 1),
            _ => assert!(false),
        };
    }

    #[test]
    fn load_vertices() {
        let expected = vec![(1.,-2.,-3.5,1f32),
        (1.,-1.,1.,1.),
        (-1.,-1.,1.,0.5),
        (-1.,-1.,-1.,1.)];
        let obj_str =
        r#"o Test
        v 1. -2.00 -3.5
        v 1 -1 1
        v -1 -1 1 0.5
        v -1 -1.000000 -1.000000"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        let data = ObjData::load(&mut input).ok().unwrap();
        assert_eq!(expected,data.vertices);
    }

    #[test]
    fn load_vertices_wrong_number_of_arguments() {
        let obj_str =
        r#"o Test
        v 1. -2.00 -3.5
        v 1 -1
        v -1 -1 1 0.5
        v -1 -1.000000 -1.000000"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        match ObjData::load(&mut input).err().unwrap() {
            LoadingError::WrongNumberOfArguments(line) => assert!(line == 2),
            _ => assert!(false),
        };
    }

    #[test]
    fn load_vertices_parse_err() {
        let obj_str =
        r#"o Test
        v 1. -2.00 -3.5
        v 1 -1 3.
        v -1 -1d 1 0.5
        v -1 -1.000000 -1.000000"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        match ObjData::load(&mut input).err().unwrap() {
            LoadingError::Parse(line) => assert!(line == 3),
            _ => assert!(false),
        };
    }

    #[test]
    fn load_normals() {
        let expected = vec![(1.,-2.,-3.5),
        (1.,-1.,1.),
        (-1.,-1.,1.),
        (-1.,-1.,-1.)];
        let obj_str =
        r#"o Test
        vn 1. -2.00 -3.5
        vn 1 -1 1
        vn -1 -1 1
        vn -1 -1.000000 -1.000000"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        let data = ObjData::load(&mut input).ok().unwrap();
        assert_eq!(expected,data.normals);
    }

    #[test]
    fn load_normals_wrong_number_of_arguments() {
        let obj_str =
        r#"o Test
        vn 1. -2.00 -3.5
        vn 1 -1 2. 1
        vn -1 -1 1
        vn -1 -1.000000 -1.000000"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        match ObjData::load(&mut input).err().unwrap() {
            LoadingError::WrongNumberOfArguments(line) => assert!(line == 2),
            _ => assert!(false),
        };

        let obj_str =
        r#"o Test
        v 1. -2.00 -3.5
        v 1 -1
        v -1 -1 1
        v -1 -1.000000 -1.000000"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        match ObjData::load(&mut input).err().unwrap() {
            LoadingError::WrongNumberOfArguments(line) => assert!(line == 2),
            _ => assert!(false),
        };
    }

    #[test]
    fn load_texcoords() {
        let expected = vec![(0.,1.,0.),
        (0.,0.5,0.),
        (0f32,1f32,1f32),
        (1.,1.,0.5)];
        let obj_str =
        r#"o Test
        vt 0. 1.00
        vt 0 0.5
        vt 0 1 1
        vt 1 1. 0.5"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        let data = ObjData::load(&mut input).ok().unwrap();
        assert_eq!(expected,data.texcoords);
    }

    #[test]
    fn load_faces() {
        let expected = vec![ vec![(1,None,Some(0)), (3,None,Some(0)), (0,None,Some(0))],
        vec![(7,None,None), (5,None,None), (4,None,None)],
        vec![(3,None,None), (4,None,None), (5,None,None)],
        vec![(7,Some(2),Some(1)), (5,Some(4),Some(2)), (4,Some(6),Some(0))],
        vec![(8,Some(3),None), (6,Some(2),None), (2,Some(1),None)],
        ];
        let obj_str =
        r#"o Test
        f 2//1 4//1 1//1
        f 8 6 5
        f 4// 5// 6//
        f 8/3/2 6/5/3 5/7/1
        f 9/4/ 7/3/ 3/2/"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        let data = ObjData::load(&mut input).ok().unwrap();
        assert_eq!(expected,data.faces);
    }

    #[test]
    fn load_faces_wrong_number_of_arguments() {
        let obj_str =
        r#"o Test
        f 2//1 4//1 1//1
        f 8 6 5
        f 4/// 5// 6//
        f 8/3/2 6/5/3 5/7/1"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        match ObjData::load(&mut input).err().unwrap() {
            LoadingError::WrongNumberOfArguments(line) => assert!(line == 3),
            _ => assert!(false),
        };

        let obj_str =
        r#"o Test
        f 2//1 4//1 1//1
        f 8 6
        f 4// 5// 6//
        f 8/3/2 6/5/3 5/7/1
        f 9/4/ 7/3/ 3/2/"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        match ObjData::load(&mut input).err().unwrap() {
            LoadingError::WrongNumberOfArguments(line) => assert!(line == 2),
            _ => assert!(false),
        };
    }

    #[test]
    fn load_faces_parse_err() {
        let obj_str =
        r#"o Test
        f 2//1 4//1 1//1
        f 8.5 6 5
        f 4// 5// 6//"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        match ObjData::load(&mut input).err().unwrap() {
            LoadingError::Parse(line) => assert!(line == 2),
            _ => assert!(false),
        };
    }

    #[test]
    fn load_object_wrong_number_of_arguments() {
        let obj_str =
        r#"o
        f 2//1 4//1 1//1
        f 8.5 6 5
        f 4// 5// 6//"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        match ObjData::load(&mut input).err().unwrap() {
            LoadingError::WrongNumberOfArguments(line) => assert!(line == 0),
            _ => assert!(false),
        };
    }

    #[test]
    fn load_unamed_object() {
        let obj = Object {
            name : String::from(""),
            primitives : vec![0,1,2,3,4]
        };
        let expected = vec![obj];
        let obj_str =
        r#"f 2//1 4//1 1//1
        f 8 6 5
        f 4// 5// 6//
        f 8/3/2 6/5/3 5/7/1
        f 9/4/ 7/3/ 3/2/"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        let data = ObjData::load(&mut input).ok().unwrap();
        assert_eq!(expected,data.objects);
    }

    #[test]
    fn load_object() {
        let obj = Object {
            name : String::from("Cube"),
            primitives : vec![0,1,2,3,4]
        };
        let expected = vec![obj];
        let obj_str =
        r#"o Cube
        f 2//1 4//1 1//1
        f 8 6 5
        f 4// 5// 6//
        f 8/3/2 6/5/3 5/7/1
        f 9/4/ 7/3/ 3/2/"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        let data = ObjData::load(&mut input).ok().unwrap();
        assert_eq!(expected,data.objects);
    }

    #[test]
    fn load_several_objects() {
        let obj1 = Object {
            name : String::from(""),
            primitives : vec![0,1,2,]
        };
        let obj2 = Object {
            name : String::from("Cube"),
            primitives : vec![3,4]
        };
        let obj3 = Object {
            name : String::from("Test"),
            primitives : vec![5]
        };
        let expected = vec![obj1,obj2,obj3];
        let obj_str =
        r#"f 2//1 4//1 1//1
        f 8 6 5
        f 4// 5// 6//
        o Cube
        f 8/3/2 6/5/3 5/7/1
        f 9/4/ 7/3/ 3/2/
        o Test
        f 4 3 5"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        let data = ObjData::load(&mut input).ok().unwrap();
        assert_eq!(expected,data.objects);
    }

    #[test]
    fn load_group() {
        let gr1 = Group {
            name : String::from("gr1"),
            indexes : vec!(0,1,2,3).into_iter().collect()
        };
        let gr2 = Group {
            name : String::from("gr2"),
            indexes : vec!(0,1,5).into_iter().collect()
        };
        let gr3 = Group {
            name : String::from("gr3"),
            indexes : vec!(4).into_iter().collect()
        };
        let expected = vec![gr1,gr2,gr3];
        let obj_str =
        r#"g gr1 gr2
        f 2//1 4//1 1//1
        f 8 6 5
        g gr1
        f 4// 5// 6//
        f 8/3/2 6/5/3 5/7/1
        g gr3
        f 9/4/ 7/3/ 3/2/
        g gr2
        f 9/4/ 7/3/ 3/2/"#;

        let mut input = BufReader::new(obj_str.as_bytes());
        let data = ObjData::load(&mut input).ok().unwrap();
        assert_eq!(expected,data.groups);
    }

    #[test]
    fn write_vertices() {
        let mut data = ObjData::new();
        data.vertices = vec![(1.,-2.,-3.5,1f32),
        (1.,-1.,1.,1.),
        (-1.,-1.,1.,0.5),
        (-1.,-1.,-1.,1.)];
        let expected =
        r#"v 1 -2 -3.5 1
v 1 -1 1 1
v -1 -1 1 0.5
v -1 -1 -1 1
"#;

        let mut output = BufWriter::new(Vec::<u8>::new());
        assert!(data.write(&mut output).is_ok());
        let buf = output.into_inner().unwrap();
        assert_eq!(expected,str::from_utf8(&buf).unwrap());
    }

    #[test]
    fn write_normals() {
        let mut data = ObjData::new();
        data.normals = vec![(1.,-2.,-3.5),
        (1.,-1.,1.),
        (-1.,-1.,1.),
        (-1.,-1.,-1.)];
        let expected =
        r#"vn 1 -2 -3.5
vn 1 -1 1
vn -1 -1 1
vn -1 -1 -1
"#;
        let mut output = BufWriter::new(Vec::<u8>::new());
        assert!(data.write(&mut output).is_ok());
        let buf = output.into_inner().unwrap();
        assert_eq!(expected,str::from_utf8(&buf).unwrap());
    }


    #[test]
    fn write_texcoords() {
        let mut data = ObjData::new();
        data.texcoords = vec![(1.,1.,0.5),
        (0.,0.,0.),
        (0.5,1.,0.),
        (1.,0.,1.)];
        let expected =
        r#"vt 1 1 0.5
vt 0 0 0
vt 0.5 1 0
vt 1 0 1
"#;
        let mut output = BufWriter::new(Vec::<u8>::new());
        assert!(data.write(&mut output).is_ok());
        let buf = output.into_inner().unwrap();
        assert_eq!(expected,str::from_utf8(&buf).unwrap());
    }

    #[test]
    fn write_faces() {
        let mut data = ObjData::new();
        data.faces = vec![ vec![(1,None,Some(0)), (3,None,Some(0)), (0,None,Some(0))],
        vec![(7,None,None), (5,None,None), (4,None,None)],
        vec![(3,None,None), (4,None,None), (5,None,None)],
        vec![(7,Some(2),Some(1)), (5,Some(4),Some(2)), (4,Some(6),Some(0))],
        vec![(8,Some(3),None), (6,Some(2),None), (2,Some(1),None)],
        ];
        let obj = Object {
            name : String::from(""),
            primitives : vec![0,1,2,3,4]
        };
        data.objects = vec![obj];
        let expected =
        r#"f 2//1 4//1 1//1
f 8// 6// 5//
f 4// 5// 6//
f 8/3/2 6/5/3 5/7/1
f 9/4/ 7/3/ 3/2/
"#;
        let mut output = BufWriter::new(Vec::<u8>::new());
        assert!(data.write(&mut output).is_ok());
        let buf = output.into_inner().unwrap();
        assert_eq!(expected,str::from_utf8(&buf).unwrap());
    }

    #[test]
    fn write_objects() {
        let mut data = ObjData::new();
        data.faces = vec![ vec![(1,None,Some(0)), (3,None,Some(0)), (0,None,Some(0))],
        vec![(7,None,None), (5,None,None), (4,None,None)],
        vec![(3,None,None), (4,None,None), (5,None,None)],
        vec![(7,Some(2),Some(1)), (5,Some(4),Some(2)), (4,Some(6),Some(0))],
        vec![(8,Some(3),None), (6,Some(2),None), (2,Some(1),None)],
        ];
        let obj1 = Object {
            name : String::from(""),
            primitives : vec![0,1]
        };
        let obj2 = Object {
            name : String::from("Test"),
            primitives : vec![2,3,4]
        };
        data.objects = vec![obj1,obj2];
        let expected =
        r#"f 2//1 4//1 1//1
f 8// 6// 5//
o Test
f 4// 5// 6//
f 8/3/2 6/5/3 5/7/1
f 9/4/ 7/3/ 3/2/
"#;
        let mut output = BufWriter::new(Vec::<u8>::new());
        assert!(data.write(&mut output).is_ok());
        let buf = output.into_inner().unwrap();
        assert_eq!(expected,str::from_utf8(&buf).unwrap());
    }

    #[test]
    fn write_groups() {
        let mut data = ObjData::new();
        data.faces = vec![ vec![(1,None,Some(0)), (3,None,Some(0)), (0,None,Some(0))],
        vec![(7,None,None), (5,None,None), (4,None,None)],
        vec![(3,None,None), (4,None,None), (5,None,None)],
        vec![(7,Some(2),Some(1)), (5,Some(4),Some(2)), (4,Some(6),Some(0))],
        vec![(8,Some(3),None), (6,Some(2),None), (2,Some(1),None)],
        ];
        let obj = Object {
            name : String::from(""),
            primitives : vec![0,1,2,3,4]
        };
        data.objects = vec![obj];
        let gr1 = Group {
            name : String::from("gr1"),
            indexes : vec!(0,1).into_iter().collect()
        };
        let gr2 = Group {
            name : String::from("gr2"),
            indexes : vec!(0,1,2).into_iter().collect()
        };
        let gr3 = Group {
            name : String::from("gr3"),
            indexes : vec!(3,4).into_iter().collect()
        };
        data.groups = vec![gr1,gr2,gr3];
        let expected =
        r#"g gr1 gr2
f 2//1 4//1 1//1
f 8// 6// 5//
g gr2
f 4// 5// 6//
g gr3
f 8/3/2 6/5/3 5/7/1
f 9/4/ 7/3/ 3/2/
"#;
        let mut output = BufWriter::new(Vec::<u8>::new());
        assert!(data.write(&mut output).is_ok());
        let buf = output.into_inner().unwrap();
        assert_eq!(expected,str::from_utf8(&buf).unwrap());
    }
}
