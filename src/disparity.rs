use std::fmt::{Display, Formatter, Error};

#[derive(Debug, Clone)]
pub struct Location {
    pieces: Vec<String>,
}

impl Location {
    pub fn new(pieces: Vec<&str>) -> Self {
        Location { pieces: pieces.clone().into_iter().map(|a| a.to_string()).collect() }
    }

    pub fn add(&self, piece: &str) -> Self {
        let mut new_location = self.clone();
        new_location.pieces.push(piece.to_string());
        new_location
//        self.pieces.push(piece.to_string());
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let output = self.pieces.join("::");
        write!(f, "{}", output)
    }
}




#[derive(Debug, Clone)]
pub struct Disparity {
    message: String,
    location: Location,
}

impl Disparity {
    pub fn new(message: &str, location: Location) -> Self {
        Disparity {
            message: message.to_string(),
            location: location,
        }
    }
}

impl Display for Disparity {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} Failed at:\n {}", self.message, self.location)
    }
}

#[derive(Debug)]
pub struct DisparityList {
    pub inner: Vec<Disparity>,
}

impl DisparityList {
    pub fn new() -> Self {
        DisparityList { inner: Vec::new() }
    }

    pub fn push(&mut self, dis: Disparity) {
        self.inner.push(dis);
    }

    pub fn option_push(&mut self, dis: Option<Disparity>) -> bool {
        dis.map(|disparity| self.push(disparity)).is_some()
    }

    pub fn merge(&mut self, other_list: DisparityList) {
        other_list.inner.into_iter().for_each(|disparity| self.push(disparity));
    }
}



impl Display for DisparityList {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let text_list: Vec<String> = self.inner.iter().map(|dis| format!("{}\n", dis)).collect();
        let text = text_list.join("\n");
        f.write_str(&text)
    }
}
