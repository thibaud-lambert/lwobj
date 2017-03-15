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
                        for arg in args {
                            let index : Vec<_> = arg.split('/').collect();
                            if index.len() != 3 {
                                return Err(LoadingError::WrongNumberOfArguments(nb));
                            }
                            let v = match index[0].parse::<usize>() {
                                Ok(val) => val-1,
                                Err(_) => return Err(LoadingError::Parse(nb)),
                            };
                            let vt = match index[1].parse::<usize>().ok() {
                                Some(val) => Some(val-1),
                                None => None,
                            };
                            let vn = match index[2].parse::<usize>().ok() {
                                Some(val) => Some(val-1),
                                None => None,
                            };
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
                        println!("{}",*g);
                        println!("{}",self.groups.len());
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

}
