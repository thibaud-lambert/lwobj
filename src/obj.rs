use std::io::BufRead;
use std::io::Write;
use std::io;
use std::str::SplitWhitespace;
use std::str::FromStr;

#[derive(Debug)]
pub enum LoadingError {
    InvalidLine(usize),
    WrongNumberOfArguments(usize),
    Parse(usize),
    Io(io::Error),
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
    pub faces : Vec<Vec<(usize,Option<usize>,Option<usize>)>>
}

impl From<io::Error> for LoadingError {
    fn from(err : io::Error) -> LoadingError {
        LoadingError::Io(err)
    }
}

fn parse<T : FromStr>(it : SplitWhitespace, nb : usize) -> Result<Vec<T>, LoadingError> {
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
        while try!(input.read_line(&mut buf)) > 0 {
            // Skip comment
            if buf.chars().next().unwrap() != '#' {
                let mut iter = buf.split_whitespace();
                match iter.next() {
                    Some("v") => {
                        let args = try!(parse::<f32>(iter,nb));
                        if args.len() == 4 {
                            data.vertices.push((args[0],args[1],args[2],args[3]));
                        } else if args.len() == 3 {
                            data.vertices.push((args[0],args[1],args[2],1.0));
                        } else {
                            return Err(LoadingError::WrongNumberOfArguments(nb));
                        }
                    },
                    Some("vn") => {
                        let args = try!(parse::<f32>(iter,nb));
                        if args.len() == 3 {
                            data.normals.push((args[0],args[1],args[2]));
                        } else {
                            return Err(LoadingError::WrongNumberOfArguments(nb));
                        }
                    },
                    Some("vt") => {
                        let args = try!(parse::<f32>(iter,nb));
                        if args.len() == 3 {
                            data.texcoords.push((args[0],args[1],args[2]));
                        } else if args.len() == 2 {
                            data.texcoords.push((args[0],args[1],0.));
                        } else if args.len() == 1 {
                            data.texcoords.push((args[0],0.,0.));
                        } else {
                            return Err(LoadingError::WrongNumberOfArguments(nb));
                        }
                    },
                    Some("s") => {
                        // Not supported
                    },
                    Some("f") => {
                        let mut vec : Vec<(usize,Option<usize>,Option<usize>)> = Vec::new();
                        for arg in iter {
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
                    },
                    Some("o") => {
                        // Not supported
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
        for indexes in &self.faces {
            try!(output.write_all("f".as_bytes()));
            for &(v,vt,vn) in indexes {
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
        let f = File::open("cube.obj").unwrap();
        let mut input = BufReader::new(f);
        let data = ObjData::load(&mut input).ok().unwrap();
        assert_eq!(expected.vertices,data.vertices);
        assert_eq!(expected.normals,data.normals);
        assert_eq!(expected.texcoords,data.texcoords);
        assert_eq!(expected.faces,data.faces);
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

}
